pub mod format;
pub mod model;

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
struct ApiError {
    message: String,
}

impl ApiError {
    fn new(msg: &str) -> ApiError {
        ApiError {
            message: msg.to_string(),
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for ApiError {
    fn description(&self) -> &str {
        &self.message
    }
}

#[derive(Serialize, Debug)]
struct CreateIssueRequest {
    fields: model::Issue,
    update: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
struct CreateIssueResponse {
    id: String,
    key: String,
    #[serde(rename = "self")]
    url: String,
}

#[derive(Deserialize, Debug, Default)]
struct IssueSearchResponse {
    total: usize,
    issues: Vec<model::IssueSearchResult>,
}

pub struct ApiConfig {
    pub email: String,
    pub token: String,
    pub subdomain: String,
    pub project: String,
}

pub async fn create_issue(issue: model::Issue, config: &ApiConfig) -> Result<(), Box<dyn Error>> {
    let request = CreateIssueRequest {
        fields: issue,
        update: HashMap::new(),
    };

    // TODO: reuse client
    let request = reqwest::Client::new()
        .post(&format!(
            "https://{}.atlassian.net/rest/api/3/issue",
            &config.subdomain
        ))
        .basic_auth(&config.email, Some(&config.token))
        .json(&request);

    let response = request.send().await?;

    match response.status() {
        StatusCode::CREATED => {
            let created = response.json::<CreateIssueResponse>().await?;
            println!(
                "https://{}.atlassian.net/browse/{}",
                config.subdomain, created.key
            );
            Ok(())
        }
        code => Err(Box::new(ApiError::new(&format!(
            "Got a {} when attempting to create an issue, {}",
            code,
            response.text().await?
        )))),
    }
}

pub async fn issues_assigned_to_me(
    config: &ApiConfig,
) -> Result<Vec<model::IssueSearchResult>, Box<dyn Error>> {
    let search_jql = "assignee = currentUser() AND (status != Closed AND status != Done)";

    // TODO: reuse client
    let request = reqwest::Client::new()
        .get(&format!(
            "https://{}.atlassian.net/rest/api/3/search",
            &config.subdomain
        ))
        .query(&[
            ("jql", &search_jql[..]),
            (
                "fields",
                "labels,components,issuetype,summary,status,project",
            ),
        ])
        .basic_auth(&config.email, Some(&config.token));

    let response = request.send().await?;

    match response.status() {
        StatusCode::OK => {
            let results = response.json::<IssueSearchResponse>().await?;
            Ok(results.issues)
        }
        code => Err(Box::new(ApiError::new(&format!(
            "Got a {} when attempting to list issues assigned to me, {}",
            code,
            response.text().await?
        )))),
    }
}

// TODO: Improve this so it converts markdown in `text` into Jira's document format.
pub fn text_to_document(text: String) -> model::Document {
    model::Document {
        version: 1,
        root: model::DocumentNode {
            doctype: String::from("doc"),
            content: vec![model::DocumentNode {
                doctype: String::from("paragraph"),
                content: vec![model::DocumentNode {
                    doctype: String::from("text"),
                    text: Some(text),
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        },
    }
}
