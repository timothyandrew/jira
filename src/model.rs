use serde::{Deserialize, Serialize};
use heck::TitleCase;
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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub marks: Vec<Mark>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<DocumentNode>,
}

// TODO: This can vary based on Jira installation, so make this more dynamic
#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "name")]
pub enum IssueStatus {
    #[serde(rename = "To Do")] 
    ToDo,
    #[serde(rename = "In Progress")] 
    InProgress,
    #[serde(rename = "In Review")] 
    InReview,
    Closed,
    Done
}

impl Default for IssueStatus {
    fn default() -> Self { IssueStatus::ToDo }
}

impl fmt::Display for IssueStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = format!("{:?}", self);
        let s = s.to_title_case();
        write!(f, "{}", s)
    }
}

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
    pub status: Option<IssueStatus>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct IssueSearchResult {
    pub id: String,
    pub key: String,
    pub fields: Issue,
}
