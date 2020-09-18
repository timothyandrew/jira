use serde::{Deserialize, Serialize};

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

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct IssueStatus {
    pub name: String,
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
