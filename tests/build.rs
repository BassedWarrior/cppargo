mod common;
use common::*;

#[test]
fn fail_outside_cppargo_project() -> anyhow::Result<()> {
    let tmp_dir = assert_fs::TempDir::new()?;

    let mut cmd = Command::cargo_bin("cppargo")?;
    cmd.current_dir(tmp_dir.path()).arg("build");
    cmd.assert().failure().stderr(predicate::str::contains(
        "Failed to find project manifest `Cppargo.toml` up to /!",
    ));

    Ok(())
}

#[test]
fn fail_because_project_has_no_main_file() -> anyhow::Result<()> {
    let tmp_dir = assert_fs::TempDir::new()?;

    let project_root = tmp_dir.child("foo");
    project_root.create_dir_all()?;
    let project_src = project_root.child("src");
    project_src.create_dir_all()?;

    let project_manifest = project_root.child("Cppargo.toml");
    project_manifest.write_str(PROJECT_MANIFEST)?;

    let mut cmd = Command::cargo_bin("cppargo")?;
    cmd.current_dir(project_root.path()).arg("build");
    cmd.assert().failure().stderr(predicate::str::contains(
        format!(
            "Missing \"src/main.cpp\" file in {}!",
            project_src.display()
        )
    ));

    Ok(())
}

#[test]
fn succeed_without_existing_target_dir() -> anyhow::Result<()> {
    let tmp_dir = assert_fs::TempDir::new()?;
    let project_root = tmp_dir.child("foo");
    project_root.create_dir_all()?;
    let project_src = project_root.child("src");
    project_src.create_dir_all()?;

    let project_manifest = project_root.child("Cppargo.toml");
    project_manifest.write_str(PROJECT_MANIFEST)?;

    let main_file = project_src.child("main.cpp");
    main_file.write_str(HELLO_WORLD_PROGRAM)?;

    let mut cmd = Command::cargo_bin("cppargo")?;
    cmd.current_dir(project_root.path()).arg("build");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Project built successfully!"));

    Ok(())
}

#[test]
fn succeed_with_existing_target_dir() -> anyhow::Result<()> {
    let tmp_dir = assert_fs::TempDir::new()?;
    let project_root = tmp_dir.child("foo");
    project_root.create_dir_all()?;
    let project_src = project_root.child("src");
    project_src.create_dir_all()?;

    let project_manifest = project_root.child("Cppargo.toml");
    project_manifest.write_str(PROJECT_MANIFEST)?;

    let main_file = project_src.child("main.cpp");
    main_file.write_str(HELLO_WORLD_PROGRAM)?;

    let project_target = project_root.child("target");
    project_target.create_dir_all()?;
    let existing_binary = project_target.child("foo");
    existing_binary.touch()?;

    let mut cmd = Command::cargo_bin("cppargo")?;
    cmd.current_dir(project_root.path()).arg("build");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Project built successfully!"));

    Ok(())
}
