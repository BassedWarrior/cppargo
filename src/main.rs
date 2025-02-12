#![warn(clippy::pedantic)]

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
            println!("Creating new project {}...", path.display());
            new::main(&path)
                .with_context(|| format!("Failed to create project {}", &path.display()))?;
            println!("Project {} created successfully!", path.display());
        }
        Commands::Build => {
            println!("Building project...");
            build::main(&env::current_dir()?).with_context(|| "Failed to build project.")?;
            println!("Project built successfully!");
        }
        Commands::Run => {
            println!("Building project...");
            build::main(&env::current_dir()?)
                .with_context(|| "Failed to build project before attempting to run it.")?;
            println!("Project built successfully!");
            println!("Running project...");
            run::main(&env::current_dir()?).with_context(|| "Failed to run project")?;
        }
    };

    Ok(())
}
