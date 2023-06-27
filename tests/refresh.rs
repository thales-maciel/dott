extern crate temp_testdir;

use std::fs::{File, create_dir_all, read_to_string, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use assert_cmd::Command;
use temp_testdir::TempDir;

pub struct TestDirs {
    pub sys_root: TempDir,
    pub home: PathBuf,
    pub repo: PathBuf,
    pub config: PathBuf,
}

impl TestDirs {
    pub fn setup() -> TestDirs {
        // setup dirs
        let sys_root = TempDir::default();
        let home_root = sys_root.join("home");
        let repo_root = home_root.join("dotfiles");

        // create dirs
        let _ = create_dir_all(&repo_root);
        
        // create config file
        let config_file_path = &repo_root.join("dott.config");
        File::create(&config_file_path).unwrap();

        TestDirs {
            sys_root,
            home: home_root,
            repo: repo_root,
            config: config_file_path.clone(),
        }
    }

    pub fn write_to_config(&self, contents: &str) {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&self.config)
            .unwrap();
        // make sure that we write a new line at the end
        file.write_all(contents.as_bytes()).unwrap();
        file.write_all("\n".as_bytes()).unwrap();
    }
}

#[test]
fn should_not_refresh_on_empty_config() {
    let dirs = TestDirs::setup();

    let mut cmd = Command::cargo_bin("dott").unwrap();
    let assert = cmd
        .current_dir(&dirs.repo)
        .arg("refresh")
        .arg("-y")
        .env("HOME", dirs.home.to_str().unwrap())
        .assert();
    assert.success().code(0);

    let mut repo_dir_contents = dirs.repo.read_dir().unwrap();
    assert!(dirs.repo.exists());
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
    assert_eq!(read_to_string(synced_file).unwrap(), "some content".to_string());
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
    assert_eq!(read_to_string(repo_file_path).unwrap(), "updated content".to_string());
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
    assert_eq!(repo_files.next().unwrap().unwrap().path(), repo_root.join("some_file.txt"));
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
    assert_eq!(repo_files.next().unwrap().unwrap().path(), repo_root.join("some_file.txt"));
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

    assert!(repo_root.join(".config/some/nested/dir/some_file.txt").exists());
}

