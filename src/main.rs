use clap::{App, AppSettings, Arg, SubCommand};
use jira::model;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("CLI Jira Interface")
        .arg(
            Arg::with_name("subdomain")
                .long("subdomain")
                .short("d")
                .help("Your atlassian.net subdomain")
                .default_value("heapinc")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("project")
                .long("project")
                .short("p")
                .help("Scope the subsequent command to this Jira project")
                .default_value("HEAP")
                .takes_value(true),
        )
        .subcommand(
            SubCommand::with_name("create")
                .about("Create Jira issues")
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
                        .help("Issue type")
                        .possible_values(&["Task", "Bug", "Story", "Sub-task"]),
                )
                .arg(
                    Arg::with_name("labels")
                        .short("l")
                        .multiple(true)
                        .takes_value(true)
                        .help("Issue labels"),
                )
                .arg(
                    Arg::with_name("components")
                        .short("c")
                        .multiple(true)
                        .takes_value(true)
                        .required(true)
                        .default_value("Capture")
                        .help("Issue components"),
                ),
        )
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .get_matches();

    let email = env::var("JIRA_EMAIL").expect("A `JIRA_EMAIL` is required");
    let token = env::var("JIRA_TOKEN").expect("A `JIRA_TOKEN` is required");
    let subdomain = matches.value_of("subdomain").unwrap();
    let project = matches.value_of("project").unwrap();

    let config = jira::ApiConfig{
        email: email.to_owned(),
        token: token.to_owned(),
        subdomain: subdomain.to_owned()
    };

    match matches.subcommand() {
        ("create", Some(c)) => {
            let issue = model::Issue {
                summary: c.value_of("title").unwrap().to_owned(),
                description: c
                    .value_of("description")
                    .map(String::from)
                    .map(jira::text_to_document),
                labels: match c.values_of("labels") {
                    Some(l) => l.map(String::from).collect(),
                    None => Vec::new(),
                },
                issuetype: model::IssueType {
                    name: String::from(c.value_of("issuetype").unwrap()),
                },
                components: c
                    .values_of("components")
                    .unwrap()
                    .map(|c| model::Component { name: c.to_owned() })
                    .collect(),
                project: model::Project {
                    key: project.to_owned(),
                },
            };

            jira::create_issue(issue, &config).await?;
        }
        _ => panic!("Invalid subcommand"),
    }

    Ok(())
}
