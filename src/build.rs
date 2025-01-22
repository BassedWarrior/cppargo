use crate::Context;
use std::{
    collections::HashSet,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

pub fn build_project(project_dir: &Path) -> anyhow::Result<()> {
    let (project_src, project_target) =
        find_project_dirs(project_dir).with_context(|| "Not at cppargo project root!")?;

    let src_files = find_src_files(&project_src).with_context(|| {
        format!(
            "Failed to gather source files from {}!",
            project_src.display()
        )
    })?;

    let Some(project_name) = project_dir.file_name() else {
        anyhow::bail!(format!(
            "Couldn't get project name from {}.",
            project_dir.display()
        ))
    };

    build_src_files(src_files, &project_target, project_name)
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

fn find_src_files(project_src: &Path) -> anyhow::Result<HashSet<PathBuf>> {
    let src_files: HashSet<PathBuf> = fs::read_dir(project_src)
        .with_context(|| format!("Couldn't read source directory {}.", &project_src.display()))?
        .filter_map(|f| Some(f.ok()?.path()))
        .filter(|f| f.is_dir() || f.extension().is_some_and(|ext| ext == "cpp"))
        .flat_map(|f| {
            if f.is_dir() {
                find_src_files(&f).unwrap_or_default()
            } else {
                HashSet::from([f])
            }
        })
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
    src_files: HashSet<PathBuf>,
    project_target: &Path,
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
        .with_context(|| format!("Couldn't start compiler: {compiler:?}"))?;

    anyhow::ensure!(compiler_status.success(), "Compilation failed!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;

    const MAIN_FILE_WITH_INCLUDE_MODULE: &str = concat!(
        "#include <iostream>\n",
        "#include \"module.hpp\"\n",
        "\n",
        "int main() {\n",
        "    std::cout << \"Hello World!\\n\";\n",
        "    hello_module();\n",
        "\n",
        "    return 0;\n",
        "}\n"
    );

    const MODULE_FILE: &str = concat!(
        "#include <iostream>\n",
        "\n",
        "void hello_module() {\n",
        "    std::cout << \"Hello Module!\\n\";\n",
        "}\n"
    );

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
        let tmp_dir = assert_fs::TempDir::new()?;
        let project_src = tmp_dir.child("src");

        // Indispensable main project file.
        let main_file = project_src.child("main.cpp");
        main_file.touch()?;

        // Same-level module files files.
        let module_header_file = project_src.child("module.hpp");
        module_header_file.touch()?;
        let module_file = project_src.child("module.cpp");
        module_file.touch()?;

        // An empty subdirectory.
        let empty_subdir = project_src.child("empty");
        empty_subdir.create_dir_all()?;

        // A subdirectory with both `.cpp` and `.hpp` files.
        let header_and_code_subdir = project_src.child("header_and_code");
        let header_and_code_header = header_and_code_subdir.child("foo.hpp");
        header_and_code_header.touch()?;
        let header_and_code_code = header_and_code_subdir.child("foo.cpp");
        header_and_code_code.touch()?;

        // A subdirectory with only `.hpp` files.
        let header_subdir = project_src.child("header");
        let header_header = header_subdir.child("bar.hpp");
        header_header.touch()?;

        // A doubly nested subdirectory to ensure recursivity.
        let double_nested_code_subdir = project_src.child("double").child("nested");
        let double_nested_file = double_nested_code_subdir.child("nested.cpp");
        double_nested_file.touch()?;

        let found_src_files = find_src_files(&project_src)?;
        let expected_src_files = HashSet::from(
            [
                main_file,
                module_file,
                header_and_code_code,
                double_nested_file,
            ]
            .map(|f| f.to_path_buf()),
        );

        anyhow::ensure!(
            found_src_files == expected_src_files,
            format!(
                "Failed to gather all source files!\nExpected: {:?}.\nGot: {:?}",
                expected_src_files, found_src_files
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
        main_file.write_str(MAIN_FILE_WITH_INCLUDE_MODULE)?;
        let module_file = project_src.child("module.hpp");
        module_file.write_str(MODULE_FILE)?;

        let src_files = HashSet::from([main_file.to_path_buf()]);
        let project_name = project_root.file_name().unwrap();
        build_src_files(src_files, project_target.path(), project_name)?;
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
        main_file.write_str(MAIN_FILE_WITH_INCLUDE_MODULE)?;
        let module_file = project_src.child("module.hpp");
        module_file.write_str(MODULE_FILE)?;

        build_project(&project_root)?;
        let project_name = project_root.file_name().unwrap();
        project_target
            .child(project_name)
            .assert(predicates::path::is_file());

        Ok(())
    }
}
