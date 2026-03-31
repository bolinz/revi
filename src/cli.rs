use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "revi",
    version,
    about = "AI-native bootstrap and release tool for code projects"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Init(InitArgs),
    Templates(TemplatesArgs),
    Doctor,
}

#[derive(Debug, Args)]
pub struct InitArgs {
    #[arg(long)]
    pub config: Option<PathBuf>,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub path: Option<PathBuf>,
    #[arg(long, value_enum)]
    pub template: Option<TemplateChoice>,
    #[arg(long)]
    pub non_interactive: bool,
}

#[derive(Debug, Args)]
pub struct TemplatesArgs {
    #[command(subcommand)]
    pub command: TemplateCommands,
}

#[derive(Debug, Subcommand)]
pub enum TemplateCommands {
    List,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum TemplateChoice {
    GenericProject,
    PythonService,
    NodeWeb,
    DesktopTauri,
    Godot3DGame,
}
