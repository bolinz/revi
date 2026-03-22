use anyhow::Result;
use clap::Parser;
use revi::{
    bootstrap,
    catalog::format_template_list,
    cli::{Cli, Commands, TemplateCommands},
    doctor,
    scaffold::scaffold,
    wizard::resolve_config,
};

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init(args) => {
            let config = resolve_config(&args)?;
            let project_dir = scaffold(&config)?;
            let report = bootstrap::run(&project_dir, &config)?;
            println!("Scaffolded {}", project_dir.display());
            println!(
                "git_initialized={} initial_commit_created={} github_repo_created={} remote_pushed={}",
                report.git_initialized,
                report.initial_commit_created,
                report.github_repo_created,
                report.remote_pushed
            );
        }
        Commands::Templates(args) => match args.command {
            TemplateCommands::List => println!("{}", format_template_list()?),
        },
        Commands::Doctor => println!("{}", doctor::run()),
    }
    Ok(())
}
