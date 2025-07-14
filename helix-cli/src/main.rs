use crate::{
    args::{CommandType, HelixCLI, OutputLanguage},
    instance_manager::InstanceManager,
    types::*,
    utils::*,
};
use clap::Parser; use helixdb::{helix_engine::graph_core::config::Config, utils::styled_string::StyledString}; use sonic_rs::json;
use spinners::{Spinner, Spinners};
use std::{
    fmt::Write,
    fs::{self, OpenOptions, read_to_string},
    io::{Write as iWrite},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
pub mod args;
mod instance_manager;
mod types;
mod utils;

#[tokio::main]
async fn main() {
    check_helix_version().await;

    let args = HelixCLI::parse();
    match args.command {
        CommandType::Deploy(command) => {}

        CommandType::Update => {
            match check_helix_installation() {
                Some(_) => {} // container path
                None => {
                    println!(
                        "{}",
                        "Helix is not installed. Please run `helix install` first."
                        .red()
                        .bold()
                    );
                    return;
                }
            };

            let repo_path = {
                let home_dir = match dirs::home_dir() {
                    Some(dir) => dir,
                    None => {
                        println!("{}", "Could not determine home directory".red().bold());
                        return;
                    }
                };
                home_dir.join(".helix/repo/helix-db/helixdb")
            };

            if !check_cargo_version() {
                match Command::new("rustup").arg("update").output() {
                    Ok(_) => println!("{}", "Updating cargo!".green().bold()),
                    Err(e) => println!("{} {}", "Error updating cargo!", e),
                }
            } else {
                println!("{}", "cargo up-to-date!".green().bold());
            }

            let local_helix_version = get_cli_version();
            let remote_helix_version = get_remote_helix_version().await.unwrap();
            println!(
                "local helix version: {}, remote helix version: {}",
                local_helix_version, remote_helix_version
            );

            if local_helix_version < remote_helix_version {
                let mut runner = Command::new("git");
                runner.arg("reset");
                runner.arg("--hard");
                runner.current_dir(&repo_path);
                match runner.output() {
                    Ok(_) => {}
                    Err(e) => {
                        println!(
                            "{} {}",
                            "Error while reseting installed helix-db version:"
                            .red()
                            .bold(),
                            e
                        );
                        return;
                    }
                }

                let mut runner = Command::new("git");
                runner.arg("pull");
                runner.current_dir(&repo_path);
                match runner.output() {
                    Ok(_) => println!(
                        "{}",
                        "New helix-db version successfully pulled!".green().bold()
                    ),
                    Err(e) => {
                        println!(
                            "{} {}",
                            "Error while pulling new helix-db version:".red().bold(),
                            e
                        );
                        return;
                    }
                }

                match get_n_helix_cli() {
                    Ok(_) => println!(
                        "{}",
                        "New helix-cli version successfully installed!"
                        .green()
                        .bold()
                    ),
                    Err(e) => {
                        println!(
                            "{} {}",
                            "Error while installing new helix-cli version:".red().bold(),
                            e
                        );
                        return;
                    }
                }
            } else {
                println!("{}", "HelixDB is up to date!".green().bold());
            }
        }

        CommandType::Compile(command) => {}
        CommandType::Install(command) => {}
        CommandType::Init(command) => {}

        CommandType::Status => {
            let instance_manager = InstanceManager::new().unwrap();
            match instance_manager.list_instances() {
                Ok(instances) => {
                    if instances.is_empty() {
                        println!("{}", "No running Helix instances".yellow.bold());
                        return;
                    }
                    for instance in instances {
                        print_instance(&instance);
                        println!();
                    }
                }
                Err(e) => println!("{} {}", "Failed to list instances:".red().bold(), e)
            }
        }

        CommandType::Stop(command) => {}
        CommandType::Save(command) => {}
        CommandType::Delete(command) => {}

        CommandType::Version => {
            match check_helix_installation() {
                Ok(_) => {}
                Err(_) => {
                    println!(
                        "{}",
                        "Helix is not installed. Please run `helix install` first."
                        .red()
                        .bold()
                    );
                    return;
                }
            };

            let repo_path = {
                let home_dir = match dirs::home_dir() {
                    Some(dir) => dir,
                    None => {
                        println!("{}", "Could not determine home directory".red().bold());
                        return;
                    }
                };
                home_dir.join(".helix/repo/helix-db/helixdb")
            };

            let local_cli_version = get_cli_version();
            let local_db_version = get_db_version();
            println!(
                "{} {}, {} {}",
                "helix-cli version:",
                local_cli_version,
                "helix-db version:",
                local_db_version
            );
        }

        // TODO: just local for now
        CommandType::Visualize(command) => {
            let instance_manager = InstanceManager::new().unwrap();
            let iid = &command.instance;

            match instance_manager.get_instance(iid) {
                Ok(Some(instance)) => {
                    println!("{}", "Helix instance found!".green().bold());
                    let port = instance.port;
                    let url = format!("http://localhost:{}/get/graphvis", port);

                    if webbrowser::open(&url).is_ok() {
                    } else {
                        println!(
                            "{} {}",
                            "Failed to open graph visualizer for instance".red().bold(),
                            iid.red().bold()
                        );
                        return;
                    }
                }
                Ok(None) => {
                    println!(
                        "{} {}",
                        "No Helix instance found with id".red().bold(),
                        iid.red().bold()
                    );
                    return;
                }
                Err(e) => {
                    println!("{} {}", "Error:".red().bold(), e);
                    return;
                }
            };
        }

        CommandType::Login => {
            let home_dir = std::env::var("HOME").unwrap_or("~/".to_string());
            let config_path = &format!("{}/.helix", home_dir);
            let config_path = Path::new(config_path);
            if !config_path.exists() {
                fs::create_dir_all(config_path).unwrap();
            }

            let cred_path = config_path.join("credentials");

            if let Ok(contents) = read_to_string(&cred_path)
                && let Some(_key) = parse_credentials(&contents)
            {
                println!(
                    "You have an existing key which may be valid, only continue if it doesn't work or you want to switch accounts. (Key checking is WIP)"
                );
            }

            let key = github_login().await.unwrap();
            println!("{}", "Successfully logged in!".green().bold());

            let mut cred_file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(true)
                .open(cred_path)
                .unwrap();

            if let Err(e) = cred_file.write_all(&format!("helix_user_key={key}").into_bytes()) {
                println!(
                    "Got error when writing key: {}\nYou're key is: {}",
                    e.to_string().red(),
                    key
                );
            }
        }

        CommandType::Logout => {
            let home_dir = std::env::var("HOME").unwrap_or("~/".to_string());
            let config_path = &format!("{}/.helix", home_dir);
            let config_path = Path::new(config_path);
            if !config_path.exists() {
                fs::create_dir_all(config_path).unwrap();
            }

            let cred_path = config_path.join("credentials");
            if cred_path.exists() {
                fs::remove_file(cred_path).unwrap()
            }
        }
    }
}

