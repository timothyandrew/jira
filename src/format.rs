use super::graphql::PullRequestStatus;
use super::model::IssueStatus;
use colored::Colorize;
use prettytable::format;
use prettytable::Table;
use prettytable::{cell, row};
use regex::Regex;
use std::env;
use std::error::Error;
use std::fs;
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

// Remove all lines starting with a '#'
fn remove_commented_lines<'a>(
    lines: impl Iterator<Item = &'a str>,
) -> impl Iterator<Item = &'a str> {
    lines.filter(|&line| {
        if let Some(c) = line.chars().next() {
            c != '#'
        } else {
            false
        }
    })
}

pub async fn text_from_editor(template: &str) -> Result<Option<(String, String)>, Box<dyn Error>> {
    let editor = env::var("EDITOR").unwrap_or("nano".to_owned());

    let temp_file = NamedTempFile::new()?;
    temp_file
        .as_file()
        .write_all(template.as_bytes())
        .expect("Failed to write template to the temp file.");

    let temp_path = temp_file.into_temp_path();

    let canonical_path = fs::canonicalize(&temp_path).unwrap();
    Command::new(editor)
        .arg(&canonical_path)
        .status()
        .expect("Failed to open $EDITOR");

    let contents =
        fs::read_to_string(&temp_path).expect(&format!("Failed to read file: {:?}", temp_path));
    let contents = contents.trim().split("\n");
    let mut contents = remove_commented_lines(contents);
    let (first_line, other_lines) = (contents.next(), contents.collect::<Vec<_>>().join("\n"));

    temp_path.close()?;

    Ok(match (first_line, other_lines) {
        (None, _) => None,
        (Some(""), _) => None,
        (Some(first_line), other_lines) => {
            Some((first_line.trim().to_owned(), other_lines.trim().to_owned()))
        }
    })
}

pub fn issue_type_colored(t: IssueStatus) -> colored::ColoredString {
    let s = t.to_string();

    match t {
        IssueStatus::Closed => s.red(),
        IssueStatus::Done => s.green(),
        IssueStatus::InProgress => s.bright_cyan(),
        IssueStatus::InReview => s.truecolor(145, 115, 188).bold(),
        IssueStatus::ToDo => s.white(),
        IssueStatus::Logged => s.dimmed().italic().truecolor(180, 180, 180),
        IssueStatus::SupportTriaged => s.dimmed().italic().truecolor(180, 180, 180),
    }
}

pub fn issue_table(issue: super::model::IssueSearchResult) {
    let mut table = Table::new();
    let format = format::FormatBuilder::new()
        .column_separator('|')
        .padding(1, 1)
        .build();
    table.set_format(format);

    table.add_row(row![
        br->"Title".dimmed(),
        issue.fields.summary.bold()
    ]);

    if let Some(labels) = issue.fields.labels {
        if labels.len() > 0 {
            table.add_row(row![
                br->"Labels".dimmed(),
                labels.concat()
            ]);
        }
    }

    if let Some(status) = issue.fields.status {
        table.add_row(row![
            br->"Status".dimmed(),
            format!("{}", status)
        ]);
    }

    if let Some(assignee) = issue.fields.assignee {
        table.add_row(row![
            br->"Assignee".dimmed(),
            format!("{}", assignee.display_name)
        ]);
    }

    if let Some(components) = issue.fields.components {
        if components.len() > 0 {
            let components = components
                .iter()
                .map(|c| c.name.to_owned())
                .collect::<Vec<_>>();

            table.add_row(row![
                br->"Components".dimmed(),
                components.join(", ")
            ]);
        }
    }

    table.add_row(row![
        br->"Type".dimmed(),
        issue.fields.issuetype.name
    ]);

    if let Some(parent) = issue.fields.parent {
        table.add_row(row![
            br->"Parent".dimmed(),
            format!("{}", parent.key)
        ]);
    }

    if let Some(epic_issues) = issue.epic_issues {
        let sub_table = issues_table(epic_issues, true);
        table.add_row(row![
            br->"Epic Tickets".dimmed(),
            sub_table
        ]);
    };

    if let Some(prs) = issue.pull_requests {
        let mut pr_table = Table::new();
        let format = format::FormatBuilder::new()
            .column_separator('|')
            .padding(1, 1)
            .borders('|')
            .separators(
                &[format::LinePosition::Top, format::LinePosition::Bottom],
                format::LineSeparator::new('-', '+', '+', '+'),
            )
            .build();
        pr_table.set_format(format);

        for pr in &prs {
            let name = Regex::new(r"(\[[A-Z]+-\d+\]\s)?(.*)")
                .unwrap()
                .captures(&pr.name)
                .unwrap()
                .get(2)
                .unwrap();
            let url = Regex::new(r"github.com/[^/]*/[^/]*/pull/(\d+)")
                .unwrap()
                .captures(&pr.url)
                .unwrap()
                .get(1)
                .unwrap();

            pr_table.add_row(row![
                format!("#{}", url.as_str()).bold(),
                name.as_str().italic(),
                l->pr_status_colored(&pr.status)
            ]);
        }

        table.add_row(row![
            br->"PRs".dimmed(),
            pr_table
        ]);
    };

    table.printstd();
}

pub fn issues_table(mut issues: Vec<super::model::IssueSearchResult>, sort: bool) -> Table {
    let mut table = Table::new();
    let format = format::FormatBuilder::new()
        .column_separator('|')
        .borders('|')
        .separators(
            &[format::LinePosition::Top, format::LinePosition::Bottom],
            format::LineSeparator::new('-', '+', '+', '+'),
        )
        .padding(1, 1)
        .build();
    table.set_format(format);

    if sort {
        issues.sort_by(|x, y| {
            let x_parent = match &x.fields.parent {
                Some(parent) => format!("{:?}{}", parent.fields.as_ref().unwrap().status, parent.key),
                None => String::new(),
            };

            let y_parent = match &y.fields.parent {
                Some(parent) => format!("{:?}{}", parent.fields.as_ref().unwrap().status, parent.key),
                None => String::new(),
            };

            format!("{}{:?}{}", x_parent, x.fields.status, x.key)
                .cmp(&format!("{}{:?}{}", y_parent, y.fields.status, y.key))
        });
    }

    for issue in issues {
        let status = issue.fields.status.unwrap_or_default();
        let status = issue_type_colored(status);

        let summary = match issue.fields.parent {
            Some(_) => format!("| {}", issue.fields.summary).truecolor(180, 180, 180),
            None => issue.fields.summary.white(),
        };

        let assignee = if let Some(assignee) = issue.fields.assignee {
            assignee.display_name.to_owned().white()
        } else {
            "<none>".dimmed()
        };

        table.add_row(row![
            c->issue.fields.issuetype.name,
            bc->status,
            bc->issue.key,
            summary,
            assignee
        ]);
    }

    table
}

fn pr_status_colored(pr: &PullRequestStatus) -> colored::ColoredString {
    let s = pr.to_string();

    match pr {
        PullRequestStatus::Closed => s.red(),
        PullRequestStatus::Open => s.green(),
        PullRequestStatus::Merged => s.truecolor(186, 150, 255),
    }
}
