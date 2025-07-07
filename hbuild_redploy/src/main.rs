use anyhow::Result;
use aws_config::BehaviorVersion;
use aws_sdk_s3::Client;
use sonic_rs::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::{net::SocketAddr, process::Command};
use tokio::io::AsyncWriteExt;
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
    // Initialize AWS SDK with explicit region configuration
    let bucket_region = std::env::var("S3_BUCKET_REGION").unwrap_or("us-west-1".to_string());
    println!("Using S3 bucket region: {}", bucket_region);

    let config = aws_config::load_defaults(BehaviorVersion::latest())
        .await
        .to_builder()
        .region(aws_config::Region::new(bucket_region.clone()))
        .build();
    let s3_client = Client::new(&config);

    println!("AWS region configured: {:?}", config.region());

    let user_id = std::env::var("USER_ID").unwrap_or("helix".to_string());
    let cluster_id = std::env::var("CLUSTER_ID").unwrap_or("helix".to_string());
    // run server on specified port
    let port = std::env::var("PORT").unwrap_or("8080".to_string());
    let instance_id = std::env::var("EC2_INSTANCE_ID").unwrap_or("helix".to_string());
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
                let cluster_id_clone = cluster_id.clone();
                tokio::spawn(async move {
                    // rename old binary
                    Command::new("mv")
                        .arg("helix")
                        .arg("helix_old")
                        .spawn()
                        .unwrap();

                    // pull binary from s3
                    let response = s3_client_clone
                        .get_object()
                        .bucket("helix-build")
                        .key(format!(
                            "{}/{}/helix/latest",
                            user_id_clone, cluster_id_clone
                        ))
                        .send()
                        .await
                        .unwrap();

                    // create binary file or overwrite if it exists
                    let mut file = File::create("helix").unwrap();
                    let body = match response.body.collect().await {
                        Ok(body) => body.to_vec(),
                        Err(e) => {
                            eprintln!("Error collecting body: {:?}", e);
                            return;
                        }
                    };
                    file.write_all(&body).unwrap();

                    // set permissions
                    Command::new("sudo ")
                        .arg("chmod")
                        .arg("+x")
                        .arg("helix")
                        .spawn()
                        .unwrap();

                    // restart systemd service
                    Command::new("sudo")
                        .arg("systemctl")
                        .arg("restart")
                        .arg("helix")
                        .spawn()
                        .unwrap();

                    // check if service is running
                    let output = Command::new("sudo")
                        .arg("systemctl")
                        .arg("status")
                        .arg("helix")
                        .output()
                        .unwrap();

                    // if not revert
                    if !output.status.success() {
                        Command::new("mv")
                            .arg("helix_old")
                            .arg("helix")
                            .spawn()
                            .unwrap();

                        Command::new("sudo")
                            .arg("systemctl")
                            .arg("restart")
                            .arg("helix")
                            .spawn()
                            .unwrap();

                        return;
                    } else {
                        // delete old binary
                        Command::new("rm").arg("helix_old").spawn().unwrap();
                    }
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {:?}", e);
            }
        }
    }
}

async fn handle_deploy_request(
    s3_client: &Client,
    user_id: &str,
    instance_id: &str,
) -> Result<String, AdminError> {
    // Step 0: Debug AWS configuration and connectivity
    println!("Step 0: Verifying AWS configuration and S3 connectivity");
    println!("User ID: {}, Instance ID: {}", user_id, instance_id);

    // Try to list objects in the bucket to verify connectivity
    let list_response = s3_client
        .list_objects_v2()
        .bucket("helix-user-builds")
        .prefix(&format!("{}/{}/helix-container/", user_id, instance_id))
        .send()
        .await;

    match list_response {
        Ok(list_result) => {
            let contents = list_result.contents();
            println!(
                "Found {} objects with prefix {}/{}/helix-container/",
                contents.len(),
                user_id,
                instance_id
            );
            for obj in contents {
                if let Some(key) = obj.key() {
                    println!("  - {}", key);
                    if let Some(size) = obj.size() {
                        println!("    Size: {} bytes", size);
                    }
                    if let Some(modified) = obj.last_modified() {
                        println!("    Last modified: {:?}", modified);
                    }
                }
            }
            if contents.is_empty() {
                println!(
                    "No objects found with prefix {}/{}/helix-container/",
                    user_id, instance_id
                );
            }
        }
        Err(e) => {
            eprintln!("Failed to list objects in bucket: {:?}", e);
            return Err(AdminError::InvalidParameter(format!(
                "Failed to verify S3 connectivity: {:?}",
                e
            )));
        }
    }

    // Step 2: Download new binary from S3
    println!(
        "Step 2: Downloading new binary from S3 for user {} and instance {}",
        user_id, instance_id
    );
    let s3_key = format!("{}/{}/helix-container/latest", user_id, instance_id);
    println!(
        "Attempting to download from bucket: helix-user-builds, key: {}",
        s3_key
    );

    let response = s3_client
        .get_object()
        .bucket("helix-user-builds")
        .key(&s3_key)
        .send()
        .await
        .map_err(|e| {
            eprintln!("S3 GetObject error details: {:?}", e);
            // Print more specific error information
            match &e {
                aws_sdk_s3::error::SdkError::ServiceError(service_err) => {
                    eprintln!("Service error: {:?}", service_err.err());
                    eprintln!("HTTP status: {:?}", service_err.raw().status());
                }
                aws_sdk_s3::error::SdkError::ConstructionFailure(construction_err) => {
                    eprintln!("Construction failure: {:?}", construction_err);
                }
                aws_sdk_s3::error::SdkError::TimeoutError(timeout_err) => {
                    eprintln!("Timeout error: {:?}", timeout_err);
                }
                aws_sdk_s3::error::SdkError::DispatchFailure(dispatch_err) => {
                    eprintln!("Dispatch failure: {:?}", dispatch_err);
                }
                _ => {
                    eprintln!("Other S3 error: {:?}", e);
                }
            }
            AdminError::S3DownloadError(
                format!(
                    "Failed to download from S3 (bucket: helix-user-builds, key: {})",
                    s3_key
                ),
                e,
            )
        })?;

    let data = response
        .body
        .collect()
        .await
        .map_err(|e| {
            AdminError::InvalidParameter(format!("Failed to read S3 response body {:?}", e))
        })?
        .into_bytes();

    let mut file = File::create("/root/.helix/bin/helix-container.new")
        .map_err(|e| AdminError::FileError("Failed to create new binary file".to_string(), e))?;

    file.write_all(&data)
        .map_err(|e| AdminError::FileError("Failed to write new binary".to_string(), e))?;

    // Ensure data is flushed to disk before proceeding
    file.flush()
        .map_err(|e| AdminError::FileError("Failed to flush binary file".to_string(), e))?;

    file.sync_all()
        .map_err(|e| AdminError::FileError("Failed to sync binary file to disk".to_string(), e))?;

    // Explicitly drop the file handle to ensure it's closed
    drop(file);

    // Step 3: Set permissions
    println!("Step 3: Setting permissions");
    let chmod_result = Command::new("chmod")
        .arg("+x")
        .arg("/root/.helix/bin/helix-container.new")
        .output()
        .map_err(|e| AdminError::CommandError("Failed to set permissions".to_string(), e))?;

    if !chmod_result.status.success() {
        let error_msg = String::from_utf8_lossy(&chmod_result.stderr);
        return Err(AdminError::InvalidParameter(format!(
            "Failed to set permissions: {}",
            error_msg
        )));
    }

    // rename the new binary to the old binary
    let rename_result = Command::new("mv")
        .arg("/root/.helix/bin/helix-container.new")
        .arg("/root/.helix/bin/helix-container")
        .output()
        .map_err(|e| AdminError::CommandError("Failed to rename binary".to_string(), e))?;

    if !rename_result.status.success() {
        let error_msg = String::from_utf8_lossy(&rename_result.stderr);
        return Err(AdminError::InvalidParameter(format!(
            "Failed to rename binary: {}",
            error_msg
        )));
    }

    // Step 4: restart systemd service
    println!("Step 4: Starting helix service");
    let start_result = Command::new("sudo")
        .arg("systemctl")
        .arg("restart")
        .arg("helix")
        .output()
        .map_err(|e| AdminError::CommandError("Failed to start service".to_string(), e))?;

    if !start_result.status.success() {
        let error_msg = String::from_utf8_lossy(&start_result.stderr);
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
                "Failed to revert to old binary after service failure".to_string(),
            ));
        }

        let restart_old_result = Command::new("sudo")
            .arg("systemctl")
            .arg("restart")
            .arg("helix")
            .output()
            .map_err(|e| {
                AdminError::CommandError("Failed to restart with old binary".to_string(), e)
            })?;

        if !restart_old_result.status.success() {
            return Err(AdminError::InvalidParameter(
                "Failed to restart service with old binary".to_string(),
            ));
        }

        return Err(AdminError::InvalidParameter(
            "New binary failed to start service, reverted to old version".to_string(),
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
    S3DownloadError(
        String,
        aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::get_object::GetObjectError>,
    ),
    CommandError(String, std::io::Error),
    FileError(String, std::io::Error),
    InvalidParameter(String),
}

impl std::fmt::Display for AdminError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdminError::AdminConnectionError(msg, err) => {
                write!(f, "Connection error: {}: {}", msg, err)
            }
            AdminError::S3DownloadError(msg, err) => write!(f, "S3 error: {}: {}", msg, err),
            AdminError::CommandError(msg, err) => write!(f, "Command error: {}: {}", msg, err),
            AdminError::FileError(msg, err) => write!(f, "File error: {}: {}", msg, err),
            AdminError::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
        }
    }
}

impl std::error::Error for AdminError {}
