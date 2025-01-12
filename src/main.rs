use anyhow::{self, Context};
use std::env;

mod cli;
use cli::{Cli, Commands, Parser};

mod build;
mod new;
mod run;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { path } => {
            println!("Creating new project {path}...");
            new::new_project(&env::current_dir().unwrap(), &path)
                .with_context(|| format!("Failed to create project {}", &path))?;
            println!("Project {path} created succesfully!");
        }
        Commands::Build => {
            println!("Building project...");
            build::build_project().with_context(|| "Failed to build project.")?;
            println!("Project built succesfully!");
        }
        Commands::Run => {
            println!("Building project...");
            build::build_project()
                .with_context(|| "Failed to build project before attempting to run it.")?;
            println!("Project built succesfully!");
            println!("Running project...");
            run::run_project().with_context(|| "Failed to run project")?;
        }
    };

    Ok(())
}
