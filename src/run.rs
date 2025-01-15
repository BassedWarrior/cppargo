use crate::anyhow::Context;

use std::{
    path::{Path, PathBuf},
    process::Command,
};

pub fn run_project(project_dir: &Path) -> anyhow::Result<()> {
    let project_binary =
        find_project_binary(project_dir).with_context(|| "Failed to find project binary!")?;
    run_project_binary(project_binary).with_context(|| "Failed to run project binary!")?;

    Ok(())
}

fn find_project_binary(project_dir: &Path) -> anyhow::Result<PathBuf> {
    let project_target = project_dir.join("target");

    let project_name = match project_dir.file_name() {
        Some(name) => name,
        None => anyhow::bail!("Couldn't get project name."),
    };

    let project_binary = project_target.join(project_name);

    anyhow::ensure!(
        &project_binary.exists(),
        format!("Project excecutable {} not found", project_binary.display())
    );

    Ok(project_binary)
}

fn run_project_binary(project_binary: PathBuf) -> anyhow::Result<()> {
    Command::new(&project_binary)
        .spawn()
        .with_context(|| {
            format!(
                "Couldn't excecute project file {}",
                &project_binary.display()
            )
        })?
        .wait()
        .with_context(|| format!("Project file {} wasn't running", &project_binary.display()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;

    #[test]
    fn proper_find_project_binary() -> anyhow::Result<()> {
        let project_root = assert_fs::TempDir::new()?;
        let project_target = project_root.child("target");
        let project_binary = project_target.child(project_root.file_name().unwrap());
        project_binary.touch()?;
        let found_project_binary = find_project_binary(&project_root)?;

        anyhow::ensure!(
            found_project_binary == project_binary.path(),
            "Failed to find project binary!"
        );

        Ok(())
    }
}
