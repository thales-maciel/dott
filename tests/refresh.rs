extern crate temp_testdir;

use assert_cmd::Command;
use std::fs::{create_dir_all, read_to_string, File};
use std::io::Write;
use temp_testdir::TempDir;

mod fixtures;
use fixtures::get_test_paths;

#[test]
fn should_not_refresh_on_empty_config() {
    let sys_root = TempDir::default();
    let (home, repo, config) = get_test_paths(&sys_root);

    File::create(config).unwrap();

    let mut cmd = Command::cargo_bin("dott").unwrap();
    let assert = cmd
        .current_dir(&repo)
        .arg("refresh")
        .arg("-y")
        .env("HOME", home.to_str().unwrap())
        .assert();
    assert.success().code(0);

    let mut repo_dir_contents = repo.read_dir().unwrap();
    assert!(repo.exists());
    assert!(repo_dir_contents.next().is_some());
    assert!(repo_dir_contents.next().is_none());
}

#[test]
fn should_add_missing_files() {
    let sys_root = TempDir::default();
    let (home, repo, config) = get_test_paths(&sys_root);

    // create tracked_file
    let file_path = home.join("some_file.txt");
    let mut file = File::create(file_path).unwrap();
    file.write_all(b"some content").unwrap();

    let mut config_file = File::create(config).unwrap();
    writeln!(&mut config_file, "some_file.txt").unwrap();

    let mut cmd = Command::cargo_bin("dott").unwrap();
    let assert = cmd
        .current_dir(&repo)
        .arg("refresh")
        .arg("-y")
        .env("HOME", home.to_str().unwrap())
        .assert();
    assert.success().code(0);

    let synced_file = repo.join("some_file.txt");
    assert!(synced_file.exists());
    assert_eq!(
        read_to_string(synced_file).unwrap(),
        "some content".to_string()
    );
}

#[test]
fn should_overwrite_file_content() {
    let sys_root = TempDir::default();
    let (home, repo, config) = get_test_paths(&sys_root);

    // create tracked_file
    let file_path = home.join("some_file.txt");
    let mut file = File::create(file_path).unwrap();
    file.write_all(b"updated content").unwrap();

    let repo_file_path = repo.join("some_file.txt");
    let mut file = File::create(&repo_file_path).unwrap();
    file.write_all(b"legacy content").unwrap();

    let mut config_file = File::create(config).unwrap();
    writeln!(&mut config_file, "some_file.txt").unwrap();

    let mut cmd = Command::cargo_bin("dott").unwrap();
    let assert = cmd
        .current_dir(&repo)
        .arg("refresh")
        .arg("-y")
        .env("HOME", home.to_str().unwrap())
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
    let sys_root = TempDir::default();
    let (home, repo, config) = get_test_paths(&sys_root);

    // create tracked_file
    let file_path = home.join("some_file.txt");
    File::create(file_path).unwrap();

    let repo_file_path = repo.join("another_file.txt");
    File::create(&repo_file_path).unwrap();

    let mut config_file = File::create(config).unwrap();
    writeln!(&mut config_file, "some_file.txt").unwrap();

    let mut cmd = Command::cargo_bin("dott").unwrap();
    let assert = cmd
        .current_dir(&repo)
        .arg("refresh")
        .arg("-y")
        .env("HOME", home.to_str().unwrap())
        .assert();
    assert.success().code(0);

    assert!(repo_file_path.exists());
    assert!(repo.join("some_file.txt").exists());
}

#[test]
fn should_remove_tracked_files_from_repo_if_they_are_missing_in_home() {
    let sys_root = TempDir::default();
    let (home, repo, config) = get_test_paths(&sys_root);

    // create tracked_file
    let file_path = home.join("some_file.txt");
    File::create(file_path).unwrap();

    let repo_file_path = repo.join("another_file.txt");
    File::create(&repo_file_path).unwrap();

    let mut config_file = File::create(config).unwrap();
    writeln!(&mut config_file, "some_file.txt").unwrap();
    writeln!(&mut config_file, "another_file.txt").unwrap();

    let mut cmd = Command::cargo_bin("dott").unwrap();
    let assert = cmd
        .current_dir(&repo)
        .arg("refresh")
        .arg("-y")
        .env("HOME", home.to_str().unwrap())
        .assert();
    assert.success().code(0);

    assert!(!repo_file_path.exists());
    assert!(repo.join("some_file.txt").exists());
}

#[test]
fn should_nest_contents_correctly() {
    let sys_root = TempDir::default();
    let (home, repo, config) = get_test_paths(&sys_root);
    let home_nested_dir = home.join(".config/some/nested/dir");

    // create dirs
    let _ = create_dir_all(&home_nested_dir);

    // create tracked_file
    let file_path = home_nested_dir.join("some_file.txt");
    File::create(file_path).unwrap();

    let mut config_file = File::create(config).unwrap();
    writeln!(&mut config_file, ".config/some/nested/dir/some_file.txt").unwrap();

    let mut cmd = Command::cargo_bin("dott").unwrap();
    let assert = cmd
        .current_dir(&repo)
        .arg("refresh")
        .arg("-y")
        .env("HOME", home.to_str().unwrap())
        .assert();
    assert.success().code(0);

    assert!(repo.join(".config/some/nested/dir/some_file.txt").exists());
}
