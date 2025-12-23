// src/utils.rs
use std::process::Command;

pub fn open_with_default(path: &str) {
    let _ = Command::new("cmd")
        .args(["/C", "start", "", path])
        .spawn();
}
