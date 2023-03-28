use aws_lambda_events::apigw::{
    ApiGatewayV2httpRequest, ApiGatewayV2httpRequestContextHttpDescription,
    ApiGatewayV2httpResponse,
};
use aws_lambda_events::encodings::Body;
use base64::{engine::general_purpose, Engine as _};
use time::format_description::well_known::Rfc2822;
use time::OffsetDateTime;
use tracing::{event, Level};

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

fn get_target(args: Vec<String>) -> Option<(String, String, String, String)> {
    if args.len() != 5 {
        return None;
    }

    Some((
        args[1].clone(),
        args[2].clone(),
        args[3].clone(),
        args[4].clone(),
    ))
}

#[tokio::main]
async fn main() {
    rust_lambda::tracing::init();
    let args: Vec<String> = env::args().collect();
    let (target_urn, target_username, signer_urn, signer_username) = get_target(args).expect(
        "Usage: LOCAL_DYNAMODB_URL=http://localhost:8000 test localhost:8080 target_username localhost:8080 signer_username",
    );

    let target_url = if target_urn.starts_with("localhost") {
        format!("http://{target_urn}")
    } else {
        format!("https://{target_urn}")
    };
    let signer_url = if signer_urn.starts_with("localhost") {
        format!("http://{signer_urn}")
    } else {
        format!("https://{signer_urn}")
    };

    let db_client = dynamodb::get_client().await;
    let target_user: Option<User> = if target_urn.starts_with("localhost") {
        dynamodb::create_table_if_not_exists(&db_client).await;
        Some(
            rust_lambda::model::user::create(
                &db_client,
                dynamodb::DEFAULT_TABLE_NAME,
                target_username.as_str(),
            )
            .await,
        )
    } else {
        None
    };
    let signer: User = if signer_urn.starts_with("localhost") {
        dynamodb::create_table_if_not_exists(&db_client).await;
        rust_lambda::model::user::create(
            &db_client,
            dynamodb::DEFAULT_TABLE_NAME,
            signer_username.as_str(),
        )
        .await
    } else {
        let get_item_output = rust_lambda::model::user::get_item(
            signer_username.as_str(),
            &db_client,
            dynamodb::DEFAULT_TABLE_NAME,
        )
        .await;
        let item = get_item_output.item.unwrap();
        serde_dynamo::from_item(item).unwrap()
    };

    let paths = std::fs::read_dir("./test-cases").unwrap();

    let http_client = reqwest::Client::new();

    for path in paths {
        let path_value = path.unwrap().path();
        event!(Level::INFO, "Testing {}", path_value.display());
        let file = fs::read_to_string(path_value).unwrap();
        let file = file
            .as_str()
            .replace("TARGET_URN_PLACEHOLDER", target_urn.as_str())
            .replace("TARGET_URL_PLACEHOLDER", target_url.as_str())
            .replace("TARGET_USERNAME_PLACEHOLDER", target_username.as_str())
            .replace("SIGNER_URL_PLACEHOLDER", signer_url.as_str())
            .replace("SIGNER_USERNAME_PLACEHOLDER", signer_username.as_str());
        let test_cases: TestCases = serde_json::from_str(&file).unwrap();
        let mut last_regex_capture = String::new();
        for mut test in test_cases {
            if let Some(body) = &test.request_body_json {
                test.request.body = Some(serde_json::to_string(body).unwrap());
            }

            let request = &mut test.request;
            let actual_response: reqwest::Response;
            let mut url = format!("{target_url}{}", &request.raw_path.as_ref().unwrap());
            event!(
                Level::INFO,
                method = &request.request_context.http.method.as_ref(),
                url = &url,
                query_string = &request.raw_query_string,
                test_name = &test.name
            );
            if &request.request_context.http.method == "POST" {
                let mut request_body = json!(&test.request_body_json).to_string();
                if let Some(placeholder) = &test.placeholder {
                    request_body = request_body
                        .to_string()
                        .replace(placeholder, last_regex_capture.as_str());
                }

                let headers = &mut request.headers;
                if let Some(_signature_header) = headers.get("signature") {
                    event!(Level::DEBUG, signature_header = "present");
                    insert_digest(headers, &request_body);
                    insert_date(headers);
                    insert_signature(&signer, headers, &request.request_context.http);
                }
                event!(
                    Level::DEBUG,
                    "curl -H '{}' -d '{}' {}",
                    headers
                        .clone()
                        .into_iter()
                        .map(|(key, value)| format!(
                            "{}: {}",
                            key.unwrap(),
                            value.to_str().unwrap()
                        ))
                        .collect::<Vec<_>>()
                        .join("' -H '"),
                    request_body,
                    url
                );
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
                let headers = &request.headers;
                actual_response = http_client
                    .get(url)
                    .headers(headers.clone())
                    .send()
                    .await
                    .unwrap();
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
            if let Some(target_user) = &target_user {
                if let Some(expected_body) = &mut test.expected_body_json {
                    let expected_object: Result<Object, serde_json::Error> =
                        serde_json::from_value(expected_body.clone());
                    if let Ok(mut expected_object) = expected_object {
                        if let Some(public_key) = expected_object.public_key.as_mut() {
                            let public_key_der = target_user.public_key.as_ref().unwrap();
                            public_key.public_key_pem =
                                rust_lambda::rsa::der_to_pem(public_key_der);
                            expected_object.published = Some(target_user.get_published_time());
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

fn insert_digest(all_headers: &mut HeaderMap, request_body: &str) {
    let digest = rust_lambda::activitypub::digest::Digest::from_body(request_body);
    event!(Level::DEBUG, digest = digest);
    let digest = HeaderValue::from_str(&digest).unwrap();
    all_headers.insert("digest", digest);
}

fn insert_date(all_headers: &mut HeaderMap) {
    let date: OffsetDateTime = OffsetDateTime::now_utc();
    let date = date.format(&Rfc2822).unwrap();
    let date = HeaderValue::from_str(&date).unwrap();
    all_headers.insert("date", date);
}

fn insert_signature(
    signer: &User,
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
    event!(
        Level::DEBUG,
        select_headers = select_headers,
        public_key = hex::encode(&signer.public_key.as_ref().unwrap()),
    );
    let signature = signer.sign(&select_headers).unwrap();
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
    event!(
        Level::DEBUG,
        query = query,
        method = http_description.method.as_ref(),
        path = http_description.path.as_ref().unwrap(),
        all_headers = format!("{all_headers:?}"),
    );
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
