use std::{
    fs,
    process::Command,
    env,
    path::Path,
};

fn new_project(name: &str) -> Result<(), &'static str> {
    let project_dir: &Path = Path::new(name);
    if project_dir.exists() {
        return Ok(())
    }
    
    if fs::create_dir(project_dir).is_err() {
        return Err("Failed to create project directory.")
    };
    
    if fs::create_dir(project_dir.join("src")).is_err() {
        return Err("Failed to create project source directory.")
    };
    
    if fs::create_dir(project_dir.join("target")).is_err() {
        return Err("Failed to create project target directory.")
    };

    let hello_world_program = concat!(
            "#include <iostream>\n",
            "\n",
            "int main() {\n",
            "    std::cout << \"Hello World!\\n\";\n",
            "    return 0;\n",
            "};"
        );

    if fs::write(
        project_dir.join("src").join("main.cpp"), hello_world_program
    ).is_err() {
        return Err("Failed to create project main.cpp file.")
    };

    Ok(())
}

fn build_project() -> Result<(), &'static str> {
    let project_dir = match env::current_dir() {
        Ok(path) => path,
        Err(_) => return Err("Couldn't get project directory.")
    };
    let project_src = project_dir.join("src");
    let project_target = project_dir.join("target");

    if ! (project_src.exists() && project_target.exists()) {
        return Err("No src and target directories.")
    };

    let src_files = match fs::read_dir(project_src) {
        Ok(files) => {
            files.filter_map(|f| f.ok())
                .map(|f| f.path())
                .filter(|f| f.extension().unwrap() == "cpp")
        },
        Err(_) => return Err("Couldn't read source directory.")
    };
    
    let project_name = match project_dir.file_name() {
        Some(name) => name,
        None => return Err("Couldn't get project name.")
    };

    let output_file = project_target.join(project_name);

    let output_args = ["-o", output_file.to_str().unwrap()];

    match Command::new("g++").args(output_args).args(src_files).spawn() {
        Ok(mut compiler) => {
            if compiler.wait().is_err() {
                return Err("Compilation failed.")
            };
        },
        Err(_) => return Err("Couldn't start compiler.")
    };
    
    Ok(())
}

fn run_project() -> Result<(), &'static str> {
    let project_dir = match env::current_dir() {
        Ok(path) => path,
        Err(_) => return Err("Couldn't access project directory.")
    };
    
    let project_target = project_dir.join("target");

    let project_name = match project_dir.file_name() {
        Some(name) => name,
        None => return Err("Couldn't get project name.")
    };

    let project_excecutable = project_target.join(project_name);
    
    let _ = match Command::new(project_excecutable).spawn() {
        Ok(mut child) => child.wait(),
        Err(_) => return Err("Couldn't excecute project file.")
    };

    Ok(())
}

fn main() {
    let mut args = env::args();
    args.next();  // Discard excecutable name

    let command = args.next().expect("No command provided.");

    if command == "new" {
        let project_name = args.next().expect("No project name provided.");
        println!("Creating new project {project_name}...");
        match new_project(&project_name) {
            Ok(_) => println!("Project {project_name} created succesfully!"),
            Err(err) => eprintln!("{err}")
        };
    } else if command == "build" {
        println!("Building project...");
        match build_project() {
            Ok(_) => println!("Project built succesfully!"),
            Err(err) => eprintln!("{err}")
        };
    } else if command == "run" {
        println!("Building project...");
        match build_project() {
            Ok(_) => println!("Project built succesfully!"),
            Err(err) => {
                eprintln!("{err}"); 
                return
            },
        };
        println!("Running project...");
        if let Err(err) = run_project() {
            eprintln!("{err}");
        };
    } else {
        eprintln!("Invalid command provided.");
    };
}
