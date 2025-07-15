use clap::{Args, Parser, Subcommand, ValueEnum};

pub mod version {
    pub const VERSION: &str = env!("CARGO_PKG_VERSION");
    pub const NAME: &str = "Helix CLI";
    pub const AUTHORS: &str = "Helix Team";
}

use version::{AUTHORS, NAME, VERSION};

#[derive(Debug, Subcommand)]
pub enum CommandType {
    /// Deploy a Helix project
    Deploy(DeployCommand),

    /// Update the cli and core database
    Update,

    /// Lint and Compile a Helix project
    Compile(CompileCommand),

    /// Install the Helix core database
    Install(InstallCommand),

    /// Initialise a new Helix project
    Init(InitCommand),

    /// List all Helix instances
    Status,

    /// Stop Helix instances
    Stop(StopCommand),

    /// Save an instnaces data.mdb file
    Save(SaveCommand),

    /// Delete an instance and all its data
    Delete(DeleteCommand),

    /// Get the current version of the cli and core database
    Version,

    /// Open graph visualizer in default browser
    Visualize(VisualizeCommand),

    /// Check login credentials or login with github
    Login,

    /// Remove login credentials
    Logout,
}

#[derive(Debug, Args)]
#[clap(name = "deploy", about = "Deploy a Helix project")]
pub struct DeployCommand {
    #[clap(help = "The path to the project")]
    pub path: Option<String>,

    #[clap(help = "Port to run the instance on")]
    pub port: Option<u16>,
}

#[derive(Debug, Args)]
#[clap(name = "compile", about = "Compile a Helix project")]
pub struct CompileCommand {
    #[clap(help = "The path to the project")]
    pub path: Option<String>,

    #[clap(help = "The output path")]
    pub output: Option<String>,
}

#[derive(Debug, Args)]
#[clap(name = "install", about = "Install the Helix repo")]
pub struct InstallCommand {
    #[clap(help = "Install HelixDB from the development branch (considered unstable)")]
    pub dev: bool,
}

#[derive(Debug, Args)]
#[clap(name = "init", about = "Initialise a new Helix project")]
pub struct InitCommand {
    #[clap(help = "The path to the project")]
    pub path: Option<String>,
}

#[derive(Debug, Args)]
#[clap(name = "stop", about = "Stop Helix instances")]
pub struct StopCommand {
    #[clap(help = "Stop all running instances")]
    pub all: bool,

    #[clap(help = "Instance ID to stop")]
    pub instance: Option<String>,
}

#[derive(Debug, Args)]
#[clap(name = "save", about = "Save an instances data.mdb file")]
pub struct SaveCommand {
    #[clap(help = "Instance ID to save")]
    pub instance: String,

    #[clap(help = "Where to save the file to")]
    pub output: Option<String>,
}

#[derive(Debug, Args)]
#[clap(name = "delete", about = "Delete an instance and its saved data")]
pub struct DeleteCommand {
    #[clap(help = "Instance ID to delete")]
    pub instance: String,
}

#[derive(Debug, Args)]
#[clap(name = "visualize", about = "Visualize the Helix graph")]
pub struct VisualizeCommand {
    #[clap(help = "Id of instance to visualize")]
    pub instance: String,

    #[clap(help = "Give nodes a label based on a property")]
    pub node_prop: Option<String>,
}

