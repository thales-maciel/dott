extern crate temp_testdir;

use assert_cmd::Command;
use std::fs::{create_dir_all, read_to_string, File};
use std::io::Write;
use temp_testdir::TempDir;

#[test]
fn should_not_install_on_empty_config() {
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
        .current_dir(repo_root)
        .arg("install")
        .arg("-y")
        .env("HOME", home_root.to_str().unwrap())
        .assert();
    assert.success().code(0);
}

#[test]
fn should_install_missing_files() {
    let sys_root = TempDir::default();
    let home_root = sys_root.join("home");
    let repo_root = home_root.join("dotfiles");

    // create dirs
    let _ = create_dir_all(&repo_root);

    // create tracked_file
    let file_path = repo_root.join("some_file.txt");
    let mut file = File::create(file_path).unwrap();
    file.write_all(b"some content").unwrap();

    let config_file_path = &repo_root.join("dott.config");
    let mut config_file = File::create(config_file_path).unwrap();
    writeln!(&mut config_file, "some_file.txt").unwrap();

    let mut cmd = Command::cargo_bin("dott").unwrap();
    let assert = cmd
        .current_dir(repo_root)
        .arg("install")
        .arg("-y")
        .env("HOME", home_root.to_str().unwrap())
        .assert();
    assert.success().code(0);

    let synced_file = home_root.join("some_file.txt");
    assert!(synced_file.exists());
    assert_eq!(
        read_to_string(synced_file).unwrap(),
        "some content".to_string()
    );
}

#[test]
fn should_overwrite_home_content() {
    let sys_root = TempDir::default();
    let home_root = sys_root.join("home");
    let repo_root = home_root.join("dotfiles");

    // create dirs
    let _ = create_dir_all(&repo_root);

    // create tracked_file
    let repo_file_path = repo_root.join("some_file.txt");
    let mut file = File::create(&repo_file_path).unwrap();
    file.write_all(b"updated content").unwrap();

    let home_file_path = home_root.join("some_file.txt");
    let mut file = File::create(&home_file_path).unwrap();
    file.write_all(b"legacy content").unwrap();

    let config_file_path = &repo_root.join("dott.config");
    let mut config_file = File::create(config_file_path).unwrap();
    writeln!(&mut config_file, "some_file.txt").unwrap();

    let mut cmd = Command::cargo_bin("dott").unwrap();
    let assert = cmd
        .current_dir(repo_root)
        .arg("install")
        .arg("-y")
        .env("HOME", home_root.to_str().unwrap())
        .assert();
    assert.success().code(0);

    assert!(home_file_path.exists());
    assert_eq!(
        read_to_string(home_file_path).unwrap(),
        "updated content".to_string()
    );
}

#[test]
fn should_install_nested_contents() {
    let sys_root = TempDir::default();
    let home_root = sys_root.join("home");
    let repo_root = home_root.join("dotfiles");
    let repo_nested_dir = repo_root.join(".config/some/nested/dir");

    // create dirs
    let _ = create_dir_all(&repo_nested_dir);

    // create tracked_file
    let file_path = repo_nested_dir.join("some_file.txt");
    File::create(file_path).unwrap();

    let config_file_path = &repo_root.join("dott.config");
    let mut config_file = File::create(config_file_path).unwrap();
    writeln!(&mut config_file, ".config/some/nested/dir/some_file.txt").unwrap();

    let mut cmd = Command::cargo_bin("dott").unwrap();
    let assert = cmd
        .current_dir(repo_root)
        .arg("install")
        .arg("-y")
        .env("HOME", home_root.to_str().unwrap())
        .assert();
    assert.success().code(0);

    assert!(home_root
        .join(".config/some/nested/dir/some_file.txt")
        .exists());
}
