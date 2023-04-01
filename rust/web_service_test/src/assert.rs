use super::test::TestCase;
use aws_lambda_events::encodings::Body;
use regex::Regex;
use serde_json::Value;

pub fn body_matches_with_replacement(test: &TestCase, actual_body_text: &String) -> String {
    if let Some(regex) = &test.regex {
        let compiled_regex = Regex::new(regex).unwrap();
        if let Some(r#match) = compiled_regex.find(actual_body_text) {
            let replaced_text = compiled_regex
                .replace_all(actual_body_text, test.placeholder.as_ref().unwrap())
                .to_string();
            assert_body_matches(test, &replaced_text);
            return r#match.as_str().to_owned();
        }
    }

    assert_body_matches(test, actual_body_text);
    String::new()
}

#[allow(clippy::option_if_let_else)]
fn assert_body_matches(test: &TestCase, actual_body_text: &String) {
    match &test.expected_response.body {
        Some(expected_body) => match expected_body {
            Body::Text(expected_body_text) => {
                assert_eq!(actual_body_text, expected_body_text);
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
                assert_eq!(actual_body_text, &String::new());
            }
        },
    }
}
