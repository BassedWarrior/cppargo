mod common;
use common::*;

#[test]
fn fail_outside_cppargo_project() -> anyhow::Result<()> {
    let tmp_dir = assert_fs::TempDir::new()?;

    let mut cmd = Command::cargo_bin("cppargo")?;
    cmd.current_dir(tmp_dir.path()).arg("build");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Not at cppargo project root"));

    Ok(())
}

#[test]
fn fail_because_project_has_no_src_files() -> anyhow::Result<()> {
    let tmp_dir = assert_fs::TempDir::new()?;

    let project_root = tmp_dir.child("foo");
    project_root.create_dir_all()?;
    let project_src = project_root.child("src");
    project_src.create_dir_all()?;
    let project_target = project_root.child("target");
    project_target.create_dir_all()?;

    let mut cmd = Command::cargo_bin("cppargo")?;
    cmd.current_dir(project_root.path()).arg("build");
    cmd.assert().failure().stderr(predicate::str::contains(
        "No source `.cpp` files to compile found",
    ));

    Ok(())
}

#[test]
fn succeed() -> anyhow::Result<()> {
    let tmp_dir = assert_fs::TempDir::new()?;
    let project_root = tmp_dir.child("foo");
    project_root.create_dir_all()?;
    let project_src = project_root.child("src");
    project_src.create_dir_all()?;
    let project_target = project_root.child("target");
    project_target.create_dir_all()?;

    let main_file = project_src.child("main.cpp");
    main_file.write_str(HELLO_WORLD_PROGRAM)?;

    let mut cmd = Command::cargo_bin("cppargo")?;
    cmd.current_dir(tmp_dir.child("foo").path()).arg("build");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Project built successfully!"));

    Ok(())
}
