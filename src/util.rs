use super::ApiConfig;
use regex::Regex;

/// Convert an issue key to it's canonical form.
pub fn issue_lossy_to_issue_key(issue_lossy: &str, config: &ApiConfig) -> Option<String> {
    let issue_pattern = Regex::new(r"^[A-Z]+\-\d+$").unwrap();
    let partial_issue_pattern = Regex::new(r"^\d+$").unwrap();

    if issue_pattern.is_match(issue_lossy) {
        Some(issue_lossy.to_owned())
    } else if partial_issue_pattern.is_match(issue_lossy) {
        Some(format!("{}-{}", config.project, issue_lossy))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    fn config() -> ApiConfig {
        ApiConfig {
            email: "".to_owned(),
            token: "".to_owned(),
            subdomain: "abcd".to_owned(),
            project: "ABCD".to_owned(),
        }
    }

    #[test]
    fn test_issue_key_full() {
        assert_eq!(
            issue_lossy_to_issue_key("ABCD-12345", &config()).unwrap(),
            "ABCD-12345"
        )
    }

    #[test]
    fn test_issue_key_partial() {
        assert_eq!(
            issue_lossy_to_issue_key("12345", &config()).unwrap(),
            "ABCD-12345"
        )
    }

    #[test]
    fn test_issue_key_nonsense() {
        assert_eq!(issue_lossy_to_issue_key("alsdkflksaj", &config()), None)
    }
}
