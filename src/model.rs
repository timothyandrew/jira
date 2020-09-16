use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct IssueType {
    pub name: String,
}

#[derive(Serialize, Debug)]
pub struct Component {
    pub name: String,
}

#[derive(Serialize, Debug)]
pub struct Project {
    pub key: String,
}

#[derive(Serialize, Debug, Default)]
pub struct Document {
    pub version: isize,
    #[serde(flatten)]
    pub root: DocumentNode,
}

#[derive(Serialize, Debug, Default)]
pub struct Mark {
    #[serde(rename = "type")]
    pub marktype: String,
}

#[derive(Serialize, Debug, Default)]
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

#[derive(Serialize, Debug)]
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
}
