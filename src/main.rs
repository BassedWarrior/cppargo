use std::{env, fs, path::Path, process::Command};

use clap::{Parser, Subcommand};

use anyhow::{self, Context};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Creates a new cppargo project at <path>
    ///
    /// This command will create a new cppargo project in the given directory.
    /// This includes a sample source file "Hello World!" program
    /// (`src/main.cpp`).
    New { path: String },
    /// Compile a local project.
    ///
    /// Iterates over the `src` directory in order to find all `.cpp` source files
    /// and gives it to the `g++` compiler to store it in the `target` directory
    /// with the project name.
    Build,
    /// Run a binary of the local project.
    ///
    /// Compile the project, and then excecute the compiled binary stored in
    /// the `target` directory.
    Run,
}

fn new_project(name: &str) -> anyhow::Result<()> {
    let project_dir: &Path = Path::new(name);

    fs::create_dir(project_dir).with_context(|| {
        format!(
            "Failed to create project directory {}.",
            project_dir.display()
        )
    })?;

    fs::create_dir(project_dir.join("src"))
        .with_context(|| "Failed to create project source directory.")?;

    fs::create_dir(project_dir.join("target"))
        .with_context(|| "Failed to create project target directory.")?;

    let hello_world_program = concat!(
        "#include <iostream>\n",
        "\n",
        "int main() {\n",
        "    std::cout << \"Hello World!\\n\";\n",
        "    return 0;\n",
        "};"
    );

    fs::write(
        project_dir.join("src").join("main.cpp"),
        hello_world_program,
    )
    .with_context(|| "Failed to create project `main.cpp` file.")?;

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

    let src_files = fs::read_dir(&project_src)
        .with_context(|| format!("Couldn't read source directory {}.", &project_src.display()))?
        .filter_map(|f| f.ok())
        .map(|f| f.path())
        .filter(|f| f.extension().unwrap() == "cpp");

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

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { path } => {
            println!("Creating new project {path}...");
            new_project(&path).with_context(|| format!("Failed to create project {}", &path))?;
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
