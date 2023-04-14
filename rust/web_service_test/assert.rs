use super::test::TestCase;
use aws_lambda_events::encodings::Body;
use regex::Regex;
use serde_json::Value;

fn regex_replace(input: &str, replace: &super::test::Replace) -> String {
    let re = Regex::new(replace.regex.as_str()).unwrap();
    re.replace(input, replace.placeholder.as_str()).to_string()
}

pub fn body_matches_with_replacement(test: &TestCase, actual_body_text: String) -> String {
    let mut actual_body_text = actual_body_text;
    if let Some(response_replace) = &test.response_replace {
        for replace in response_replace {
            actual_body_text = regex_replace(actual_body_text.as_str(), replace);
        }
    }

    if let Some(replace) = &test.cross_request_replace {
        let compiled_regex = Regex::new(replace.regex.as_str()).unwrap();
        if let Some(r#match) = compiled_regex.find(actual_body_text.as_str()) {
            let actual_body = regex_replace(actual_body_text.as_str(), replace);
            assert_body_matches(test, actual_body.as_str());
            return r#match.as_str().to_owned();
        }
    }

    assert_body_matches(test, actual_body_text.as_str());
    String::new()
}

#[allow(clippy::option_if_let_else)]
fn assert_body_matches(test: &TestCase, actual_body_text: &str) {
    match &test.expected_response.body {
        Some(expected_body) => match expected_body {
            Body::Text(expected_body_text) => {
                assert_eq!(&actual_body_text, expected_body_text);
            }
            _ => {
                panic!("Expected response body isn't Text")
            }
        },
        None => match &test.expected_body_json {
            Some(expected_body_value) => {
                let actual_body_value: Value =
                    serde_json::from_str(actual_body_text).expect("expected JSON response body");
                assert_eq!(&actual_body_value, expected_body_value);
            }
            None => {
                assert_eq!(actual_body_text, String::new());
            }
        },
    }
}
