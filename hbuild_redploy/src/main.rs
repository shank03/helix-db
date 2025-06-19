use anyhow::Result;
use aws_config::BehaviorVersion;
use aws_sdk_s3::Client;
use sonic_rs::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
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

#[tokio::main]
async fn main() -> Result<(), AdminError> {
    println!("Starting helix build service");
    // Initialize AWS SDK
    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let s3_client = Client::new(&config);
    let user_id = std::env::var("USER_ID").unwrap_or("helix".to_string());
    // run server on specified port
    let port = std::env::var("PORT").unwrap_or("8080".to_string());

    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
    let listener = TcpListener::bind(&addr).await.map_err(|e| {
        eprintln!("Failed to bind to address {}: {}", addr, e);
        AdminError::AdminConnectionError("Failed to bind to address".to_string(), e)
    })?;

    loop {
        match listener.accept().await {
            Ok((mut conn, addr)) => {
                println!("New connection from {}", addr);
                let s3_client_clone = s3_client.clone();
                let user_id_clone = user_id.clone();
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
                        .key(format!("{}/helix/latest", user_id_clone))
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
                    Command::new("chmod")
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
    S3Error(String, std::env::VarError),
    InvalidParameter(String),
}
