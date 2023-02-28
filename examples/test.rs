use aws_lambda_events::apigw::{
    ApiGatewayV2httpRequest, ApiGatewayV2httpRequestContextHttpDescription,
    ApiGatewayV2httpResponse,
};
use aws_lambda_events::encodings::Body;
use base64::{engine::general_purpose, Engine as _};
use std::time::SystemTime;
use tracing::{event, Level};

use chrono::{offset::Utc, DateTime};
use http::header::{HeaderMap, HeaderValue};
use regex::Regex;
use rust_lambda::activitypub::object::Object;
use rust_lambda::activitypub::signature;
use rust_lambda::activitypub::signer::Signer;
use rust_lambda::dynamodb;
use rust_lambda::model::user::User;
use serde::Deserialize;
use serde_json::{json, Value};
use std::env;
use std::fs;

type TestCases = Vec<TestCase>;

#[derive(Deserialize)]
struct TestCase {
    name: String,
    request: ApiGatewayV2httpRequest,
    request_body_json: Option<Value>,
    expected_response: ApiGatewayV2httpResponse,
    expected_body_json: Option<Value>,
    regex: Option<String>,
    placeholder: Option<String>,
}

fn get_target(args: Vec<String>) -> Option<String> {
    if args.len() != 2 {
        return None;
    }

    Some(args[1].clone())
}

#[tokio::main]
async fn main() {
    rust_lambda::tracing::init();
    let args: Vec<String> = env::args().collect();
    let test_target_url = get_target(args)
        .expect("Usage: LOCAL_DYNAMODB_URL=http://localhost:8000 test localhost:8080");

    let signer: Option<User> = if test_target_url.contains("localhost") {
        let db_client = dynamodb::get_client().await;
        dynamodb::create_table_if_not_exists(&db_client).await;
        Some(
            rust_lambda::model::user::create(
                db_client,
                dynamodb::DEFAULT_TABLE_NAME,
                "test_username",
            )
            .await,
        )
    } else {
        None
    };

    let paths = std::fs::read_dir("./test-cases").unwrap();

    let http_client = reqwest::Client::new();

    for path in paths {
        let path_value = path.unwrap().path();
        println!("Testing: {}", path_value.display());
        let file = fs::read_to_string(path_value).unwrap();
        let file = file
            .as_str()
            .replace("example.com", test_target_url.as_str());
        let test_cases: TestCases = serde_json::from_str(&file).unwrap();
        let mut last_regex_capture = String::new();
        for mut test in test_cases {
            if let Some(body) = &test.request_body_json {
                test.request.body = Some(serde_json::to_string(body).unwrap());
            }

            let request = &mut test.request;
            let actual_response: reqwest::Response;
            let mut url = format!(
                "http://{test_target_url}{}",
                &request.raw_path.as_ref().unwrap()
            );
            if &request.request_context.http.method == "POST" {
                println!(
                    "{} {} {}",
                    &request.request_context.http.method, &url, &test.name
                );
                let mut request_body = json!(&test.request_body_json).to_string();
                if let Some(placeholder) = &test.placeholder {
                    request_body = request_body
                        .to_string()
                        .replace(placeholder, last_regex_capture.as_str());
                }

                let headers = &mut request.headers;
                if let Some(user) = &signer {
                    if let Some(_signature_header) = headers.get("signature") {
                        let digest = rust_lambda::activitypub::digest::Digest::from_body(
                            &request_body.to_string(),
                        );
                        event!(Level::DEBUG, digest = digest);
                        insert_date(headers);
                        insert_signature(user, headers, &request.request_context.http);
                    }
                }
                actual_response = http_client
                    .post(url)
                    .body(request_body)
                    .headers(headers.to_owned())
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
                println!(
                    "{} {} {}",
                    &request.request_context.http.method, &url, &test.name
                );
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
            if let Some(signer) = &signer {
                if let Some(expected_body) = &mut test.expected_body_json {
                    let expected_object: Result<Object, serde_json::Error> =
                        serde_json::from_value(expected_body.clone());
                    if let Ok(mut expected_object) = expected_object {
                        if let Some(public_key) = expected_object.public_key.as_mut() {
                            let public_key_der = signer.public_key.as_ref().unwrap();
                            public_key.public_key_pem =
                                rust_lambda::rsa::der_to_pem(public_key_der);
                            expected_object.published = Some(signer.get_published_time());
                            test.expected_body_json =
                                Some(serde_json::to_value(expected_object).unwrap());
                        }
                    }
                }
            }
            last_regex_capture = assert_body_matches_with_replacement(&test, &actual_body_text);
        }
    }
}

fn insert_date(all_headers: &mut HeaderMap) {
    let date: DateTime<Utc> = SystemTime::now().into();
    let date = format!("{}", date.format("%a, %d %b %Y %T GMT"));
    let date = HeaderValue::from_str(&date).unwrap();
    all_headers.insert("date", date);
}

fn insert_signature(
    user: &User,
    all_headers: &mut HeaderMap,
    http_description: &ApiGatewayV2httpRequestContextHttpDescription,
) {
    let signature_header = all_headers.get("signature").unwrap();
    let parsed_signature_header = signature::parse_header(signature_header.to_str().unwrap());
    let select_headers = select_headers(
        all_headers,
        parsed_signature_header.headers.unwrap().as_str(),
        http_description,
    );
    event!(Level::DEBUG, select_headers = select_headers);
    let signature = user.sign(&select_headers).unwrap();
    let signature = general_purpose::STANDARD.encode(signature);

    event!(Level::DEBUG, signature = signature);
    let signature_header = signature_header.to_str().unwrap().to_owned();
    let signature_header =
        signature_header.replace("SIGNATURE_PLACEHOLDER", format!("{}", signature).as_str());
    let signature_header = HeaderValue::from_str(signature_header.as_str()).unwrap();
    all_headers.insert("signature", signature_header);
    event!(Level::DEBUG, all_headers = format!("{:?}", all_headers));
}

fn select_headers(
    all_headers: &HeaderMap,
    query: &str,
    http_description: &ApiGatewayV2httpRequestContextHttpDescription,
) -> String {
    query
        .split_whitespace()
        .map(|header| {
            if header == "(request-target)" {
                return format!(
                    "(request-target): {} {}",
                    http_description.method.as_ref().to_lowercase(),
                    http_description.path.as_ref().unwrap()
                );
            }
            format!(
                "{}: {}",
                header.to_lowercase(),
                all_headers.get(header).unwrap().to_str().unwrap_or("")
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
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
                let actual_body_value: Value =
                    serde_json::from_str(actual_body_text).expect("expected JSON response body");
                assert_eq!(&actual_body_value, expected_body_value);
                return;
            }
            None => {
                assert_eq!(actual_body_text, &String::new());
            }
        },
    }
}
