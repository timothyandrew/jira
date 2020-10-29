use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use colored::*;
use jira::model;
use std::env;
use std::error::Error;

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

    let issue_type = args.value_of("issuetype").unwrap();
    let epic = args.value_of("epic").map(|e| {
        let epic = jira::util::issue_lossy_to_issue_key(e, &config);
        let epic = epic.expect("Invalid epic key!");
        String::from(epic)
    });

    if let Some(_) = epic {
        if !(["Task", "Bug", "Story"].contains(&issue_type)) {
            panic!("Can't create a {} under an epic!", issue_type);
        }
    }

    let issue = model::Issue {
        summary: title,
        description: Some(jira::text_to_document(description)),
        labels: match args.values_of("labels") {
            Some(l) => Some(l.map(String::from).collect()),
            None => None,
        },
        issuetype: model::IssueType {
            name: String::from(issue_type),
        },
        components: Some(
            args.values_of("components")
                .unwrap()
                .map(|c| model::Component { name: c.to_owned() })
                .collect(),
        ),
        epic,
        project: Some(model::Project {
            key: config.project.to_owned(),
        }),
        ..model::Issue::default()
    };

    let issue = match args.value_of("parent") {
        Some(parent) => model::Issue {
            parent: Some(model::IssueParent {
                key: jira::util::issue_lossy_to_issue_key(parent, config)
                    .expect("Invalid parent key!")
                    .to_owned(),
                ..Default::default()
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
    let issue_key = jira::util::issue_lossy_to_issue_key(issue_key, &config);
    let issue_key = issue_key.expect("Invalid issue key!");

    let transition = args.value_of("transition").unwrap();

    jira::update_issue_status(&issue_key, transition.into(), config).await?;

    Ok(())
}

async fn subcommand_take(
    args: &ArgMatches<'_>,
    config: &jira::ApiConfig,
) -> Result<(), Box<dyn Error>> {
    let issue_key = args.value_of("issue").unwrap();
    let issue_key = jira::util::issue_lossy_to_issue_key(issue_key, &config);
    let issue_key = issue_key.expect("Invalid issue key!");

    jira::assign_issue_to_myself(&issue_key, &config).await?;
    Ok(())
}

async fn subcommand_show(
    args: &ArgMatches<'_>,
    config: &jira::ApiConfig,
) -> Result<(), Box<dyn Error>> {
    let issue_key = args.value_of("issue").unwrap();
    let issue_key = jira::util::issue_lossy_to_issue_key(issue_key, &config);
    let issue_key = issue_key.expect("Invalid issue key!");

    let result = jira::get_issue(&issue_key, config).await?;
    jira::format::issue_table(result);
    Ok(())
}

async fn subcommand_open(
    args: &ArgMatches<'_>,
    config: &jira::ApiConfig,
) -> Result<(), Box<dyn Error>> {
    let issue_key = args.value_of("issue").unwrap();
    let issue_key = jira::util::issue_lossy_to_issue_key(issue_key, &config);
    let issue_key = issue_key.expect("Invalid issue key!");

    open::that(format!(
        "https://{}.atlassian.net/browse/{}",
        config.subdomain, issue_key
    ))
    .unwrap();

    Ok(())
}

async fn subcommand_list(
    args: &ArgMatches<'_>,
    config: &jira::ApiConfig,
) -> Result<(), Box<dyn Error>> {
    match args.subcommand() {
        ("backlog", Some(_)) => {
            println!("{}", "Issues in the backlog".yellow());
            let results = jira::search::backlog_issues(&config).await?;
            let table = jira::format::issues_table(results, true);
            table.printstd();
        }
        ("epics", Some(_)) => {
            println!("{}", "Epics".yellow());
            let results = jira::search::epics(&config).await?;
            let table = jira::format::issues_table(results, false);
            table.printstd();
        }
        ("me", Some(_)) => {
            println!("{}", "Issues assigned to me".green());
            let results = jira::search::issues_assigned_to_me(&config).await?;
            let table = jira::format::issues_table(results, true);
            table.printstd();
        }
        ("sprint", Some(_)) => {
            println!("{}", "Issues in the current sprint".blue());
            let results = jira::search::sprint_issues(&config).await?;
            let table = jira::format::issues_table(results, true);
            table.printstd();
        }
        _ => {
            println!("{}", "Issues assigned to me".green());
            let results = jira::search::issues_assigned_to_me(&config).await?;
            let table = jira::format::issues_table(results, true);
            table.printstd();
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
                    Arg::with_name("epic")
                        .long("epic")
                        .short("e")
                        .help("Epic that this task belongs to")
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
                        .possible_values(&["Task", "Bug", "Story", "Sub-task", "Epic"]),
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
                    SubCommand::with_name("epics")
                        .alias("e")
                        .help("List all epics"),
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
                        .index(1)
                        .value_name("ISSUE_KEY")
                        .takes_value(true)
                        .required(true)
                        .help("The issue (key, with or without the project prefix) to assign to yourself"),
                ),
        )
        .subcommand(
            SubCommand::with_name("open")
                .alias("o")
                .about("Open an issue in your default browser")
                .arg(
                    Arg::with_name("issue")
                        .index(1)
                        .value_name("ISSUE_KEY")
                        .takes_value(true)
                        .required(true)
                        .help("The issue (key, with or without the project prefix) to open"),
                ),
        )
        .subcommand(
            SubCommand::with_name("show")
                .alias("s")
                .about("View a single issue")
                .arg(
                    Arg::with_name("issue")
                        .index(1)
                        .takes_value(true)
                        .required(true)
                        .value_name("ISSUE_KEY")
                        .help("The issue (key, with or without the project prefix) to show details for"),
                ),
        )
        .subcommand(
            SubCommand::with_name("transition")
                .alias("t")
                .about("Change/transition issue status")
                .arg(
                    Arg::with_name("issue")
                        .index(1)
                        .takes_value(true)
                        .value_name("ISSUE_KEY")
                        .required(true)
                        .help("The issue (key, with or without the project prefix) to transition"),
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
        ("show", Some(args)) => subcommand_show(&args, &config).await?,
        ("open", Some(args)) => subcommand_open(&args, &config).await?,
        _ => panic!("Invalid subcommand"),
    }

    Ok(())
}
