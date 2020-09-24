//! This covers an undocumented/internal GraphQL API that Jira uses for it's own frontend.
//! This looks to be the only way to fetch a list of PRs associated with an issue.

use reqwest::{Client, Method, StatusCode};
use serde::{Deserialize, Serialize};
use std::fmt;
use heck::TitleCase;

use std::error::Error;

use super::{model, ApiConfig, ApiError};

static ISSUE_PR_GRAPHQL: &'static str = include_str!("../graphql/issue_prs.graphql");

#[derive(Serialize, Debug)]
struct GetIssuePrsRequestVariables {
    #[serde(rename = "issueId")]
    issue_id: String,
}

#[derive(Serialize, Debug)]
struct GetIssuePrsRequest {
    #[serde(rename = "operationName")]
    operation_name: String,
    query: String,
    variables: GetIssuePrsRequestVariables,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PullRequestStatus {
    Open,
    #[serde(rename = "DECLINED")]
    Closed,
    Merged,
}

impl fmt::Display for PullRequestStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = format!("{:?}", self);
        let s = s.to_title_case();
        write!(f, "{}", s)
    }
}

#[serde(rename_all = "camelCase")]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PullRequest {
    pub name: String,
    pub url: String,
    pub status: PullRequestStatus,
    pub last_update: String,
}

#[serde(rename_all = "camelCase")]
#[derive(Deserialize, Debug)]
struct Branch {
    name: String,
    url: String,
    pull_requests: Vec<PullRequest>,
}

#[derive(Deserialize, Debug)]
struct Repository {
    name: String,
    branches: Vec<Branch>,
}

#[derive(Deserialize, Debug)]
struct DevInfoInstanceType {
    repository: Vec<Repository>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DevInfoDetails {
    instance_types: Vec<DevInfoInstanceType>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DevInfo {
    details: DevInfoDetails,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResponseData {
    pub development_information: DevInfo,
}

#[derive(Deserialize, Debug)]
struct Response {
    data: ResponseData,
}

pub async fn get_issue_pull_requests(
    issue: &model::IssueSearchResult,
    config: &ApiConfig,
) -> Result<Option<Vec<PullRequest>>, Box<dyn Error>> {
    let request = GetIssuePrsRequest {
        operation_name: "DevDetailsDialog".to_owned(),
        // TODO: Don't copy this every time
        query: ISSUE_PR_GRAPHQL.to_owned(),
        variables: GetIssuePrsRequestVariables {
            issue_id: issue.id.to_owned(),
        },
    };

    let request = Client::new()
        .request(
            Method::POST,
            &format!("https://{}.atlassian.net/jsw/graphql", &config.subdomain),
        )
        .query(&[("operation", "DevDetailsDialog")])
        .json(&request)
        .basic_auth(&config.email, Some(&config.token));

    let response = request.send().await?;

    match response.status() {
        StatusCode::OK => {
            let result = response.json::<Response>().await?;
            let mut pull_requests: Vec<PullRequest> = Vec::new();

            // Ugh
            let instance = result
                .data
                .development_information
                .details
                .instance_types
                .first();

            if let Some(instance) = instance {
                let repo = instance.repository.first().unwrap();
                for branch in &repo.branches {
                    for pr in &branch.pull_requests {
                        // TODO: Don't clone
                        pull_requests.push(pr.clone());
                    }
                }

                Ok(Some(pull_requests))
            } else {
                Ok(None)
            }
        }
        code => Err(Box::new(ApiError::new(&format!(
            "Got a {} when attempting to fetch issue, {}",
            code,
            response.text().await?
        )))),
    }
}
