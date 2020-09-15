use clap::{App, Arg, SubCommand};
use jira::model;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("CLI Jira Interface")
        .subcommand(
            SubCommand::with_name("create")
                .about("Create issues")
                .arg(
                    Arg::with_name("title")
                        .long("title")
                        .short("t")
                        .help("Issue title")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("description")
                        .long("description")
                        .short("d")
                        .takes_value(true)
                        .help("Issue description"),
                )
                .arg(
                    Arg::with_name("issuetype")
                        .long("issue-type")
                        .short("p")
                        .default_value("Task")
                        .takes_value(true)
                        .help("Issue type"),
                )
                .arg(
                    Arg::with_name("labels")
                        .short("l")
                        .multiple(true)
                        .takes_value(true)
                        .help("Issue labels (use multiple times)"),
                ),
        )
        .get_matches();

    let token = env::var("JIRA_TOKEN").expect("A `JIRA_TOKEN` is required");

    match matches.subcommand() {
        ("create", Some(c)) => {
            // TODO: Don't hardcode these IDs

            // 10005: HEAP project
            // 10003: Capture components
            // Issue Types: bug (10004), story (10001), task (10002), subtask (10003)

            let issue = model::Issue {
                summary: c.value_of("title").unwrap().to_owned(),
                description: c.value_of("description").map(String::from).map(jira::text_to_document),
                labels: match c.values_of("labels") {
                    Some(l) => l.map(String::from).collect(),
                    None => Vec::new(),
                },
                issuetype: model::IssueType{name: String::from(c.value_of("issuetype").unwrap())},
                components: vec![model::Component {
                    id: String::from("10003"),
                }],
                project: model::Project{key: String::from("HEAP")}
            };

            jira::create_issue(issue, &token).await?;
        }
        _ => panic!("Invalid subcommand"),
    }

    Ok(())
}
