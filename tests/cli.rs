mod common;
use common::*;

#[test]
fn succeed_create_and_build_project() -> anyhow::Result<()> {
    let tmp_dir = assert_fs::TempDir::new()?;

    let mut cmd = Command::cargo_bin("cppargo")?;
    cmd.args(["new", "foo"]).current_dir(tmp_dir.path());
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("cppargo")?;
    cmd.current_dir(tmp_dir.child("foo").path()).arg("build");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Project built successfully!"));

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
    cmd.current_dir(project_dir.path()).arg("run");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Hello World!"));

    Ok(())
}
