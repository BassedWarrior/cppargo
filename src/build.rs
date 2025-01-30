use crate::Context;
use std::{
    collections::HashSet,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

pub fn main(current_dir: &Path) -> anyhow::Result<()> {
    let project_root =
        find_project_root(current_dir).with_context(|| "Not at cppargo project root!")?;

    let project_src = project_root.join("src");

    let src_files = find_src_files(&project_src).with_context(|| {
        format!(
            "Failed to gather source files from {}!",
            project_src.display()
        )
    })?;

    let project_manifest = project_root.join("Cppargo.toml");
    let project_name = get_project_name(&project_manifest)?;

    let project_target = project_root.join("target");
    ensure_target_dir_exists(&project_target)
        .with_context(|| "Failed to ensure target directory exists for storing built binaries!")?;

    let binary_path = project_target.join(project_name);
    build_src_files(src_files, &binary_path).with_context(|| "Failed to build source files!")?;
    Ok(())
}

fn find_project_root(dir: &Path) -> anyhow::Result<PathBuf> {
    let project_root = match fs::read_dir(dir)?
        .flatten()
        .find(|f| f.file_name() == OsString::from_str("Cppargo.toml").unwrap())
    {
        Some(manifest) => {
            if let Some(parent) = manifest.path().parent() {
                parent.to_path_buf()
            } else {
                anyhow::bail!("Couldn't get root!");
            }
        }
        None => {
            if let Some(parent_dir) = dir.parent() {
                find_project_root(parent_dir)?
            } else {
                anyhow::bail!(format!(
                    "Failed to find project root up to {}!",
                    dir.display()
                ))
            }
        }
    };

    Ok(project_root)
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

fn get_project_name(project_manifest: &Path) -> anyhow::Result<String> {
    let manifest = toml_edit::DocumentMut::from_str(&fs::read_to_string(project_manifest)?)?;
    let Some(project_name) = manifest["project"]["name"].as_str() else {
        anyhow::bail!("Failed to gather project name!")
    };

    Ok(String::from_str(project_name)?)
}

fn ensure_target_dir_exists(project_target: &Path) -> anyhow::Result<()> {
    if !project_target.try_exists()? {
        fs::create_dir(project_target).with_context(|| {
            format!(
                "Failed to create project target directory at {}!",
                project_target.display()
            )
        })?;
    }

    Ok(())
}

fn build_src_files(src_files: HashSet<PathBuf>, binary_path: &Path) -> anyhow::Result<()> {
    let output_args = [
        "-o",
        match binary_path.to_str() {
            Some(s) => s,
            None => anyhow::bail!(format!(
                "Failed to parse binary path to string: {}",
                binary_path.display()
            )),
        },
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
