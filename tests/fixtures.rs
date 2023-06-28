extern crate temp_testdir;

use std::fs::create_dir_all;
use std::path::PathBuf;
use temp_testdir::TempDir;

pub fn get_test_paths(sys_root: &TempDir) -> (PathBuf, PathBuf, PathBuf) {
    let home = sys_root.join("home");
    let repo = home.join("repo");
    let config = repo.join("dott.config");
    create_dir_all(&repo).unwrap();
    return (home, repo, config);
}
