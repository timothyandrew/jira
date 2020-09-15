pub mod model;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use reqwest::StatusCode;
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
    update: HashMap<String, String>
}

#[derive(Deserialize, Debug)]
struct CreateIssueResponse {
    id: String,
    key: String,
    #[serde(rename = "self")] 
    url: String
}

pub async fn create_issue(issue: model::Issue, token: &str) -> Result<(), Box<dyn Error>> {
    let request = CreateIssueRequest{fields: issue, update: HashMap::new()};

    // TODO: reuse client
    let response = reqwest::Client::new()
        .post("https://heapinc.atlassian.net/rest/api/3/issue")
        .basic_auth("tim@heapanalytics.com", Some(token))
        .json(&request)
        .send()
        .await?;

    //println!("{:?}", serde_json::to_string(&request));

    match response.status() {
        StatusCode::CREATED => {
            let created = response.json::<CreateIssueResponse>().await?;
            println!("https://heapinc.atlassian.net/browse/{}", created.key);
            Ok(())
        }
        code => Err(Box::new(ApiError::new(
            &format!("Got a {} when attempting to create an issue", code),
        ))),
    }
}

// TODO: Improve this so it converts markdown in `text` into Jira's document format.
pub fn text_to_document(text: String) -> model::Document {
    model::Document {
        version: 1,
        root: model::DocumentNode {
            doctype: String::from("doc"),
            content: vec![
                model::DocumentNode {
                    doctype: String::from("paragraph"),
                    content: vec![
                        model::DocumentNode {
                            doctype: String::from("text"),
                            text: Some(text),
                            ..Default::default()
                        }
                    ],
                    ..Default::default()
                }
            ],
            ..Default::default()
        } 
    }
}
