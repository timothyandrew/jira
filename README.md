# Jira CLI Interface

## Usage

```bash
❯ jira help
CLI Jira Interface

USAGE:
    jira [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -p, --project <project>        Scope the subsequent command to this Jira project [default: HEAP]
    -d, --subdomain <subdomain>    Your atlassian.net subdomain [default: heapinc]

SUBCOMMANDS:
    create    Create Jira issues
    help      Prints this message or the help of the given subcommand(s)

❯ jira help create
jira-create
Create Jira issues

USAGE:
    jira create [OPTIONS] -c <components>... --title <title>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c <components>...                 Issue components [default: Capture]
    -d, --description <description>    Issue description
    -p, --issue-type <issuetype>       Issue type [default: Task]  [values: Task, Bug, Story, Sub-task]
    -l <labels>...                     Issue labels
    -t, --title <title>                Issue title
```