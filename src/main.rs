use std::{
    env::args,
    io::{self, Read},
    process::Command,
};

use serde_json::{json, Value};

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() < 4 {
        println!("Usage: {} GIT_REPO_DIR REVISION_RANGE SUB_PROJECT", args[0]);

        return;
    }
    let repo = &args[1];
    let range = &args[2];
    let sub_project = &args[3];

    // First, we get the git log
    let git_log = Command::new("git")
        .args(&["log", "--no-color", range, sub_project])
        .current_dir(repo)
        .output()
        .expect("failed to execute process")
        .stdout;
    let git_log = Value::String(String::from_utf8(git_log).unwrap());

    // Now the release notes from stdin (multiline)
    let mut release_notes = String::new();
    let mut stdin = io::BufReader::new(io::stdin());
    stdin.read_to_string(&mut release_notes).unwrap();
    let release_notes = Value::String(release_notes);

    let json = json!({ "prompt": git_log, "completion": release_notes });

    println!("{}", json);
}
