extern crate temp_testdir;

use assert_cmd::Command;
use std::fs::{create_dir_all, read_to_string, File};
use std::io::Write;
use temp_testdir::TempDir;

#[test]
fn should_not_refresh_on_empty_config() {
    // setup dirs
    let sys_root = TempDir::default();
    let home_root = sys_root.join("home");
    let repo_root = home_root.join("dotfiles");

    // create dirs
    let _ = create_dir_all(&repo_root);

    let config_file_path = &repo_root.join("dott.config");
    File::create(config_file_path).unwrap();

    let mut cmd = Command::cargo_bin("dott").unwrap();
    let assert = cmd
        .current_dir(&repo_root)
        .arg("refresh")
        .arg("-y")
        .env("HOME", home_root.to_str().unwrap())
        .assert();
    assert.success().code(0);

    let mut repo_dir_contents = repo_root.read_dir().unwrap();
    assert!(repo_root.exists());
    assert!(repo_dir_contents.next().is_some());
    assert!(repo_dir_contents.next().is_none());
}

#[test]
fn should_add_missing_files() {
    // setup dirs
    let sys_root = TempDir::default();
    let home_root = sys_root.join("home");
    let repo_root = home_root.join("dotfiles");

    // create dirs
    let _ = create_dir_all(&repo_root);

    // create tracked_file
    let file_path = home_root.join("some_file.txt");
    let mut file = File::create(file_path).unwrap();
    file.write_all(b"some content").unwrap();

    let config_file_path = &repo_root.join("dott.config");
    let mut config_file = File::create(config_file_path).unwrap();
    writeln!(&mut config_file, "some_file.txt").unwrap();

    let mut cmd = Command::cargo_bin("dott").unwrap();
    let assert = cmd
        .current_dir(&repo_root)
        .arg("refresh")
        .arg("-y")
        .env("HOME", home_root.to_str().unwrap())
        .assert();
    assert.success().code(0);

    let synced_file = repo_root.join("some_file.txt");
    assert!(synced_file.exists());
    assert_eq!(
        read_to_string(synced_file).unwrap(),
        "some content".to_string()
    );
}

#[test]
fn should_overwrite_file_content() {
    // setup dirs
    let sys_root = TempDir::default();
    let home_root = sys_root.join("home");
    let repo_root = home_root.join("dotfiles");

    // create dirs
    let _ = create_dir_all(&repo_root);

    // create tracked_file
    let file_path = home_root.join("some_file.txt");
    let mut file = File::create(file_path).unwrap();
    file.write_all(b"updated content").unwrap();

    let repo_file_path = repo_root.join("some_file.txt");
    let mut file = File::create(&repo_file_path).unwrap();
    file.write_all(b"legacy content").unwrap();

    let config_file_path = &repo_root.join("dott.config");
    let mut config_file = File::create(config_file_path).unwrap();
    writeln!(&mut config_file, "some_file.txt").unwrap();

    let mut cmd = Command::cargo_bin("dott").unwrap();
    let assert = cmd
        .current_dir(&repo_root)
        .arg("refresh")
        .arg("-y")
        .env("HOME", home_root.to_str().unwrap())
        .assert();
    assert.success().code(0);

    assert!(repo_file_path.exists());
    assert_eq!(
        read_to_string(repo_file_path).unwrap(),
        "updated content".to_string()
    );
}

#[test]
fn should_not_remove_untracked_files_from_repo() {
    // setup dirs
    let sys_root = TempDir::default();
    let home_root = sys_root.join("home");
    let repo_root = home_root.join("dotfiles");

    // create dirs
    let _ = create_dir_all(&repo_root);

    // create tracked_file
    let file_path = home_root.join("some_file.txt");
    File::create(file_path).unwrap();

    let repo_file_path = repo_root.join("another_file.txt");
    File::create(&repo_file_path).unwrap();

    let config_file_path = &repo_root.join("dott.config");
    let mut config_file = File::create(config_file_path).unwrap();
    writeln!(&mut config_file, "some_file.txt").unwrap();

    let mut cmd = Command::cargo_bin("dott").unwrap();
    let assert = cmd
        .current_dir(&repo_root)
        .arg("refresh")
        .arg("-y")
        .env("HOME", home_root.to_str().unwrap())
        .assert();
    assert.success().code(0);

    let mut repo_files = repo_root.read_dir().unwrap();

    assert!(repo_file_path.exists());
    assert_eq!(
        repo_files.next().unwrap().unwrap().path(),
        repo_root.join("some_file.txt")
    );
}

#[test]
fn should_remove_tracked_files_from_repo_if_they_are_missing_in_home() {
    // setup dirs
    let sys_root = TempDir::default();
    let home_root = sys_root.join("home");
    let repo_root = home_root.join("dotfiles");

    // create dirs
    let _ = create_dir_all(&repo_root);

    // create tracked_file
    let file_path = home_root.join("some_file.txt");
    File::create(file_path).unwrap();

    let repo_file_path = repo_root.join("another_file.txt");
    File::create(&repo_file_path).unwrap();

    let config_file_path = &repo_root.join("dott.config");
    let mut config_file = File::create(config_file_path).unwrap();
    writeln!(&mut config_file, "some_file.txt").unwrap();
    writeln!(&mut config_file, "another_file.txt").unwrap();

    let mut cmd = Command::cargo_bin("dott").unwrap();
    let assert = cmd
        .current_dir(&repo_root)
        .arg("refresh")
        .arg("-y")
        .env("HOME", home_root.to_str().unwrap())
        .assert();
    assert.success().code(0);

    let mut repo_files = repo_root.read_dir().unwrap();

    assert!(!repo_file_path.exists());
    assert_eq!(
        repo_files.next().unwrap().unwrap().path(),
        repo_root.join("some_file.txt")
    );
}

#[test]
fn should_nest_contents_correctly() {
    // setup dirs
    let sys_root = TempDir::default();
    let home_root = sys_root.join("home");
    let repo_root = home_root.join("dotfiles");
    let home_nested_dir = home_root.join(".config/some/nested/dir");

    // create dirs
    let _ = create_dir_all(&repo_root);
    let _ = create_dir_all(&home_nested_dir);

    // create tracked_file
    let file_path = home_nested_dir.join("some_file.txt");
    File::create(file_path).unwrap();

    let config_file_path = &repo_root.join("dott.config");
    let mut config_file = File::create(config_file_path).unwrap();
    writeln!(&mut config_file, ".config/some/nested/dir/some_file.txt").unwrap();

    let mut cmd = Command::cargo_bin("dott").unwrap();
    let assert = cmd
        .current_dir(&repo_root)
        .arg("refresh")
        .arg("-y")
        .env("HOME", home_root.to_str().unwrap())
        .assert();
    assert.success().code(0);

    assert!(repo_root
        .join(".config/some/nested/dir/some_file.txt")
        .exists());
}
