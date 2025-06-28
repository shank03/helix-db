use anyhow::Result;
use aws_config::BehaviorVersion;
use aws_sdk_s3::Client;
use sonic_rs::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use std::fs::File;
use std::io::{Read, Write};
use std::{net::SocketAddr, process::Command};
use tokio::net::TcpListener;

// Constants for timeouts
//const SOCKET_TIMEOUT: Duration = Duration::from_secs(30);

// make sure build is run in sudo mode

#[derive(Debug, Deserialize, Serialize)]
pub struct HBuildDeployRequest {
    user_id: String,
    instance_id: String,
    version: String,
}

#[derive(Debug, Serialize)]
pub struct DeployResponse {
    success: bool,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl DeployResponse {
    fn success(message: String) -> Self {
        Self {
            success: true,
            message,
            error: None,
        }
    }

    fn error(message: String, error: String) -> Self {
        Self {
            success: false,
            message,
            error: Some(error),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), AdminError> {
    println!("Starting helix build service");
    // Initialize AWS SDK
    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let s3_client = Client::new(&config);
    let user_id = std::env::var("USER_ID").unwrap_or("helix".to_string());
    // run server on specified port
    let port = std::env::var("PORT").unwrap_or("8080".to_string());
    let instance_id = std::env::var("INSTANCE_ID").unwrap_or("helix".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
    let listener = TcpListener::bind(&addr).await.map_err(|e| {
        eprintln!("Failed to bind to address {}: {}", addr, e);
        AdminError::AdminConnectionError("Failed to bind to address".to_string(), e)
    })?;

    println!("Server listening on {}", addr);

    loop {
        match listener.accept().await {
            Ok((mut conn, addr)) => {
                println!("New connection from {}", addr);
                let s3_client_clone = s3_client.clone();
                let user_id_clone = user_id.clone();
                let instance_id_clone = instance_id.clone();
                tokio::spawn(async move {
                    let response = match handle_deploy_request(&s3_client_clone, &user_id_clone, &instance_id_clone).await {
                        Ok(msg) => {
                            println!("Deployment successful: {}", msg);
                            DeployResponse::success(msg)
                        }
                        Err(e) => {
                            eprintln!("Deployment failed: {}", e);
                            DeployResponse::error("Deployment failed".to_string(), e.to_string())
                        }
                    };

                    let response_json = sonic_rs::to_string(&response).unwrap_or_else(|_| {
                        r#"{"success":false,"message":"Failed to serialize response","error":"JSON serialization error"}"#.to_string()
                    });

                    let http_response = if response.success {
                        format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                            response_json.len(),
                            response_json
                        )
                    } else {
                        format!(
                            "HTTP/1.1 500 Internal Server Error\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                            response_json.len(),
                            response_json
                        )
                    };

                    if let Err(e) = conn.write_all(http_response.as_bytes()).await {
                        eprintln!("Failed to send response: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {:?}", e);
            }
        }
    }
}

async fn handle_deploy_request(s3_client: &Client, user_id: &str, instance_id: &str) -> Result<String, AdminError> {
    // Step 1: Backup old binary
    println!("Step 1: Backing up old binary");
    let backup_result = Command::new("mv")
        .arg("/root/.helix/bin/helix-container")
        .arg("/root/.helix/bin/helix-container_old")
        .output()
        .map_err(|e| AdminError::CommandError("Failed to backup old binary".to_string(), e))?;

    if !backup_result.status.success() {
        let error_msg = String::from_utf8_lossy(&backup_result.stderr);
        return Err(AdminError::InvalidParameter(format!(
            "Failed to backup old binary: {}",
            error_msg
        )));
    }

    // Step 2: Download new binary from S3
    println!("Step 2: Downloading new binary from S3 for user {} and instance {}", user_id, instance_id);
    let response = s3_client
        .get_object()
        .bucket("helix-user-builds")
        .key(format!("{}/{}/helix-container/latest", user_id, instance_id))
        .send()
        .await
        .map_err(|e| AdminError::S3DownloadError("Failed to download from S3".to_string(), e))?;

    let data = response
        .body
        .collect()
        .await
        .map_err(|e| AdminError::InvalidParameter(format!("Failed to read S3 response body {:?}", e)))?
        .into_bytes();

    let mut file = File::create("/root/.helix/bin/helix-container")
        .map_err(|e| AdminError::FileError("Failed to create new binary file".to_string(), e))?;
    
    file.write_all(&data)
        .map_err(|e| AdminError::FileError("Failed to write new binary".to_string(), e))?;

    // Step 3: Set permissions
    println!("Step 3: Setting permissions");
    let chmod_result = Command::new("chmod")
        .arg("+x")
        .arg("/root/.helix/bin/helix-container")
        .output()
        .map_err(|e| AdminError::CommandError("Failed to set permissions".to_string(), e))?;

    if !chmod_result.status.success() {
        let error_msg = String::from_utf8_lossy(&chmod_result.stderr);
        return Err(AdminError::InvalidParameter(format!(
            "Failed to set permissions: {}",
            error_msg
        )));
    }

    // Step 4: Restart systemd service
    println!("Step 4: Restarting helix service");
    let restart_result = Command::new("sudo")
        .arg("systemctl")
        .arg("restart")
        .arg("helix")
        .output()
        .map_err(|e| AdminError::CommandError("Failed to restart service".to_string(), e))?;

    if !restart_result.status.success() {
        let error_msg = String::from_utf8_lossy(&restart_result.stderr);
        return Err(AdminError::InvalidParameter(format!(
            "Failed to restart service: {}",
            error_msg
        )));
    }

    // Step 5: Wait a moment for service to start, then check status
    println!("Step 5: Checking service status");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    let status_result = Command::new("sudo")
        .arg("systemctl")
        .arg("is-active")
        .arg("helix")
        .output()
        .map_err(|e| AdminError::CommandError("Failed to check service status".to_string(), e))?;

    if !status_result.status.success() {
        // Service failed to start, revert to old binary
        println!("Service failed to start, reverting to old binary");
        
        let revert_result = Command::new("mv")
            .arg("/root/.helix/bin/helix-container_old")
            .arg("/root/.helix/bin/helix-container")
            .output()
            .map_err(|e| AdminError::CommandError("Failed to revert binary".to_string(), e))?;

        if !revert_result.status.success() {
            return Err(AdminError::InvalidParameter(
                "Failed to revert to old binary after service failure".to_string()
            ));
        }

        let restart_old_result = Command::new("sudo")
            .arg("systemctl")
            .arg("restart")
            .arg("helix")
            .output()
            .map_err(|e| AdminError::CommandError("Failed to restart with old binary".to_string(), e))?;

        if !restart_old_result.status.success() {
            return Err(AdminError::InvalidParameter(
                "Failed to restart service with old binary".to_string()
            ));
        }

        return Err(AdminError::InvalidParameter(
            "New binary failed to start service, reverted to old version".to_string()
        ));
    }

    // Step 6: Clean up old binary if everything is successful
    println!("Step 6: Cleaning up old binary");
    let cleanup_result = Command::new("sudo")
        .arg("rm")
        .arg("-f")
        .arg("/root/.helix/bin/helix-container_old")
        .output()
        .map_err(|e| AdminError::CommandError("Failed to cleanup old binary".to_string(), e))?;

    if !cleanup_result.status.success() {
        println!("Warning: Failed to cleanup old binary, but deployment was successful");
    }

    Ok("Deployment completed successfully. New helix-container binary is running.".to_string())
}

#[derive(Debug)]
pub enum AdminError {
    AdminConnectionError(String, std::io::Error),
    S3DownloadError(String, aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::get_object::GetObjectError>),
    CommandError(String, std::io::Error),
    FileError(String, std::io::Error),
    InvalidParameter(String),
}

impl std::fmt::Display for AdminError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdminError::AdminConnectionError(msg, err) => write!(f, "Connection error: {}: {}", msg, err),
            AdminError::S3DownloadError(msg, err) => write!(f, "S3 error: {}: {}", msg, err),
            AdminError::CommandError(msg, err) => write!(f, "Command error: {}: {}", msg, err),
            AdminError::FileError(msg, err) => write!(f, "File error: {}: {}", msg, err),
            AdminError::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
        }
    }
}

impl std::error::Error for AdminError {}
