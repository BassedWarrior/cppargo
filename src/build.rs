use crate::Context;
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

pub fn build_project(project_dir: &Path) -> anyhow::Result<()> {
    let (project_src, project_target) =
        find_project_dirs(project_dir).with_context(|| "Failed to get project directories!")?;

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

fn find_project_dirs(project_dir: &Path) -> anyhow::Result<(PathBuf, PathBuf)> {
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

    Ok((project_src, project_target))
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

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;

    #[test]
    fn proper_find_dirs() -> anyhow::Result<()> {
        let project_root = assert_fs::TempDir::new()?;
        let project_src = project_root.child("src");
        let project_target = project_root.child("target");
        project_src.create_dir_all()?;
        project_target.create_dir_all()?;

        anyhow::ensure!(
            (project_src.to_path_buf(), project_target.to_path_buf())
                == find_project_dirs(&project_root)?,
            "Failed to find proper project directories!"
        );

        Ok(())
    }

    #[test]
    fn proper_find_src_files() -> anyhow::Result<()> {
        let project_root = assert_fs::TempDir::new()?;
        let project_src = project_root.child("src");
        let main_file = project_src.child("main.cpp");
        let module_file = project_src.child("module.cpp");
        main_file.touch()?;
        module_file.touch()?;
        let found_src_files = find_src_files(&project_src)?;

        anyhow::ensure!(
            found_src_files.len() == 2
                && found_src_files.contains(&main_file.to_path_buf())
                && found_src_files.contains(&module_file.to_path_buf()),
            format!(
                "Failed to gather all source files!\nExpected: {:?}.\nGot: {:?}",
                vec!(main_file.path(), module_file.path()),
                found_src_files
            )
        );

        Ok(())
    }

    #[test]
    fn proper_build_src_files() -> anyhow::Result<()> {
        let project_root = assert_fs::TempDir::new()?;
        let project_src = project_root.child("src");
        let project_target = project_root.child("target");
        project_target.create_dir_all()?;

        let main_file = project_src.child("main.cpp");
        let module_file = project_src.child("module.hpp");
        main_file.write_str(concat!(
            "#include <iostream>\n",
            "#include \"module.hpp\"\n",
            "\n",
            "int main() {\n",
            "    std::cout << \"Hello World!\\n\";\n",
            "    hello_module();\n",
            "\n",
            "    return 0;\n",
            "}\n"
        ))?;
        module_file.write_str(concat!(
            "#include <iostream>\n",
            "\n",
            "void hello_module() {\n",
            "    std::cout << \"Hello Module!\\n\";\n",
            "}\n"
        ))?;

        let src_files = vec![main_file.to_path_buf()];
        let project_name = project_root.file_name().unwrap();
        build_src_files(src_files, project_target.to_path_buf(), project_name)?;
        project_target
            .child(project_name)
            .assert(predicates::path::is_file());

        Ok(())
    }

    #[test]
    fn proper_build_project() -> anyhow::Result<()> {
        let project_root = assert_fs::TempDir::new()?;
        let project_src = project_root.child("src");
        let project_target = project_root.child("target");
        project_target.create_dir_all()?;

        let main_file = project_src.child("main.cpp");
        let module_file = project_src.child("module.hpp");
        main_file.write_str(concat!(
            "#include <iostream>\n",
            "#include \"module.hpp\"\n",
            "\n",
            "int main() {\n",
            "    std::cout << \"Hello World!\\n\";\n",
            "    hello_module();\n",
            "\n",
            "    return 0;\n",
            "}\n"
        ))?;
        module_file.write_str(concat!(
            "#include <iostream>\n",
            "\n",
            "void hello_module() {\n",
            "    std::cout << \"Hello Module!\\n\";\n",
            "}\n"
        ))?;

        build_project(&project_root)?;
        let project_name = project_root.file_name().unwrap();
        project_target
            .child(project_name)
            .assert(predicates::path::is_file());

        Ok(())
    }
}
