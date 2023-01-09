use std::{env::args, process::Command};

use gitlab::{
    api::{common::SortOrder, paged, projects::repository::tags::Tags, Pagination, Query},
    Gitlab,
};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
struct Tag {
    name: String,
    release: Option<Release>,
}

#[derive(Debug, Deserialize)]
struct Release {
    description: String,
}

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() < 2 {
        println!("Usage: {} LOCAL_REPO_DIR", args[0]);

        return;
    }
    let local_repo = &args[1];

    // Get all tags from gitlab using `gitlab` crate
    let client = Gitlab::new("gitlab.freedesktop.org", "glpat-pUKXJxv88LPBAppYe6fD").unwrap();
    let tags_endpoint = Tags::builder()
        .project("dbus/zbus")
        .sort(SortOrder::Ascending)
        .build()
        .unwrap();
    let tags: Vec<_> = paged(tags_endpoint, Pagination::All)
        .query(&client)
        .unwrap()
        .into_iter()
        // Only interested in release tags.
        .filter(|tag: &Tag| tag.release.is_some())
        .collect();

    // Two problems:
    //
    // 1. The git log output is long and contains a lot of noise.
    // 2. We're creating the `range` incorrectly by assuming the previous tag belongs to the same
    //    subproject.

    let mut training_data = vec![];
    for i in 0..tags.len() {
        let range = if i == 0 {
            format!("{}", tags[i].name)
        } else {
            format!("{}..{}", tags[i - 1].name, tags[i].name)
        };
        let components = tags[i].name.split('-').collect::<Vec<_>>();
        let sub_project = match components.first() {
            Some(sub_project) => sub_project,
            None => continue,
        };

        let git_log = Command::new("git")
            .args(&["log", "--no-color", &range, sub_project])
            .current_dir(local_repo)
            .output()
            .expect("failed to execute process")
            .stdout;
        let git_log = Value::String(String::from_utf8(git_log).unwrap());
        let release_notes = Value::String(tags[i].release.as_ref().unwrap().description.clone());

        let json = json!({ "prompt": git_log, "completion": release_notes });

        training_data.push(json);
    }

    println!("{}", serde_json::to_string_pretty(&training_data).unwrap());
}
