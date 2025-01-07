use std::{env, fs, path::PathBuf, process::Command};

use anyhow::{self, Context};

mod cli;
use cli::{Cli, Commands, Parser};

mod new;

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
            build_project().with_context(|| "Failed to build project.")?;
            println!("Project built succesfully!");
        }
        Commands::Run => {
            println!("Building project...");
            build_project()
                .with_context(|| "Failed to build project before attempting to run it.")?;
            println!("Project built succesfully!");
            println!("Running project...");
            run_project().with_context(|| "Failed to run project")?;
        }
    };

    Ok(())
}

fn build_project() -> anyhow::Result<()> {
    let project_dir = env::current_dir().with_context(|| "Couldn't get project directory.")?;
    let project_src = project_dir.join("src");
    let project_target = project_dir.join("target");

    anyhow::ensure!(
        project_src.exists() && project_target.exists(),
        format!(
            "No src {} and target {} directories.",
            project_src.display(),
            project_target.display()
        )
    );

    let src_files: Vec<PathBuf> = fs::read_dir(&project_src)
        .with_context(|| format!("Couldn't read source directory {}.", &project_src.display()))?
        .filter_map(|f| f.ok())
        .map(|f| f.path())
        .filter(|f| f.extension().unwrap() == "cpp")
        .collect();

    anyhow::ensure!(
        !src_files.is_empty(),
        format!(
            "No source `.cpp` files to compile found in \"{}\" directory.",
            &project_src.display()
        )
    );

    let project_name = match project_dir.file_name() {
        Some(name) => name,
        None => anyhow::bail!(format!(
            "Couldn't get project name from {}.",
            project_dir.display()
        )),
    };

    let output_file = project_target.join(project_name);

    let output_args = [
        "-o",
        output_file
            .to_str()
            .expect("Should be able to convert output_file to string"),
    ];

    let mut compiler = Command::new("g++");
    compiler.args(output_args).args(src_files);
    println!("Running compiler...\n{:?}", &compiler);
    let compiler_status = compiler
        .status()
        .with_context(|| format!("Couldn't start compiler: {:?}", compiler))?;

    anyhow::ensure!(compiler_status.success(), "Compilation failed!");

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
