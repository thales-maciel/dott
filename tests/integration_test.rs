extern crate temp_testdir;

use std::fs::{File, create_dir_all, read_to_string};
use std::io::Write;
use assert_cmd::Command;
use temp_testdir::TempDir;

#[test]
fn should_add_patterns_to_config_file_when_config_file_does_not_exist() {
    // setup dirs
    let sys_root = TempDir::default();
    let home_root = sys_root.join("home");
    let data_root = home_root.join(".local/share");
    let config_root = home_root.join(".config");

    // create dirs
    let _ = create_dir_all(&data_root);
    let _ = create_dir_all(&config_root);

    // create tracked_file
    let file_path = home_root.join("some_file.txt");
    let mut file = File::create(file_path).unwrap();
    file.write_all("testing".as_bytes()).unwrap();

    let mut cmd = Command::cargo_bin("dotr").unwrap();
    let assert = cmd
        .arg("add")
        .arg("some_file.txt")
        .env("HOME", home_root.to_str().unwrap())
        .env("XDG_CONFIG_HOME", config_root.to_str().unwrap())
        .env("XDG_DATA_HOME", data_root.to_str().unwrap())
        .current_dir(&home_root)
        .assert();

    assert.success().code(0);

    let config_file_path = &config_root.join("dotr/dotr.config");
    println!("config_file_path {}", config_file_path.display());

    // assert config file exists with pattern set
    assert!(config_file_path.exists());
    assert_eq!(read_to_string(config_file_path).unwrap().lines().next(), Some("some_file.txt"));
}

#[test]
fn should_add_patterns_to_config_file_when_config_file_exists() {
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
    File::create(file_path).unwrap();

    let config_file_path = &dotr_config_path.join("dotr.config");
    let mut config_file = File::create(config_file_path).unwrap();
    let _ = writeln!(&mut config_file, "some_file.txt");

    let mut cmd = Command::cargo_bin("dotr").unwrap();
    let assert = cmd
        .arg("add")
        .arg("another_file.txt")
        .env("HOME", home_root.to_str().unwrap())
        .env("XDG_CONFIG_HOME", config_root.to_str().unwrap())
        .env("XDG_DATA_HOME", data_root.to_str().unwrap())
        .current_dir(&home_root)
        .assert();
    assert.success().code(0);

    let contents = read_to_string(&config_file_path).unwrap();
    let mut lines = contents.lines();

    assert_eq!(lines.next(), Some("some_file.txt"));
    assert_eq!(lines.next(), Some("another_file.txt"));
}

#[test]
fn should_not_add_patterns_to_config_file_twice() {
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
    File::create(file_path).unwrap();

    let config_file_path = &dotr_config_path.join("dotr.config");
    let mut config_file = File::create(config_file_path).unwrap();
    let _ = writeln!(&mut config_file, "some_file.txt");

    let mut cmd = Command::cargo_bin("dotr").unwrap();
    let assert = cmd
        .arg("add")
        .arg("some_file.txt")
        .env("HOME", home_root.to_str().unwrap())
        .env("XDG_CONFIG_HOME", config_root.to_str().unwrap())
        .env("XDG_DATA_HOME", data_root.to_str().unwrap())
        .current_dir(&home_root)
        .assert();
    assert.success().code(0);

    let contents = read_to_string(&config_file_path).unwrap();
    let mut lines = contents.lines();

    assert_eq!(lines.next(), Some("some_file.txt"));
    assert_eq!(lines.next(), None);
}

#[test]
fn should_add_patterns_to_config_file_relative_to_home() {
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
    File::create(file_path).unwrap();

    let config_file_path = &dotr_config_path.join("dotr.config");
    let mut config_file = File::create(config_file_path).unwrap();
    let _ = writeln!(&mut config_file, "some_file.txt");

    let some_dir_path = &home_root.join("some_dir");
    create_dir_all(&some_dir_path).unwrap();

    let mut cmd = Command::cargo_bin("dotr").unwrap();
    let assert = cmd
        .arg("add")
        .arg("another_file.txt")
        .env("HOME", home_root.to_str().unwrap())
        .env("XDG_CONFIG_HOME", config_root.to_str().unwrap())
        .env("XDG_DATA_HOME", data_root.to_str().unwrap())
        .current_dir(&some_dir_path)
        .assert();
    assert.success().code(0);

    let contents = read_to_string(&config_file_path).unwrap();
    let mut lines = contents.lines();

    assert_eq!(lines.next(), Some("some_file.txt"));
    assert_eq!(lines.next(), Some("some_dir/another_file.txt"));
}

#[test]
fn should_resolve_references_to_upper_directories() {
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
    File::create(file_path).unwrap();

    let config_file_path = &dotr_config_path.join("dotr.config");
    let mut config_file = File::create(config_file_path).unwrap();
    let _ = writeln!(&mut config_file, "some_file.txt");

    let some_dir_path = &home_root.join("some_dir");
    create_dir_all(&some_dir_path).unwrap();

    let mut cmd = Command::cargo_bin("dotr").unwrap();
    let assert = cmd
        .arg("add")
        .arg("../another_file.txt")
        .env("HOME", home_root.to_str().unwrap())
        .env("XDG_CONFIG_HOME", config_root.to_str().unwrap())
        .env("XDG_DATA_HOME", data_root.to_str().unwrap())
        .current_dir(&some_dir_path)
        .assert();
    assert.success().code(0);

    let contents = read_to_string(&config_file_path).unwrap();
    let mut lines = contents.lines();

    assert_eq!(lines.next(), Some("some_file.txt"));
    assert_eq!(lines.next(), Some("another_file.txt"));
}
