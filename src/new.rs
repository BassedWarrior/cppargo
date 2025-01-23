use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::Context;

const HELLO_WORLD_PROGRAM: &str = concat!(
    "#include <iostream>\n",
    "\n",
    "int main() {\n",
    "    std::cout << \"Hello World!\\n\";\n",
    "\n",
    "    return 0;\n",
    "}\n"
);

pub fn new_project(path: &Path) -> anyhow::Result<()> {
    let project_root: PathBuf =
        create_project_fs(path).with_context(|| "Failed to create project file structure")?;

    create_manifest(&project_root)?;

    create_hello_world(&project_root.join("src"))
        .with_context(|| "Failed to create simple 'Hello World!' program.")?;

    Ok(())
}

fn create_project_fs(project_root: &Path) -> anyhow::Result<PathBuf> {
    anyhow::ensure!(
        !project_root.exists(),
        format!("Path {} already exists!", project_root.display())
    );

    // Create project root directory.
    fs::create_dir_all(project_root).with_context(|| {
        format!(
            "Failed to create project directory at {}.",
            project_root.display()
        )
    })?;

    // Create `source` directory where all `.cpp` files should be.
    fs::create_dir(project_root.join("src")).with_context(|| {
        format!(
            "Failed to create project source directory {}.",
            project_root.join("src").display()
        )
    })?;

    Ok(project_root.to_path_buf())
}

fn create_manifest(project_root: &Path) -> anyhow::Result<()> {
    let name = match project_root.file_name() {
        Some(osstr) => match osstr.to_str() {
            Some(str) => str.to_string(),
            None => anyhow::bail!(format!("Failed to convert project name to str {osstr:?}!")),
        },
        None => anyhow::bail!(format!(
            "Failed to get project name from project root: {}!",
            project_root.display()
        )),
    };

    let mut manifest = toml_edit::DocumentMut::new();
    manifest["project"] = toml_edit::Item::Table(toml_edit::Table::new());
    manifest["project"]["name"] = toml_edit::value(name);

    let manifest_path = project_root.join("Cppargo.toml");
    fs::write(&manifest_path, manifest.to_string())
        .with_context(|| format!("Failed to create manifest at {}!", manifest_path.display()))?;

    Ok(())
}

fn create_hello_world(project_src: &Path) -> anyhow::Result<()> {
    fs::write(project_src.join("main.cpp"), HELLO_WORLD_PROGRAM).with_context(|| {
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
        let tmp_dir = assert_fs::TempDir::new()?;
        let project_root = tmp_dir.child("test");

        create_project_fs(&project_root)
            .with_context(|| "Failed to create project file structure!")?;

        let project_src = project_root.child("src");

        project_root.assert(predicates::path::is_dir());
        project_src.assert(predicates::path::is_dir());

        Ok(())
    }

    #[test]
    fn proper_create_manifest() -> anyhow::Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let project_root = tmp_dir.child("foo");
        project_root.create_dir_all()?;
        let project_manifest = project_root.child("Cppargo.toml");

        create_manifest(project_root.path())?;

        project_manifest.assert(concat!("[project]\n", "name = \"foo\"\n"));

        Ok(())
    }

    #[test]
    fn proper_nested_project_fs_creation() -> anyhow::Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let project_root = tmp_dir.child("parent/project_dir");

        create_project_fs(&project_root)
            .with_context(|| "Failed to create project file structure!")?;

        let project_src = project_root.child("src");

        project_root.assert(predicates::path::is_dir());
        project_src.assert(predicates::path::is_dir());

        Ok(())
    }

    #[test]
    fn proper_hello_world_creation() -> anyhow::Result<()> {
        let project_root = assert_fs::TempDir::new()?;
        let project_src = project_root.child("src");
        project_src.create_dir_all().unwrap();

        create_hello_world(project_src.path()).unwrap();
        let hello_world = project_src.child("main.cpp");

        hello_world.assert(HELLO_WORLD_PROGRAM);

        Ok(())
    }

    #[test]
    fn proper_new_project() -> anyhow::Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let project_root = tmp_dir.child("foo");

        new_project(&project_root).with_context(|| "Failed to create new project!")?;

        let project_manifest = project_root.child("Cppargo.toml");
        let project_src = project_root.child("src");
        let project_hello_world = project_src.child("main.cpp");

        project_root.assert(predicates::path::is_dir());
        project_manifest.assert("[project]\nname = \"foo\"\n");
        project_src.assert(predicates::path::is_dir());
        project_hello_world.assert(predicates::path::is_file());

        Ok(())
    }
}
