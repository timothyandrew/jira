use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct IssueType {
    pub name: String,
}

#[derive(Serialize, Debug)]
pub struct Component {
    pub id: String,
}

#[derive(Serialize, Debug)]
pub struct Project {
    pub key: String,
}

#[derive(Serialize, Debug)]
pub struct Issue {
    pub summary: String,
    pub project: Project,
    pub description: Option<String>,
    pub labels: Vec<String>,
    pub issuetype: IssueType,
    pub components: Vec<Component>,
}
