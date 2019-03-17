// Runs commands
use home;

use std::process::Command;
use std::fs::OpenOptions;
use std::path::PathBuf;

pub fn run() {
    let log_path: Result<PathBuf, &'static str> = match home::home_dir() {
        Some(path) => Ok(path.join("runner.out.log")),
        None => Err("problem determining home dir")
    };

    let file_out = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path.unwrap())
        .unwrap();

    let file_err = file_out.try_clone().unwrap();

    let my_cmd = Command::new("bash")
        .arg("/Users/tkbrigham/developer/socrata/feature-flag-monitor/bin/start")
        .stdout(file_out)
        .stderr(file_err)
        .spawn()
        .expect("failed to my_cmd");

    println!("PID = {:?}", my_cmd.id());
}
