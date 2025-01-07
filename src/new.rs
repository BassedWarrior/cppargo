use std::{fs, path::PathBuf};

use crate::Context;

pub fn new_project(name: &str) -> anyhow::Result<()> {
    let project_dir: PathBuf =
        create_project_fs(name).with_context(|| "Failed to create project file structure")?;

    create_hello_world(project_dir.join("src"))
        .with_context(|| "Failed to create simple 'Hello World!' program.")?;

    Ok(())
}

fn create_project_fs(name: &str) -> anyhow::Result<PathBuf> {
    // Create project root directory.
    let project_dir = PathBuf::from(name);
    fs::create_dir(&project_dir).with_context(|| {
        format!(
            "Failed to create project directory {}.",
            project_dir.display()
        )
    })?;

    // Create `source` directory where all `.cpp` files should be.
    fs::create_dir(project_dir.join("src")).with_context(|| {
        format!(
            "Failed to create project source directory {}.",
            project_dir.join("src").display()
        )
    })?;

    // Create `target` directory where all binary files should be.
    fs::create_dir(project_dir.join("target")).with_context(|| {
        format!(
            "Failed to create project target directory {}.",
            project_dir.join("target").display()
        )
    })?;

    Ok(project_dir)
}

fn create_hello_world(project_src: PathBuf) -> anyhow::Result<()> {
    let hello_world_program = concat!(
        "#include <iostream>\n",
        "\n",
        "int main() {\n",
        "    std::cout << \"Hello World!\\n\";\n",
        "    return 0;\n",
        "};"
    );

    fs::write(project_src.join("main.cpp"), hello_world_program).with_context(|| {
        format!(
            "Failed to create project `{}` file.",
            project_src.join("main.cpp").display()
        )
    })?;

    Ok(())
}
