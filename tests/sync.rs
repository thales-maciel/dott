extern crate temp_testdir;

use std::fs::{File, create_dir_all, read_to_string};
use std::io::Write;
use assert_cmd::Command;
use temp_testdir::TempDir;

#[test]
fn should_not_sync_on_empty_config() {
    // setup dirs
    let sys_root = TempDir::default();
    let home_root = sys_root.join("home");
    let data_root = home_root.join(".local/share");
    let config_root = home_root.join(".config");
    let dotr_config_path = config_root.join("dotr");

    // create dirs
    let _ = create_dir_all(&data_root);
    let _ = create_dir_all(&dotr_config_path);

    let config_file_path = &dotr_config_path.join("dotr.config");
    File::create(config_file_path).unwrap();

    let mut cmd = Command::cargo_bin("dotr").unwrap();
    let assert = cmd
        .arg("sync")
        .env("HOME", home_root.to_str().unwrap())
        .env("XDG_CONFIG_HOME", config_root.to_str().unwrap())
        .env("XDG_DATA_HOME", data_root.to_str().unwrap())
        .assert();
    assert.success().code(0);

    let repo_dir = data_root.join("dotr");
    let mut repo_dir_contents = repo_dir.read_dir().unwrap();
    assert!(repo_dir.exists());
    assert!(repo_dir_contents.next().is_none());
}

#[test]
fn should_add_missing_files() {
    // setup dirs
    let sys_root = TempDir::default();
    let home_root = sys_root.join("home");
    let data_root = home_root.join(".local/share");
    let config_root = home_root.join(".config");
    let dotr_config_path = config_root.join("dotr");

    // create dirs
    let _ = create_dir_all(&data_root);
    let _ = create_dir_all(&dotr_config_path);

    // create tracked_file
    let file_path = home_root.join("some_file.txt");
    let mut file = File::create(file_path).unwrap();
    file.write_all(b"some content").unwrap();

    let config_file_path = &dotr_config_path.join("dotr.config");
    let mut config_file = File::create(config_file_path).unwrap();
    writeln!(&mut config_file, "some_file.txt").unwrap();

    let mut cmd = Command::cargo_bin("dotr").unwrap();
    let assert = cmd
        .arg("sync")
        .env("HOME", home_root.to_str().unwrap())
        .env("XDG_CONFIG_HOME", config_root.to_str().unwrap())
        .env("XDG_DATA_HOME", data_root.to_str().unwrap())
        .assert();
    assert.success().code(0);

    let synced_file = data_root.join("dotr/some_file.txt");
    assert!(synced_file.exists());
    assert_eq!(read_to_string(synced_file).unwrap(), "some content".to_string());
}

#[test]
fn should_overwrite_file_content() {
    // setup dirs
    let sys_root = TempDir::default();
    let home_root = sys_root.join("home");
    let data_root = home_root.join(".local/share");
    let repo_root = data_root.join("dotr");
    let config_root = home_root.join(".config");
    let dotr_config_path = config_root.join("dotr");

    // create dirs
    let _ = create_dir_all(&repo_root);
    let _ = create_dir_all(&dotr_config_path);

    // create tracked_file
    let file_path = home_root.join("some_file.txt");
    let mut file = File::create(file_path).unwrap();
    file.write_all(b"updated content").unwrap();
    
    let repo_file_path = repo_root.join("some_file.txt");
    let mut file = File::create(&repo_file_path).unwrap();
    file.write_all(b"legacy content").unwrap();

    let config_file_path = &dotr_config_path.join("dotr.config");
    let mut config_file = File::create(config_file_path).unwrap();
    writeln!(&mut config_file, "some_file.txt").unwrap();

    let mut cmd = Command::cargo_bin("dotr").unwrap();
    let assert = cmd
        .arg("sync")
        .env("HOME", home_root.to_str().unwrap())
        .env("XDG_CONFIG_HOME", config_root.to_str().unwrap())
        .env("XDG_DATA_HOME", data_root.to_str().unwrap())
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
    let data_root = home_root.join(".local/share");
    let repo_root = data_root.join("dotr");
    let config_root = home_root.join(".config");
    let dotr_config_path = config_root.join("dotr");

    // create dirs
    let _ = create_dir_all(&repo_root);
    let _ = create_dir_all(&dotr_config_path);

    // create tracked_file
    let file_path = home_root.join("some_file.txt");
    File::create(file_path).unwrap();
    
    let repo_file_path = repo_root.join("another_file.txt");
    File::create(&repo_file_path).unwrap();

    let config_file_path = &dotr_config_path.join("dotr.config");
    let mut config_file = File::create(config_file_path).unwrap();
    writeln!(&mut config_file, "some_file.txt").unwrap();

    let mut cmd = Command::cargo_bin("dotr").unwrap();
    let assert = cmd
        .arg("sync")
        .env("HOME", home_root.to_str().unwrap())
        .env("XDG_CONFIG_HOME", config_root.to_str().unwrap())
        .env("XDG_DATA_HOME", data_root.to_str().unwrap())
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
    let data_root = home_root.join(".local/share");
    let repo_root = data_root.join("dotr");
    let config_root = home_root.join(".config");
    let dotr_config_path = config_root.join("dotr");

    // create dirs
    let _ = create_dir_all(&repo_root);
    let _ = create_dir_all(&dotr_config_path);

    // create tracked_file
    let file_path = home_root.join("some_file.txt");
    File::create(file_path).unwrap();
    
    let repo_file_path = repo_root.join("another_file.txt");
    File::create(&repo_file_path).unwrap();

    let config_file_path = &dotr_config_path.join("dotr.config");
    let mut config_file = File::create(config_file_path).unwrap();
    writeln!(&mut config_file, "some_file.txt").unwrap();
    writeln!(&mut config_file, "another_file.txt").unwrap();

    let mut cmd = Command::cargo_bin("dotr").unwrap();
    let assert = cmd
        .arg("sync")
        .env("HOME", home_root.to_str().unwrap())
        .env("XDG_CONFIG_HOME", config_root.to_str().unwrap())
        .env("XDG_DATA_HOME", data_root.to_str().unwrap())
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
    let data_root = home_root.join(".local/share");
    let repo_root = data_root.join("dotr");
    let config_root = home_root.join(".config");
    let dotr_config_path = config_root.join("dotr");
    let home_nested_dir = config_root.join("some/nested/dir");

    // create dirs
    let _ = create_dir_all(&repo_root);
    let _ = create_dir_all(&dotr_config_path);
    let _ = create_dir_all(&home_nested_dir);

    // create tracked_file
    let file_path = home_nested_dir.join("some_file.txt");
    File::create(file_path).unwrap();
    
    let config_file_path = &dotr_config_path.join("dotr.config");
    let mut config_file = File::create(config_file_path).unwrap();
    writeln!(&mut config_file, ".config/some/nested/dir/some_file.txt").unwrap();

    let mut cmd = Command::cargo_bin("dotr").unwrap();
    let assert = cmd
        .arg("sync")
        .env("HOME", home_root.to_str().unwrap())
        .env("XDG_CONFIG_HOME", config_root.to_str().unwrap())
        .env("XDG_DATA_HOME", data_root.to_str().unwrap())
        .assert();
    assert.success().code(0);

    assert!(repo_root.join(".config/some/nested/dir/some_file.txt").exists());
}

