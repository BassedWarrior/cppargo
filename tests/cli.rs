use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::process::Command;

const HELLO_WORLD_PROGRAM: &str = concat!(
    "#include <iostream>\n",
    "\n",
    "int main() {\n",
    "    std::cout << \"Hello World!\\n\";\n",
    "\n",
    "    return 0;\n",
    "}\n"
);

#[cfg(test)]
mod new_subcommand {
    use super::*;

    #[test]
    fn succeed() -> anyhow::Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;

        let mut cmd = Command::cargo_bin("cppargo")?;
        cmd.arg("n").arg("foo").current_dir(tmp_dir.path());
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("Project foo created successfully"));

        let project_dir = tmp_dir.child("foo");
        project_dir.assert(predicates::path::is_dir());
        let project_src = project_dir.child("src");
        project_src.assert(predicates::path::is_dir());
        let project_target = project_dir.child("target");
        project_target.assert(predicates::path::is_dir());
        let main_file = project_src.child("main.cpp");
        main_file.assert(predicates::path::is_file());
        main_file.assert(HELLO_WORLD_PROGRAM);

        Ok(())
    }

    #[test]
    fn fail_because_project_dir_already_exists() -> anyhow::Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let existing_dir = tmp_dir.child("foo");
        existing_dir.create_dir_all()?;

        let mut cmd = Command::cargo_bin("cppargo")?;
        cmd.args(["new", "foo"]).current_dir(tmp_dir.path());
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("Failed to create project foo"));

        Ok(())
    }
}

#[cfg(test)]
mod build_subcommand {
    use super::*;

    #[test]
    fn fail_outside_cppargo_project() -> anyhow::Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;

        let mut cmd = Command::cargo_bin("cppargo")?;
        cmd.current_dir(tmp_dir.path()).arg("build");
        cmd.assert()
            .failure()
            .stderr(predicates::str::contains("Not at cppargo project root"));

        Ok(())
    }

    #[test]
    fn fail_because_project_has_no_src_files() -> anyhow::Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;

        let project_dir = tmp_dir.child("foo");
        project_dir.create_dir_all()?;
        let project_src = project_dir.child("src");
        project_src.create_dir_all()?;
        let project_target = project_dir.child("target");
        project_target.create_dir_all()?;

        let mut cmd = Command::cargo_bin("cppargo")?;
        cmd.arg("build").current_dir(project_dir.path());
        cmd.assert().failure().stderr(predicates::str::contains(
            "No source `.cpp` files to compile found",
        ));

        Ok(())
    }

    #[test]
    fn succeed() -> anyhow::Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let project_dir = tmp_dir.child("foo");
        project_dir.create_dir_all()?;
        let project_src = project_dir.child("src");
        project_src.create_dir_all()?;
        let project_target = project_dir.child("target");
        project_target.create_dir_all()?;

        let main_file = project_src.child("main.cpp");
        main_file.write_str(HELLO_WORLD_PROGRAM)?;

        let mut cmd = Command::cargo_bin("cppargo")?;
        cmd.arg("build").current_dir(tmp_dir.child("foo").path());
        cmd.assert()
            .success()
            .stdout(predicates::str::contains("Project built successfully!"));

        Ok(())
    }
}

#[test]
fn succeed_create_and_build_project() -> anyhow::Result<()> {
    let tmp_dir = assert_fs::TempDir::new()?;

    let mut cmd = Command::cargo_bin("cppargo")?;
    cmd.args(["new", "foo"]).current_dir(tmp_dir.path());
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("cppargo")?;
    cmd.arg("build").current_dir(tmp_dir.child("foo").path());
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Project built successfully!"));

    Ok(())
}

#[test]
fn succeed_build_and_run_project() -> anyhow::Result<()> {
    let tmp_dir = assert_fs::TempDir::new()?;
    let project_dir = tmp_dir.child("foo");
    project_dir.create_dir_all()?;
    let project_src = project_dir.child("src");
    project_src.create_dir_all()?;
    let project_target = project_dir.child("target");
    project_target.create_dir_all()?;

    let main_file = project_src.child("main.cpp");
    main_file.write_str(HELLO_WORLD_PROGRAM)?;

    let mut cmd = Command::cargo_bin("cppargo")?;
    cmd.arg("run").current_dir(project_dir.path());
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Hello World!"));

    Ok(())
}
