use crate::Context;
use std::{
    env,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

pub fn build_project() -> anyhow::Result<()> {
    let (project_dir, project_src, project_target) =
        find_project_dirs().with_context(|| "Failed to get project directories!")?;

    let src_files = find_src_files(&project_src).with_context(|| {
        format!(
            "Failed to gather source files from {}!",
            project_src.display()
        )
    })?;

    let project_name = match project_dir.file_name() {
        Some(name) => name,
        None => anyhow::bail!(format!(
            "Couldn't get project name from {}.",
            project_dir.display()
        )),
    };

    build_src_files(src_files, project_target, project_name)
        .with_context(|| "Failed to build source files!")?;
    Ok(())
}

fn find_project_dirs() -> anyhow::Result<(PathBuf, PathBuf, PathBuf)> {
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

    Ok((project_dir, project_src, project_target))
}

fn find_src_files(project_src: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let src_files: Vec<PathBuf> = fs::read_dir(project_src)
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

    Ok(src_files)
}

fn build_src_files(
    src_files: Vec<PathBuf>,
    project_target: PathBuf,
    project_name: &OsStr,
) -> anyhow::Result<()> {
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
