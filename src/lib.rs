pub mod convert;
pub mod format;
pub mod graphql;
pub mod model;
pub mod search;
pub mod util;

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
    transition: model::IssueTransition,
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
    )
    .json(&request);

    let response = request.send().await?;

    match response.status() {
        StatusCode::NO_CONTENT => Ok(()),
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

pub async fn get_issue(
    issue_key: &str,
    config: &ApiConfig,
) -> Result<model::IssueSearchResult, Box<dyn Error>> {
    let result = get_issue_simple(issue_key, config).await?;

    // Enrich issue with PRs
    let pull_requests = graphql::get_issue_pull_requests(&result, config).await?;
    let result = model::IssueSearchResult {
        pull_requests,
        ..result
    };

    // Enrich issue with subtasks
    let subtasks = search::issue_subtasks(config, issue_key).await?;
    let subtasks = Some(subtasks);
    let result = model::IssueSearchResult { subtasks, ..result };

    // Enrich issue with child issues if this issue is an epic
    let result = if result.fields.issuetype.name == "Epic" {
        let epic_issues = Some(search::epic_issues(&config, &result).await?);
        model::IssueSearchResult {
            epic_issues,
            ..result
        }
    } else {
        result
    };

    // Enrich issue with parent epic if this is part of an epic
    let result = if let Some(epic) = &result.fields.epic {
        match epic {
            model::IssueEpic::Key(k) => model::IssueSearchResult {
                fields: model::Issue {
                    epic: Some(model::IssueEpic::Full(Box::new(
                        get_issue_simple(&k, &config).await?,
                    ))),
                    ..result.fields
                },
                ..result
            },
            model::IssueEpic::Full(_) => result,
        }
    } else {
        result
    };

    Ok(result)
}

async fn get_issue_simple(
    issue_key: &str,
    config: &ApiConfig,
) -> Result<model::IssueSearchResult, Box<dyn Error>> {
    let request = build_request(&format!("/issue/{}", issue_key), Method::GET, &config);
    let response = request.send().await?;

    match response.status() {
        StatusCode::OK => {
            // println!("{}", response.text().await?);
            // std::process::exit(0);
            let result = response.json::<model::IssueSearchResult>().await?;
            Ok(result)
        }
        code => Err(Box::new(ApiError::new(&format!(
            "Got a {} when attempting to fetch issue, {}",
            code,
            response.text().await?
        )))),
    }
}
