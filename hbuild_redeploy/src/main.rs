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

    let user_id = std::env::var("USER_ID").expect("USER_ID is not set");
    let cluster_id = std::env::var("CLUSTER_ID").expect("CLUSTER_ID is not set");
    // run server on specified port
    let port = std::env::var("PORT").unwrap_or("6900".to_string());
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
                    Command::new("sudo")
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
