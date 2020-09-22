pub mod format;
pub mod model;

use reqwest::{Client, Method, RequestBuilder, StatusCode};
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
struct AssignIssueRequest {
    #[serde(rename = "accountId")]
    account_id: String,
}

#[derive(Serialize, Debug)]
struct TransitionIssueRequest {
    transition: model::IssueTransition
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

fn build_request(path: &str, method: Method, config: &ApiConfig) -> RequestBuilder {
    Client::new()
        .request(
            method,
            &format!(
                "https://{}.atlassian.net/rest/api/3/{}",
                &config.subdomain, path
            ),
        )
        .basic_auth(&config.email, Some(&config.token))
}

pub async fn update_issue_status(
    issue_key: &str,
    transition: model::IssueTransition,
    config: &ApiConfig,
) -> Result<(), Box<dyn Error>> {
    let request = TransitionIssueRequest { transition };

    let request = build_request(
        &format!("/issue/{}/transitions", issue_key),
        Method::POST,
        &config,
    ).json(&request);

    let response = request.send().await?;

    match response.status() {
        StatusCode::NO_CONTENT => {
            Ok(())
        }
        code => Err(Box::new(ApiError::new(&format!(
            "Got a {} when attempting to transition issue status, {}",
            code,
            response.text().await?
        )))),
    }
}

pub async fn get_myself(config: &ApiConfig) -> Result<model::User, Box<dyn Error>> {
    let request = build_request("/myself", Method::GET, &config);
    let response = request.send().await?;

    match response.status() {
        StatusCode::OK => {
            let user = response.json::<model::User>().await?;
            Ok(user)
        }
        code => Err(Box::new(ApiError::new(&format!(
            "Got a {} when attempting to fetch myself, {}",
            code,
            response.text().await?
        )))),
    }
}

pub async fn assign_issue_to_myself(
    issue_key: &str,
    config: &ApiConfig,
) -> Result<(), Box<dyn Error>> {
    let user = get_myself(&config).await?;

    let request = AssignIssueRequest {
        account_id: user.account_id,
    };

    let request = build_request(
        &format!("/issue/{}/assignee", issue_key),
        Method::PUT,
        &config,
    )
    .json(&request);

    let response = request.send().await?;

    match response.status() {
        StatusCode::NO_CONTENT => Ok(()),
        code => Err(Box::new(ApiError::new(&format!(
            "Got a {} when attempting to assign issue to myself, {}",
            code,
            response.text().await?
        )))),
    }
}

pub async fn create_issue(issue: model::Issue, config: &ApiConfig) -> Result<(), Box<dyn Error>> {
    let request = CreateIssueRequest {
        fields: issue,
        update: HashMap::new(),
    };

    let request = build_request("/issue", Method::POST, &config).json(&request);
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
    let request = build_request("/search", Method::GET, &config).query(&[
        ("jql", &search_jql[..]),
        (
            "fields",
            "labels,components,issuetype,summary,status,project,parent",
        ),
    ]);

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
