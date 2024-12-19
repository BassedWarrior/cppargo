use std::{env, fs, path::Path, process::Command};

use anyhow::{self, Context};

/// Creates a new project file structure
///
/// Creates a directory with the project `name`, and inside it, two other
/// subdirectories.
///
/// - A `src` directory with a `main.rs` sample "Hello World!" C++ program.
/// - A `target` directory in which the compiled binaries are stored and run.
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

/// Compiles all source files and stores binary
///
/// Iterates over the `src` directory in order to find all `.cpp` source files
/// and gives it to the `g++` compiler to store it in the `target` directory
/// with the project name.
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

/// Compile and run the project.
///
/// Call the `build_project()` function to compile the project, and then
/// excecutes the compiled binary stored in the `target` directory.
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
    let mut args = env::args();
    args.next(); // Discard excecutable name

    let command = args.next().with_context(|| "No command provided.")?;

    if command == "new" {
        let project_name = args.next().expect("No project name provided.");
        println!("Creating new project {project_name}...");
        new_project(&project_name)
            .with_context(|| format!("Failed to create project {}", &project_name))?;
        println!("Project {project_name} created succesfully!");
    } else if command == "build" {
        println!("Building project...");
        build_project().with_context(|| "Failed to build project.")?;
        println!("Project built succesfully!");
    } else if command == "run" {
        println!("Building project...");
        build_project().with_context(|| "Failed to build project before attempting to run it.")?;
        println!("Project built succesfully!");
        println!("Running project...");
        run_project().with_context(|| "Failed to run project")?;
    } else {
        anyhow::bail!("Invalid command provided.");
    };

    Ok(())
}
