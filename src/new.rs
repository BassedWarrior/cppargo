use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::Context;

pub fn new_project(project_root: &Path, name: &str) -> anyhow::Result<()> {
    let project_dir: PathBuf = create_project_fs(project_root, name)
        .with_context(|| "Failed to create project file structure")?;

    create_hello_world(&project_dir.join("src"))
        .with_context(|| "Failed to create simple 'Hello World!' program.")?;

    Ok(())
}

fn create_project_fs(project_root: &Path, name: &str) -> anyhow::Result<PathBuf> {
    // Create project root directory.
    let project_dir = project_root.join(name);
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

fn create_hello_world(project_src: &Path) -> anyhow::Result<()> {
    let hello_world_program = concat!(
        "#include <iostream>\n",
        "\n",
        "int main() {\n",
        "    std::cout << \"Hello World!\\n\";\n",
        "\n",
        "    return 0;\n",
        "}\n"
    );

    fs::write(project_src.join("main.cpp"), hello_world_program).with_context(|| {
        format!(
            "Failed to create project `{}` file.",
            project_src.join("main.cpp").display()
        )
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;
    use assert_fs::prelude::*;

    #[test]
    fn proper_project_fs_creation() -> anyhow::Result<()> {
        let project_root = assert_fs::TempDir::new().unwrap();
        let project_name = "test";
        create_project_fs(&project_root, project_name)
            .with_context(|| "Failed to create project file structure!")?;
        let project_dir = project_root.child(project_name);
        let project_src = project_dir.child("src");
        let project_target = project_dir.child("target");
        anyhow::ensure!(
            project_dir.exists() && project_src.exists() && project_target.exists(),
            "Couldn't find all the proper directories!"
        );
        Ok(())
    }

    #[test]
    fn proper_hello_world_creation() -> anyhow::Result<()> {
        let project_root = assert_fs::TempDir::new().unwrap();
        let project_src = project_root.child("src");
        project_src.create_dir_all().unwrap();

        create_hello_world(project_src.path()).unwrap();
        let hello_world = project_src.child("main.cpp");

        anyhow::ensure!(
            hello_world.exists(),
            "Failed to create the hello world base program!"
        );

        Ok(())
    }

    #[test]
    fn proper_hello_world_content() -> anyhow::Result<()> {
        let project_root = assert_fs::TempDir::new().unwrap();
        let project_src = project_root.child("src");
        project_src.create_dir_all().unwrap();

        create_hello_world(project_src.path()).unwrap();
        let hello_world = project_src.child("main.cpp");

        hello_world.assert(concat!(
            "#include <iostream>\n",
            "\n",
            "int main() {\n",
            "    std::cout << \"Hello World!\\n\";\n",
            "\n",
            "    return 0;\n",
            "}\n"
        ));

        Ok(())
    }

    #[test]
    fn proper_new_project() -> anyhow::Result<()> {
        let project_root = assert_fs::TempDir::new().unwrap();
        let project_name = "foo";
        new_project(&project_root, project_name)
            .with_context(|| "Failed to create new project!")?;

        let project_dir = project_root.child(project_name);
        let project_src = project_dir.child("src");
        let project_target = project_dir.child("target");
        let project_hello_world = project_src.child("main.cpp");

        anyhow::ensure!(
            project_dir.exists()
                && project_src.exists()
                && project_target.exists()
                && project_hello_world.exists(),
            "Failed to properly create all the elements of a new project!"
        );

        Ok(())
    }
}
