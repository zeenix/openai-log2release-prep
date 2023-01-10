use std::{collections::HashMap, env::args, process::Command};

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
    if args.len() < 3 {
        println!("Usage: {} LOCAL_REPO_DIR GITLAB_REPO", args[0]);

        return;
    }
    let local_repo = &args[1];
    let gitlab_repo = &args[2];
    // Split gitlab URL into host and project.
    let (host, project) = match gitlab_repo.split_once('/') {
        Some((host, project)) => (host, project),
        None => {
            println!("Invalid gitlab repo URL: {}", gitlab_repo);

            return;
        }
    };

    // Get all tags from gitlab using `gitlab` crate
    let client = Gitlab::new(host, "glpat-pUKXJxv88LPBAppYe6fD").unwrap();
    let tags_endpoint = Tags::builder()
        .project(project)
        .sort(SortOrder::Ascending)
        .build()
        .unwrap();
    let tags: Vec<Tag> = paged(tags_endpoint, Pagination::All)
        .query(&client)
        .unwrap();

    let mut subproject_tags = HashMap::new();
    for tag in tags {
        if tag.release.is_none() {
            // Only interested in release tags.
            continue;
        }
        let subproject = match tag.name.split('-').next() {
            Some(subproject) => subproject.to_owned(),
            None => continue,
        };
        let subproject_tags = subproject_tags.entry(subproject).or_insert(vec![]);
        subproject_tags.push(tag);
    }

    let mut training_data = vec![];
    for (subproject, tags) in subproject_tags {
        for i in 0..tags.len() {
            let range = if i == 0 {
                format!("{}", tags[i].name)
            } else {
                format!("{}..{}", tags[i - 1].name, tags[i].name)
            };

            let git_log = Command::new("git")
                .args(&[
                    "log",
                    "--no-color",
                    // <subject> <newline> <body> <newline>
                    "--pretty=format:%s%n%b%n",
                    &range,
                    &subproject,
                ])
                .current_dir(local_repo)
                .output()
                .expect("failed to execute process")
                .stdout;
            let git_log = Value::String(String::from_utf8(git_log).unwrap());
            let release_notes =
                Value::String(tags[i].release.as_ref().unwrap().description.clone());

            let json = json!({ "prompt": git_log, "completion": release_notes });

            training_data.push(json);
        }
    }

    println!("{}", serde_json::to_string_pretty(&training_data).unwrap());
}
