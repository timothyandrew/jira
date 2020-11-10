use super::convert;
use heck::TitleCase;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct IssueType {
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Component {
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Project {
    pub key: String,
}

// TODO: This can vary based on Jira installation, so make this more dynamic
#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "name")]
pub enum IssueStatus {
    #[serde(rename = "To Do")]
    ToDo,
    Logged,
    #[serde(rename = "In Progress")]
    InProgress,
    #[serde(rename = "Support Triaged")]
    SupportTriaged,
    #[serde(rename = "In Review")]
    InReview,
    Closed,
    Done,
}

impl Default for IssueStatus {
    fn default() -> Self {
        IssueStatus::ToDo
    }
}

impl fmt::Display for IssueStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = format!("{:?}", self);
        let s = s.to_title_case();
        write!(f, "{}", s)
    }
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct IssueParent {
    pub key: String,
    pub fields: Option<Box<Issue>>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct IssueAssignee {
    pub display_name: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum IssueEpic {
    Key(String),
    // UGLY
    Full(Box<IssueSearchResult>),
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Issue {
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<Project>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<convert::Node>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
    pub issuetype: IssueType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Vec<Component>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<IssueStatus>,
    pub parent: Option<IssueParent>,
    pub assignee: Option<IssueAssignee>,
    #[serde(rename = "customfield_10008")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub epic: Option<IssueEpic>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct IssueSearchResult {
    pub id: String,
    pub key: String,
    pub fields: Issue,
    pub pull_requests: Option<Vec<super::graphql::PullRequest>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub epic_issues: Option<Vec<IssueSearchResult>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct User {
    #[serde(rename = "accountId")]
    pub account_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct IssueTransition {
    pub id: usize,
}

// TODO: Don't hardcode these IDs
impl From<&str> for IssueTransition {
    fn from(s: &str) -> Self {
        let id = match s {
            "todo" => 11,
            "in-progress" => 21,
            "review" => 51,
            "closed" => 41,
            "done" => 31,
            _ => panic!("Invalid status!"),
        };

        IssueTransition { id }
    }
}
