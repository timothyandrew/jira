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

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Document {
    pub version: isize,
    #[serde(flatten)]
    pub root: DocumentNode,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Mark {
    #[serde(rename = "type")]
    pub marktype: String,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct DocumentNode {
    #[serde(rename = "type")]
    pub doctype: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marks: Option<Vec<Mark>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<DocumentNode>>,
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
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct PullRequestMetadata {}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Issue {
    pub summary: String,
    pub project: Project,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Document>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<String>,
    pub issuetype: IssueType,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub components: Vec<Component>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<IssueStatus>,
    pub parent: Option<IssueParent>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct IssueSearchResult {
    pub id: String,
    pub key: String,
    pub fields: Issue,
    pub pull_requests: Option<Vec<super::graphql::PullRequest>>,
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
