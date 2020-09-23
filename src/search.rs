use reqwest::{Method, StatusCode};
use serde::{Deserialize};

use std::error::Error;

use super::{ApiConfig,ApiError};
use super::model;

#[derive(Deserialize, Debug, Default)]
struct IssueSearchResponse {
    total: usize,
    issues: Vec<model::IssueSearchResult>,
}

pub async fn issues_assigned_to_me(
    config: &ApiConfig,
) -> Result<Vec<model::IssueSearchResult>, Box<dyn Error>> {
    let search_jql = "assignee = currentUser() AND (status != Closed AND status != Done)";

    // TODO: reuse client
    let request = super::build_request("/search", Method::GET, &config).query(&[
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
