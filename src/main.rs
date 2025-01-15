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
            println!("Project {path} created successfully!");
        }
        Commands::Build => {
            println!("Building project...");
            build::build_project(&env::current_dir()?)
                .with_context(|| "Failed to build project.")?;
            println!("Project built successfully!");
        }
        Commands::Run => {
            println!("Building project...");
            build::build_project(&env::current_dir()?)
                .with_context(|| "Failed to build project before attempting to run it.")?;
            println!("Project built successfully!");
            println!("Running project...");
            run::run_project(&env::current_dir()?).with_context(|| "Failed to run project")?;
        }
    };

    Ok(())
}
