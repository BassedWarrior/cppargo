use predicates::str::{ContainsPredicate, EndsWithPredicate};
use std::{ops::Deref, path::Path};

mod common;
use common::*;

fn ensure_project_created_successfully<T>(project_root: T)
where
    T: PathChild + PathAssert + Deref<Target = Path>,
{
    ensure_project_structure_created_successfully(&project_root);
}

fn ensure_project_structure_created_successfully<T>(project_root: &T)
where
    T: PathChild + PathAssert + Deref<Target = Path>,
{
    project_root.assert(predicates::path::is_dir());
    let project_manifest = project_root.child("Cppargo.toml");
    project_manifest.assert(format!(
        "[project]\nname = \"{}\"\n",
        project_root.file_name().unwrap().to_str().unwrap()
    ));
    let project_src = project_root.child("src");
    project_src.assert(predicates::path::is_dir());
    let main_file = project_src.child("main.cpp");
    main_file.assert(predicates::path::is_file());
    main_file.assert(HELLO_WORLD_PROGRAM);
}

fn success_predicate(project_path: &Path) -> EndsWithPredicate {
    predicate::str::ends_with(format!(
        "Project {} created successfully!\n",
        project_path.display()
    ))
}

fn fail_predicate(project_path: &Path) -> ContainsPredicate {
    predicate::str::contains(format!(
        "Failed to create project {}\n",
        project_path.display()
    ))
}

#[test]
fn succeed_relative() -> anyhow::Result<()> {
    let tmp_dir = assert_fs::TempDir::new()?;
    let project_root = tmp_dir.child("foo");
    let project_path = project_root.path().strip_prefix(tmp_dir.path())?;

    let mut cmd = Command::cargo_bin("cppargo")?;
    cmd.current_dir(tmp_dir.path()).arg("new").arg(project_path);
    cmd.assert()
        .success()
        .stdout(success_predicate(project_path));

    ensure_project_created_successfully(project_root);

    Ok(())
}

#[test]
fn succeed_relative_nested() -> anyhow::Result<()> {
    let tmp_dir = assert_fs::TempDir::new()?;
    let project_root = tmp_dir.child("foo").child("bar");
    let project_path = project_root.path().strip_prefix(tmp_dir.path())?;

    let mut cmd = Command::cargo_bin("cppargo")?;
    cmd.current_dir(tmp_dir.path()).arg("new").arg(project_path);
    cmd.assert()
        .success()
        .stdout(success_predicate(project_path));

    ensure_project_created_successfully(project_root);

    Ok(())
}

#[test]
fn succeed_absolute() -> anyhow::Result<()> {
    let tmp_dir = assert_fs::TempDir::new()?;
    let project_root = tmp_dir.child("foo");
    let project_path = project_root.path();

    let mut cmd = Command::cargo_bin("cppargo")?;
    cmd.arg("new").arg(project_path);
    cmd.assert()
        .success()
        .stdout(success_predicate(project_path));

    ensure_project_created_successfully(project_root);

    Ok(())
}

#[test]
fn succeed_absolute_nested() -> anyhow::Result<()> {
    let tmp_dir = assert_fs::TempDir::new()?;
    let project_root = tmp_dir.child("foo").child("bar");
    let project_path = project_root.path();

    let mut cmd = Command::cargo_bin("cppargo")?;
    cmd.arg("new").arg(project_path);
    cmd.assert()
        .success()
        .stdout(success_predicate(project_path));

    ensure_project_created_successfully(project_root);

    Ok(())
}

#[test]
fn fail_because_project_dir_already_exists() -> anyhow::Result<()> {
    let tmp_dir = assert_fs::TempDir::new()?;
    let existing_dir = tmp_dir.child("foo");
    existing_dir.create_dir_all()?;
    let project_path = tmp_dir.child("foo").path().to_path_buf();

    let mut cmd = Command::cargo_bin("cppargo")?;
    cmd.current_dir(tmp_dir.path())
        .arg("new")
        .arg(&project_path);
    cmd.assert().failure().stderr(fail_predicate(&project_path));

    Ok(())
}
