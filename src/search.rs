use reqwest::{Method, StatusCode};
use serde::Deserialize;

use std::error::Error;

use super::model;
use super::{ApiConfig, ApiError};

#[derive(Deserialize, Debug, Default)]
struct IssueSearchResponse {
    total: usize,
    issues: Vec<model::IssueSearchResult>,
}

struct IssueSearchPageCounter {
    pub start_at: usize,
    pub issues_seen: usize,
}

impl Default for IssueSearchPageCounter {
    fn default() -> Self {
        Self {
            start_at: 0,
            issues_seen: 0,
        }
    }
}

pub async fn epics(config: &ApiConfig) -> Result<Vec<model::IssueSearchResult>, Box<dyn Error>> {
    let search_jql = "issuetype = Epic AND status not in (Closed, Done) AND component in (Capture,iOS,Android,Mobile) order by updated ASC";
    search_issues(search_jql, config).await
}

pub async fn issue_subtasks(
    config: &ApiConfig,
    issue_key: &str,
) -> Result<Vec<model::IssueSearchResult>, Box<dyn Error>> {
    let search_jql = format!("parent = {}", issue_key);
    search_issues(&search_jql, config).await
}

pub async fn epic_issues(
    config: &ApiConfig,
    epic: &model::IssueSearchResult,
) -> Result<Vec<model::IssueSearchResult>, Box<dyn Error>> {
    let search_jql = &format!("'Epic Link' = {}", epic.key);
    search_issues(search_jql, config).await
}

pub async fn backlog_issues(
    config: &ApiConfig,
) -> Result<Vec<model::IssueSearchResult>, Box<dyn Error>> {
    let search_jql = "sprint is empty AND component in (Capture,iOS,Android,Mobile) AND (status != Closed AND status != Done)";
    search_issues(search_jql, config).await
}

pub async fn sprint_issues(
    config: &ApiConfig,
) -> Result<Vec<model::IssueSearchResult>, Box<dyn Error>> {
    let search_jql = "sprint in openSprints () AND component in (Capture,iOS,Android,Mobile)";
    search_issues(search_jql, config).await
}

pub async fn issues_assigned_to_me(
    config: &ApiConfig,
) -> Result<Vec<model::IssueSearchResult>, Box<dyn Error>> {
    let search_jql = "assignee = currentUser() AND (status != Closed AND status != Done)";
    search_issues(search_jql, config).await
}

async fn search_issues(
    search_jql: &str,
    config: &ApiConfig,
) -> Result<Vec<model::IssueSearchResult>, Box<dyn Error>> {
    let mut start_at = 0;
    let mut results = Vec::new();

    loop {
        let mut page = search_issues_single_page(search_jql, start_at, config).await?;
        start_at = start_at + page.issues.len();
        results.append(&mut page.issues);

        if page.total > start_at {
            // Do nothing, fetch another page
            eprintln!("Fetching a page of results starting at index: {}", start_at);
        } else {
            break;
        }
    }

    Ok(results)
}

async fn search_issues_single_page(
    search_jql: &str,
    start_at: usize,
    config: &ApiConfig,
) -> Result<IssueSearchResponse, Box<dyn Error>> {
    // TODO: reuse client
    let request = super::build_request("/search", Method::GET, &config).query(&[
        ("jql", &search_jql[..]),
        ("startAt", &start_at.to_string()),
        (
            "fields",
            "assignee,labels,components,issuetype,summary,status,project,parent",
        ),
    ]);

    let response = request.send().await?;

    match response.status() {
        StatusCode::OK => {
            let results = response.json::<IssueSearchResponse>().await?;
            Ok(results)
        }
        code => Err(Box::new(ApiError::new(&format!(
            "Got a {} when attempting to list issues assigned to me, {}",
            code,
            response.text().await?
        )))),
    }
}
