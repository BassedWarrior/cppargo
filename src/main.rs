use std::{env, process::Command};

use anyhow::{self, Context};

mod cli;
use cli::{Cli, Commands, Parser};

mod new;
mod build;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { path } => {
            println!("Creating new project {path}...");
            new::new_project(&path)
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
            run_project().with_context(|| "Failed to run project")?;
        }
    };

    Ok(())
}

fn run_project() -> anyhow::Result<()> {
    let project_dir = env::current_dir().with_context(|| "Couldn't access project directory.")?;

    let project_target = project_dir.join("target");

    let project_name = match project_dir.file_name() {
        Some(name) => name,
        None => anyhow::bail!("Couldn't get project name."),
    };

    let project_excecutable = project_target.join(project_name);

    Command::new(&project_excecutable)
        .spawn()
        .with_context(|| {
            format!(
                "Couldn't excecute project file {}",
                &project_excecutable.display()
            )
        })?
        .wait()
        .with_context(|| {
            format!(
                "Project file {} wasn't running",
                &project_excecutable.display()
            )
        })?;

    Ok(())
}
