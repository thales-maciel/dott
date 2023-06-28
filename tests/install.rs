extern crate temp_testdir;

use assert_cmd::Command;
use std::fs::{create_dir_all, read_to_string, File};
use std::io::Write;
use temp_testdir::TempDir;

mod fixtures;
use fixtures::get_test_paths;

#[test]
fn should_not_install_on_empty_config() {
    let sys_root = TempDir::default();
    let (home, repo, config) = get_test_paths(&sys_root);

    File::create(&config).unwrap();

    let mut cmd = Command::cargo_bin("dott").unwrap();
    let assert = cmd
        .current_dir(&repo)
        .arg("install")
        .arg("-y")
        .env("HOME", home.to_str().unwrap())
        .assert();
    assert.success().code(0);
}

#[test]
fn should_install_missing_files() {
    let sys_root = TempDir::default();
    let (home, repo, config) = get_test_paths(&sys_root);

    // create tracked_file
    let file_path = &repo.join("some_file.txt");
    let mut file = File::create(file_path).unwrap();
    file.write_all(b"some content").unwrap();

    let mut config_file = File::create(config).unwrap();
    writeln!(&mut config_file, "some_file.txt").unwrap();

    let mut cmd = Command::cargo_bin("dott").unwrap();
    let assert = cmd
        .current_dir(&repo)
        .arg("install")
        .arg("-y")
        .env("HOME", &home.to_str().unwrap())
        .assert();
    assert.success().code(0);

    let synced_file = home.join("some_file.txt");
    assert!(synced_file.exists());
    assert_eq!(
        read_to_string(synced_file).unwrap(),
        "some content".to_string()
    );
}

#[test]
fn should_overwrite_home_content() {
    let sys_root = TempDir::default();
    let (home, repo, config) = get_test_paths(&sys_root);

    // create tracked_file
    let repo_file_path = repo.join("some_file.txt");
    let mut file = File::create(&repo_file_path).unwrap();
    file.write_all(b"updated content").unwrap();

    let home_file_path = home.join("some_file.txt");
    let mut file = File::create(&home_file_path).unwrap();
    file.write_all(b"legacy content").unwrap();

    let mut config_file = File::create(config).unwrap();
    writeln!(&mut config_file, "some_file.txt").unwrap();

    let mut cmd = Command::cargo_bin("dott").unwrap();
    let assert = cmd
        .current_dir(repo)
        .arg("install")
        .arg("-y")
        .env("HOME", home.to_str().unwrap())
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
    let (home, repo, config) = get_test_paths(&sys_root);
    let repo_nested_dir = repo.join(".config/some/nested/dir");

    // create dirs
    let _ = create_dir_all(&repo_nested_dir);

    // create tracked_file
    let file_path = repo_nested_dir.join("some_file.txt");
    File::create(file_path).unwrap();

    let mut config_file = File::create(config).unwrap();
    writeln!(&mut config_file, ".config/some/nested/dir/some_file.txt").unwrap();

    let mut cmd = Command::cargo_bin("dott").unwrap();
    let assert = cmd
        .current_dir(repo)
        .arg("install")
        .arg("-y")
        .env("HOME", home.to_str().unwrap())
        .assert();
    assert.success().code(0);

    assert!(home.join(".config/some/nested/dir/some_file.txt").exists());
}
