use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use jira::model;
use std::env;
use std::error::Error;
use colored::*;

static CREATE_ISSUE_TEMPLATE: &'static str = include_str!("../template/create_issue.md");

// Why is `<'_>` required?
async fn subcommand_create(
    args: &ArgMatches<'_>,
    config: &jira::ApiConfig,
) -> Result<(), Box<dyn Error>> {
    let (title, description) = match (args.value_of("title"), args.value_of("description")) {
        (Some(t), Some(d)) => (t.to_owned(), d.to_owned()),
        (Some(t), None) => (t.to_owned(), "".to_owned()),
        (_, _) => {
            if let Some((title, description)) =
                jira::format::text_from_editor(CREATE_ISSUE_TEMPLATE).await?
            {
                (title, description)
            } else {
                panic!("Aborting: issue title wasn't provided.");
            }
        }
    };

    let issue = model::Issue {
        summary: title,
        description: Some(jira::text_to_document(description)),
        labels: match args.values_of("labels") {
            Some(l) => l.map(String::from).collect(),
            None => Vec::new(),
        },
        issuetype: model::IssueType {
            name: String::from(args.value_of("issuetype").unwrap()),
        },
        components: args
            .values_of("components")
            .unwrap()
            .map(|c| model::Component { name: c.to_owned() })
            .collect(),
        project: model::Project {
            key: config.project.to_owned(),
        },
        ..model::Issue::default()
    };

    let issue = match args.value_of("parent") {
        Some(parent) => model::Issue {
            parent: Some(model::IssueParent {
                key: parent.to_owned(),
            }),
            ..issue
        },
        None => issue,
    };

    jira::create_issue(issue, &config).await?;

    Ok(())
}

async fn subcommand_transition(
    args: &ArgMatches<'_>,
    config: &jira::ApiConfig,
) -> Result<(), Box<dyn Error>> {
    let issue_key = args.value_of("issue").unwrap();
    let transition = args.value_of("transition").unwrap();

    jira::update_issue_status(issue_key, transition.into(), config).await?;

    Ok(())
}

async fn subcommand_take(
    args: &ArgMatches<'_>,
    config: &jira::ApiConfig,
) -> Result<(), Box<dyn Error>> {
    let issue_key = args.value_of("issue").unwrap();
    jira::assign_issue_to_myself(issue_key, &config).await?;
    Ok(())
}

async fn subcommand_list(args: &ArgMatches<'_>, config: &jira::ApiConfig) -> Result<(), Box<dyn Error>> {
    match args.subcommand() {
        ("backlog", Some(_)) => {
            println!("{}", "Issues in the backlog".yellow());
            let results = jira::search::backlog_issues(&config).await?;
            jira::format::issue_table(results);
        }
        ("me", Some(_)) => {
            println!("{}", "Issues assigned to me".green());
            let results = jira::search::issues_assigned_to_me(&config).await?;
            jira::format::issue_table(results);
        }
        ("sprint", Some(_)) => {
            println!("{}", "Issues in the current sprint".blue());
            let results = jira::search::sprint_issues(&config).await?;
            jira::format::issue_table(results);
        }
        _ => {
            println!("{}", "Issues assigned to me".green());
            let results = jira::search::issues_assigned_to_me(&config).await?;
            jira::format::issue_table(results);
        }
    }

    Ok(())
}

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
                        .short("y")
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
                )
                .arg(
                    Arg::with_name("parent")
                        .long("parent")
                        .short("p")
                        .takes_value(true)
                        .required_if("issuetype", "Sub-task")
                        .help("Parent issue (if creating a sub-task)"),
                ),
        )
        .subcommand(
            SubCommand::with_name("list")
                .about("Display a summary of relevant issues. Default: list issues assigned to me.")
                .subcommand(
                    SubCommand::with_name("backlog")
                        .alias("b")
                        .help("List all issues in the backlog"),
                )
                .subcommand(
                    SubCommand::with_name("me")
                        .alias("m")
                        .help("List all issues assigned to me"),
                )
                .subcommand(
                    SubCommand::with_name("sprint")
                        .alias("s")
                        .help("List all issues in the current sprint"),
                ),
        )
        .subcommand(
            SubCommand::with_name("take")
                .about("Assign an issue to yourself")
                .arg(
                    Arg::with_name("issue")
                        .long("issue")
                        .short("i")
                        .takes_value(true)
                        .required(true)
                        .help("The issue to assign to yourself"),
                ),
        )
        .subcommand(
            SubCommand::with_name("transition")
                .about("Change/transition issue status")
                .arg(
                    Arg::with_name("issue")
                        .long("issue")
                        .short("i")
                        .takes_value(true)
                        .required(true)
                        .help("The issue to transition"),
                )
                .arg(
                    Arg::with_name("transition")
                        .long("transition-to")
                        .short("t")
                        .takes_value(true)
                        .required(true)
                        .possible_values(&["todo", "in-progress", "review", "closed", "done"])
                        .help("Status to transition the issue to"),
                ),
        )
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .get_matches();

    let email = env::var("JIRA_EMAIL").expect("A `JIRA_EMAIL` is required");
    let token = env::var("JIRA_TOKEN").expect("A `JIRA_TOKEN` is required");
    let subdomain = matches.value_of("subdomain").unwrap();
    let project = matches.value_of("project").unwrap();

    let config = jira::ApiConfig {
        email: email.to_owned(),
        token: token.to_owned(),
        subdomain: subdomain.to_owned(),
        project: project.to_owned(),
    };

    match matches.subcommand() {
        ("create", Some(args)) => subcommand_create(&args, &config).await?,
        ("list", Some(args)) => subcommand_list(&args, &config).await?,
        ("take", Some(args)) => subcommand_take(&args, &config).await?,
        ("transition", Some(args)) => subcommand_transition(&args, &config).await?,
        _ => panic!("Invalid subcommand"),
    }

    Ok(())
}
