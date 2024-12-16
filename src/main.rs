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
    if project_dir.exists() {
        return Ok(());
    }

    fs::create_dir(project_dir)
        .with_context(|| format!("Failed to create project directory."))?;

    fs::create_dir(project_dir.join("src"))
        .with_context(|| format!("Failed to create project source directory."))?;

    fs::create_dir(project_dir.join("target"))
        .with_context(|| format!("Failed to create project target directory."))?;

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
    .with_context(|| format!("Failed to create project `main.cpp` file."))?;

    Ok(())
}

/// Compiles all source files and stores binary
///
/// Iterates over the `src` directory in order to find all `.cpp` source files
/// and gives it to the `g++` compiler to store it in the `target` directory
/// with the project name.
fn build_project() -> anyhow::Result<()> {
    let project_dir =
        env::current_dir().with_context(|| format!("Couldn't get project directory."))?;
    let project_src = project_dir.join("src");
    let project_target = project_dir.join("target");

    if !(project_src.exists() && project_target.exists()) {
        anyhow::bail!("No src and target directories.");
    };

    let src_files = match fs::read_dir(project_src) {
        Ok(files) => files
            .filter_map(|f| f.ok())
            .map(|f| f.path())
            .filter(|f| f.extension().unwrap() == "cpp"),
        Err(_) => anyhow::bail!("Couldn't read source directory."),
    };

    let project_name = match project_dir.file_name() {
        Some(name) => name,
        None => anyhow::bail!("Couldn't get project name."),
    };

    let output_file = project_target.join(project_name);

    let output_args = ["-o", output_file.to_str().unwrap()];

    match Command::new("g++")
        .args(output_args)
        .args(src_files)
        .spawn()
    {
        Ok(mut compiler) => {
            match compiler.wait() {
                Ok(exit_status) => {
                    if !exit_status.success() {
                        anyhow::bail!("Compilation failed");
                    }
                }
                Err(_) => anyhow::bail!("Compiler couldn't run properly."),
            };
        }
        Err(_) => anyhow::bail!("Couldn't start compiler."),
    };

    Ok(())
}

/// Compile and run the project.
///
/// Call the `build_project()` function to compile the project, and then
/// excecutes the compiled binary stored in the `target` directory.
fn run_project() -> anyhow::Result<()> {
    let project_dir = match env::current_dir() {
        Ok(path) => path,
        Err(_) => anyhow::bail!("Couldn't access project directory."),
    };

    let project_target = project_dir.join("target");

    let project_name = match project_dir.file_name() {
        Some(name) => name,
        None => anyhow::bail!("Couldn't get project name."),
    };

    let project_excecutable = project_target.join(project_name);

    let _ = match Command::new(project_excecutable).spawn() {
        Ok(mut child) => child.wait(),
        Err(_) => anyhow::bail!("Couldn't excecute project file."),
    };

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let mut args = env::args();
    args.next(); // Discard excecutable name

    let command = args.next().expect("No command provided.");

    if command == "new" {
        let project_name = args.next().expect("No project name provided.");
        println!("Creating new project {project_name}...");
        match new_project(&project_name) {
            Ok(_) => println!("Project {project_name} created succesfully!"),
            Err(err) => anyhow::bail!("{err}")
        };
    } else if command == "build" {
        println!("Building project...");
        match build_project() {
            Ok(_) => println!("Project built succesfully!"),
            Err(err) => anyhow::bail!("{err}"),
        };
    } else if command == "run" {
        println!("Building project...");
        match build_project() {
            Ok(_) => println!("Project built succesfully!"),
            Err(err) => {
                eprintln!("{err}");
                anyhow::bail!("Cannot run project");
            }
        };
        println!("Running project...");
        if let Err(err) = run_project() {
            anyhow::bail!("{err}");
        };
    } else {
        anyhow::bail!("Invalid command provided.");
    };

    Ok(())
}
