use aws_lambda_events::apigw::ApiGatewayV2httpRequest;
use aws_lambda_events::apigw::ApiGatewayV2httpResponse;
use aws_lambda_events::encodings::Body;
use regex::Regex;
use serde::Deserialize;
use serde_json::{json, Value};
use std::env;
use std::fs::File;

type TestCases = Vec<TestCase>;

const DB_URL: &str = "http://localhost:8000";

#[derive(Deserialize)]
struct TestCase {
    request: ApiGatewayV2httpRequest,
    request_body_json: Option<Value>,
    expected_response: ApiGatewayV2httpResponse,
    expected_body_json: Option<Value>,
    regex: Option<String>,
    placeholder: Option<String>,
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let test_target_url = &args[1];
    if test_target_url.contains("localhost") {
        let db_client = rust_lambda::dynamodb::get_local_client(DB_URL.to_owned()).await;
        rust_lambda::dynamodb::create_table_if_not_exists(&db_client).await;
    }

    let paths = std::fs::read_dir("./test-cases").unwrap();

    let http_client = reqwest::Client::new();

    for path in paths {
        let path_value = path.unwrap().path();
        println!("Testing: {}", path_value.display());
        let file = File::open(path_value).unwrap();
        let mut file_deserializer = serde_json::Deserializer::from_reader(file);
        let test_cases = TestCases::deserialize(&mut file_deserializer).unwrap();
        let mut last_regex_capture = String::new();
        for mut test in test_cases {
            match &test.request_body_json {
                Some(body) => {
                    test.request.body = Some(serde_json::to_string(body).unwrap());
                }
                None => {}
            }

            let request = &test.request;
            let actual_response: reqwest::Response;
            let mut url = format!("{test_target_url}{}", &request.raw_path.as_ref().unwrap());
            if &request.request_context.http.method == "POST" {
                println!("{} {}", &request.request_context.http.method, &url);
                let mut request_body = json!(&test.request_body_json).to_string();
                if let Some(placeholder) = &test.placeholder {
                    request_body = request_body
                        .to_string()
                        .replace(placeholder, last_regex_capture.as_str());
                }
                actual_response = http_client
                    .post(url)
                    .body(request_body)
                    .headers(request.headers.to_owned())
                    .send()
                    .await
                    .unwrap();
            } else {
                if let Some(placeholder) = &test.placeholder {
                    url = url.replace(placeholder, last_regex_capture.as_str());
                }
                let query_string = &request.raw_query_string;
                if query_string.is_some() {
                    url = format!("{}?{}", url, query_string.as_ref().unwrap());
                }
                println!("{} {}", &request.request_context.http.method, &url);
                actual_response = http_client.get(url).send().await.unwrap();
            }
            assert_eq!(
                actual_response.status(),
                test.expected_response.status_code as u16
            );
            assert_eq!(
                actual_response.headers().get("content-type"),
                test.expected_response.headers.get("content-type")
            );

            let actual_body_text = actual_response.text().await.unwrap();
            last_regex_capture = assert_body_matches_with_replacement(&test, &actual_body_text);
        }
    }
}

fn assert_body_matches_with_replacement(test: &TestCase, actual_body_text: &String) -> String {
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

fn assert_body_matches(test: &TestCase, actual_body_text: &String) {
    match &test.expected_response.body {
        Some(expected_body) => match expected_body {
            Body::Text(expected_body_text) => {
                assert_eq!(actual_body_text, expected_body_text);
                return;
            }
            _ => {
                assert!(false)
            }
        },
        None => match &test.expected_body_json {
            Some(expected_body_value) => {
                let actual_body_value: Value = serde_json::from_str(actual_body_text).unwrap();
                assert_eq!(&actual_body_value, expected_body_value);
                return;
            }
            None => {
                assert!(false);
            }
        },
    }

    assert!(test.expected_response.body.is_none());
    assert!(test.expected_body_json.is_none());
}
