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
    let project_root = find_project_root(current_dir).with_context(|| {
        format!(
            "Current directory {} is not inside a `cppargo` project!",
            current_dir.display()
        )
    })?;

    let project_manifest = get_project_manifest(&project_root)
        .with_context(|| "Failed to parse project manifest!")?;
    let project_name = get_project_name(&project_manifest)
        .with_context(|| "Failed to parse project name!")?;

    let project_src = project_root.join("src");

    let src_files = find_src_files(&project_src).with_context(|| {
        format!(
            "Failed to gather source files from {}!",
            project_src.display()
        )
    })?;

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
                anyhow::bail!(format!(
                    "Found manifest {} has no parent directory to use as project root!",
                    manifest.path().display()
                ));
            }
        }
        None => {
            if let Some(parent_dir) = dir.parent() {
                find_project_root(parent_dir)?
            } else {
                anyhow::bail!(format!(
                    "Failed to find project manifest `Cppargo.toml` up to {}!",
                    dir.display()
                ))
            }
        }
    };

    Ok(project_root)
}

fn get_project_manifest(project_root: &Path) -> anyhow::Result<toml_edit::DocumentMut> {
    let manifest_path = project_root.join("Cppargo.toml");

    let manifest = toml_edit::DocumentMut::from_str(
        &fs::read_to_string(&manifest_path).with_context(|| {
            format!(
                "Failed to read project manifest {} file!",
                manifest_path.display()
            )
        })?,
    )
    .with_context(|| {
        format!(
            "Failed to parse project manifest {} file!",
            manifest_path.display()
        )
    })?;

    Ok(manifest)
}

fn get_project_name(project_manifest: &toml_edit::DocumentMut) -> anyhow::Result<String> {
    let Some(project_item) = project_manifest.get("project") else {
        anyhow::bail!("Missing '[project]' table in manifest!")
    };
    let toml_edit::Item::Table(project_table) = project_item else {
        anyhow::bail!("Found 'project' key is not a table! (Should be '[project]')")
    };

    let Some(project_name_item) = project_table.get("name") else {
        anyhow::bail!("Missing 'name' key in '[project]' table in manifest!")
    };

    let toml_edit::Item::Value(project_name_value) = project_name_item else {
        anyhow::bail!("Found 'name' key in '[project]' table is not a value! \
                      (Should be 'name = \"YourMagicProjectName\"')"
        )
    };

    let toml_edit::Value::String(project_name_formatted) = project_name_value else {
        anyhow::bail!("Value for 'name' key in '[project]' table is not a string! \
                      (Should be 'name = \"YourMagicProjectName\"')"
        )
    };

    if project_name_formatted.clone().into_value().is_empty() {
        anyhow::bail!("Missing value for 'name' key in '[project]' table! \
                      (Should be 'name = \"YourMagicProjectName\"')"
        )
    }

    Ok(project_name_formatted.clone().into_value())
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

    const PROJECT_MANIFEST: &str = "[project]\nname = \"foo\"\n";
    const PROJEJCT_MANIFEST_WITH_NO_NAME: &str = "[project]\nname =\n";

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

    #[cfg(test)]
    mod find_project_root {
        use super::*;

        #[test]
        fn in_current_dir() -> anyhow::Result<()> {
            let tmp_dir = assert_fs::TempDir::new()?;
            let project_manifest = tmp_dir.child("Cppargo.toml");
            project_manifest.touch()?;

            anyhow::ensure!(
                tmp_dir.to_path_buf() == find_project_root(&tmp_dir)?,
                "Failed to find project root from project root!"
            );

            Ok(())
        }

        #[test]
        fn from_nested_dir() -> anyhow::Result<()> {
            let tmp_dir = assert_fs::TempDir::new()?;
            let project_manifest = tmp_dir.child("Cppargo.toml");
            project_manifest.touch()?;

            let project_src = tmp_dir.child("src");
            project_src.create_dir_all()?;

            anyhow::ensure!(
                tmp_dir.to_path_buf() == find_project_root(&project_src)?,
                "Failed to find project root from inside nested directory!"
            );

            Ok(())
        }

        #[test]
        fn fail_outside_project() -> anyhow::Result<()> {
            let tmp_dir = assert_fs::TempDir::new()?;

            match find_project_root(&tmp_dir) {
                Ok(root) => {
                    anyhow::bail!(format!("Found some project root at {}!", root.display()))
                }
                Err(err) => {
                    if err.to_string() != "Failed to find project manifest `Cppargo.toml` up to /!"
                    {
                        anyhow::bail!(format!(
                            "Got a non-expected error: \"{}\"!",
                            err.to_string()
                        ))
                    }
                }
            }

            Ok(())
        }
    }

    #[cfg(test)]
    mod get_project_manifest {
        use super::*;

        #[test]
        fn succeed() -> anyhow::Result<()> {
            let tmp_dir = assert_fs::TempDir::new()?;
            let project_manifest_file = tmp_dir.child("Cppargo.toml");
            project_manifest_file.write_str(PROJECT_MANIFEST)?;

            get_project_manifest(tmp_dir.path())
                .with_context(|| "Failed to parse proper manifest!")?;

            Ok(())
        }

        #[test]
        fn missing_manifest() -> anyhow::Result<()> {
            let tmp_dir = assert_fs::TempDir::new()?;

            match get_project_manifest(tmp_dir.path()) {
                Err(err) => {
                    if err.to_string()
                        == format!(
                            "Failed to read project manifest {} file!",
                            tmp_dir.child("Cppargo.toml").path().display()
                        )
                    {
                        return Ok(());
                    }

                    anyhow::bail!(format!(
                        "Found unexpected error: {}\nExpected: Failed to read project manifest {} file!",
                        err,
                        tmp_dir.child("Cppargo.toml").path().display()
                    ));
                }
                Ok(_) => {
                    anyhow::bail!("Shouldn't have parsed non-existent manifest!");
                }
            }
        }

        #[test]
        fn bad_syntax_in_manifest() -> anyhow::Result<()> {
            let tmp_dir = assert_fs::TempDir::new()?;
            let project_manifest_file = tmp_dir.child("Cppargo.toml");
            project_manifest_file.write_str(PROJEJCT_MANIFEST_WITH_NO_NAME)?;

            match get_project_manifest(tmp_dir.path()) {
                Err(err) => {
                    if err.to_string()
                        == format!(
                            "Failed to parse project manifest {} file!",
                            tmp_dir.child("Cppargo.toml").path().display()
                        )
                    {
                        return Ok(());
                    }

                    anyhow::bail!(format!(
                        "Found unexpected error: {}\nExpected: Failed to parse project manifest {} file!",
                        err,
                        tmp_dir.child("Cppargo.toml").path().display()
                    ));
                }
                Ok(_) => {
                    anyhow::bail!("Shouldn't have parsed manifest with bad syntax!");
                }
            }
        }
    }

    #[cfg(test)]
    mod get_project_name {
        use super::*;

        #[test]
        fn succeed() -> anyhow::Result<()> {
            const TEST_PROJECT_NAME: &str = "foo";
            let mut project_manifest = toml_edit::DocumentMut::new();
            project_manifest["project"] = toml_edit::table();
            project_manifest["project"]["name"] = toml_edit::value(TEST_PROJECT_NAME);

            let project_name = get_project_name(&project_manifest)?;

            anyhow::ensure!(
                project_name == TEST_PROJECT_NAME,
                format!(
                    "Got wrong project name!\n\
                    Expected: {}\n\
                    Got: {}\n",
                    TEST_PROJECT_NAME, project_name
                )
            );

            Ok(())
        }

        fn test_fail(
            project_manifest: &toml_edit::DocumentMut,
            expected_error: &str,
        ) -> anyhow::Result<()> {
            let anyhow_err = match get_project_name(project_manifest) {
                Ok(name) => {
                    anyhow::bail!(format!(
                        "Succeded when it should have failed!\n\
                         Got name: {}",
                        name
                    ))
                }
                Err(err) => err,
            };

            if anyhow_err.to_string() == expected_error {
                return Ok(());
            }

            anyhow::bail!(format!(
                "Got wrong type of error!\n\
                     Expected: {}\n\
                     Got: {}",
                expected_error,
                anyhow_err
            ));
        }

        #[test]
        fn no_project_table_in_manifest() -> anyhow::Result<()> {
            const EXPECTED_ERROR: &str = "Missing '[project]' table in manifest!";

            let project_manifest = toml_edit::DocumentMut::new();

            test_fail(&project_manifest, EXPECTED_ERROR)
        }

        #[test]
        fn project_key_is_not_list() -> anyhow::Result<()> {
            const EXPECTED_ERROR: &str = "Found 'project' key is not a table! \
                                          (Should be '[project]')";

            let mut project_manifest = toml_edit::DocumentMut::new();
            project_manifest["project"] = toml_edit::value("");

            test_fail(&project_manifest, EXPECTED_ERROR)
        }

        #[test]
        fn no_name_key_in_manifest() -> anyhow::Result<()> {
            const EXPECTED_ERROR: &str = "Missing 'name' key in '[project]' table in manifest!";

            let mut project_manifest = toml_edit::DocumentMut::new();
            project_manifest["project"] = toml_edit::table();

            test_fail(&project_manifest, EXPECTED_ERROR)
        }

        #[test]
        fn name_key_is_not_value() -> anyhow::Result<()> {
            const EXPECTED_ERROR: &str = "Found 'name' key in '[project]' table is not a value! \
                                          (Should be 'name = \"YourMagicProjectName\"')";

            let mut project_manifest = toml_edit::DocumentMut::new();
            project_manifest["project"] = toml_edit::table();
            project_manifest["project"]["name"] = toml_edit::table();

            test_fail(&project_manifest, EXPECTED_ERROR)
        }

        #[test]
        fn name_value_is_not_a_string() -> anyhow::Result<()> {
            const EXPECTED_ERROR: &str = "Value for 'name' key in '[project]' table is not a string! \
                                          (Should be 'name = \"YourMagicProjectName\"')";

            let mut project_manifest = toml_edit::DocumentMut::new();
            project_manifest["project"] = toml_edit::table();
            project_manifest["project"]["name"] = toml_edit::value(0);

            test_fail(&project_manifest, EXPECTED_ERROR)
        }

        #[test]
        fn name_value_is_empty() -> anyhow::Result<()> {
            const EXPECTED_ERROR: &str = "Missing value for 'name' key in '[project]' table! \
                                          (Should be 'name = \"YourMagicProjectName\"')";

            let mut project_manifest = toml_edit::DocumentMut::new();
            project_manifest["project"] = toml_edit::table();
            project_manifest["project"]["name"] = toml_edit::value("");

            test_fail(&project_manifest, EXPECTED_ERROR)
        }
    }

    #[cfg(test)]
    mod find_src_files {
        use super::*;

        fn ensure_found_expected_files(
            found: &HashSet<PathBuf>,
            expected: &HashSet<PathBuf>,
        ) -> anyhow::Result<()> {
            anyhow::ensure!(
                found == expected,
                format!(
                    "Failed to gather all source files!\nGot: {:?}.\nExpected: {:?}",
                    found, expected
                )
            );

            Ok(())
        }

        #[test]
        fn find_main() -> anyhow::Result<()> {
            let tmp_dir = assert_fs::TempDir::new()?;
            let project_src = tmp_dir.child("src");

            let main_file = project_src.child("main.cpp");
            main_file.touch()?;

            let found_src_files = find_src_files(&project_src)?;
            let expected_src_files = HashSet::from([main_file].map(|f| f.to_path_buf()));

            ensure_found_expected_files(&found_src_files, &expected_src_files)?;

            Ok(())
        }

        #[test]
        fn find_only_cpp_files() -> anyhow::Result<()> {
            let tmp_dir = assert_fs::TempDir::new()?;
            let project_src = tmp_dir.child("src");

            let main_file = project_src.child("main.cpp");
            main_file.touch()?;

            let header_file = project_src.child("header.hpp");
            header_file.touch()?;

            let text_file = project_src.child("text.txt");
            text_file.touch()?;

            let c_file = project_src.child("c_file.c");
            c_file.touch()?;

            let c_header_file = project_src.child("c_header.h");
            c_header_file.touch()?;

            let binary_file = project_src.child("binary");
            binary_file.touch()?;

            let found_src_files = find_src_files(&project_src)?;
            let expected_src_files = HashSet::from([main_file].map(|f| f.to_path_buf()));

            ensure_found_expected_files(&found_src_files, &expected_src_files)?;

            Ok(())
        }

        #[test]
        fn find_same_dir_files() -> anyhow::Result<()> {
            let tmp_dir = assert_fs::TempDir::new()?;
            let project_src = tmp_dir.child("src");

            let main_file = project_src.child("main.cpp");
            main_file.touch()?;

            let other_file = project_src.child("other.cpp");
            other_file.touch()?;

            let found_src_files = find_src_files(&project_src)?;
            let expected_src_files =
                HashSet::from([main_file, other_file].map(|f| f.to_path_buf()));

            ensure_found_expected_files(&found_src_files, &expected_src_files)?;

            Ok(())
        }

        #[test]
        fn find_with_empty_directories() -> anyhow::Result<()> {
            let tmp_dir = assert_fs::TempDir::new()?;
            let project_src = tmp_dir.child("src");

            let main_file = project_src.child("main.cpp");
            main_file.touch()?;

            let empty_dir = project_src.child("empty");
            empty_dir.create_dir_all()?;

            let found_src_files = find_src_files(&project_src)?;
            let expected_src_files = HashSet::from([main_file].map(|f| f.to_path_buf()));

            ensure_found_expected_files(&found_src_files, &expected_src_files)?;

            Ok(())
        }

        #[test]
        fn find_within_nested_directories() -> anyhow::Result<()> {
            let tmp_dir = assert_fs::TempDir::new()?;
            let project_src = tmp_dir.child("src");

            let main_file = project_src.child("main.cpp");
            main_file.touch()?;

            let nested_file = project_src.child("nested").child("nested.cpp");
            nested_file.touch()?;

            let found_src_files = find_src_files(&project_src)?;
            let expected_src_files =
                HashSet::from([main_file, nested_file].map(|f| f.to_path_buf()));

            ensure_found_expected_files(&found_src_files, &expected_src_files)?;

            Ok(())
        }

        #[test]
        fn find_within_doubly_nested_directories() -> anyhow::Result<()> {
            let tmp_dir = assert_fs::TempDir::new()?;
            let project_src = tmp_dir.child("src");

            let main_file = project_src.child("main.cpp");
            main_file.touch()?;

            let doubly_nested_file = project_src
                .child("doubly")
                .child("nested")
                .child("doubly_nested.cpp");
            doubly_nested_file.touch()?;

            let found_src_files = find_src_files(&project_src)?;
            let expected_src_files =
                HashSet::from([main_file, doubly_nested_file].map(|f| f.to_path_buf()));

            ensure_found_expected_files(&found_src_files, &expected_src_files)?;

            Ok(())
        }
    }

    #[cfg(test)]
    mod ensure_target_dir_exists {
        use super::*;

        #[test]
        fn create_missing_target_dir() -> anyhow::Result<()> {
            let tmp_dir = assert_fs::TempDir::new()?;

            let project_target = tmp_dir.child("target");

            ensure_target_dir_exists(project_target.path())?;
            project_target.assert(predicates::path::is_dir());

            Ok(())
        }

        #[test]
        fn target_dir_already_exists() -> anyhow::Result<()> {
            let tmp_dir = assert_fs::TempDir::new()?;

            let project_target = tmp_dir.child("target");
            project_target.create_dir_all()?;

            ensure_target_dir_exists(project_target.path())?;
            project_target.assert(predicates::path::is_dir());

            Ok(())
        }
    }

    #[test]
    fn proper_build_src_files() -> anyhow::Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let project_root = tmp_dir.child("foo");
        let project_src = project_root.child("src");
        let project_target = project_root.child("target");
        project_target.create_dir_all()?;

        let main_file = project_src.child("main.cpp");
        main_file.write_str(MAIN_FILE_WITH_INCLUDE_MODULE)?;
        let module_file = project_src.child("module.hpp");
        module_file.write_str(MODULE_FILE)?;

        let project_binary = project_target.child("foo");

        let src_files = HashSet::from([main_file.to_path_buf()]);
        let project_name = "foo";
        build_src_files(src_files, project_binary.path())?;
        project_target
            .child(project_name)
            .assert(predicates::path::is_file());

        Ok(())
    }

    #[test]
    fn proper_main() -> anyhow::Result<()> {
        let project_root = assert_fs::TempDir::new()?;
        let project_src = project_root.child("src");
        let project_target = project_root.child("target");
        project_target.create_dir_all()?;

        let project_manifest = project_root.child("Cppargo.toml");
        project_manifest.write_str(PROJECT_MANIFEST)?;

        let main_file = project_src.child("main.cpp");
        main_file.write_str(MAIN_FILE_WITH_INCLUDE_MODULE)?;
        let module_file = project_src.child("module.hpp");
        module_file.write_str(MODULE_FILE)?;

        main(&project_root)?;
        project_target
            .child("foo")
            .assert(predicates::path::is_file());

        Ok(())
    }
}
