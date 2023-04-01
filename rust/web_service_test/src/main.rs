use test::TestCases;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use tracing::{event, Level};

use library::activitypub::object::Object;
use library::dynamodb;
use library::model::user::User;
use std::env;
use std::fs;

mod assert;
mod test;

fn get_target(args: &[String]) -> Option<(String, String, String, String, String)> {
    if args.len() != 6 {
        return None;
    }

    Some((
        args[1].clone(),
        args[2].clone(),
        args[3].clone(),
        args[4].clone(),
        args[5].clone(),
    ))
}

fn add_protocol(domain: &str) -> String {
    if domain.starts_with("localhost") {
        format!("http://{domain}")
    } else {
        format!("https://{domain}")
    }
}

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() {
    library::trace::init();
    let args: Vec<String> = env::args().collect();
    let (target_domain, target_username, signer_domain, signer_username, table_name)
     = get_target(&args).expect(
        "Usage: LOCAL_DYNAMODB_URL=http://localhost:8000 test localhost:8080 target_username localhost:8080 signer_username dynamodb_table_name",
    );

    let table_name = table_name.as_str();
    let target_url = add_protocol(target_domain.as_str());
    let signer_url = add_protocol(signer_domain.as_str());
    let signature_key_id = format!("{signer_url}/users/{signer_username}#main-key");

    let db_client = dynamodb::get_client().await;
    let target_user: Option<User> = if target_domain.starts_with("localhost") {
        dynamodb::create_table_if_not_exists(&db_client, table_name).await;
        Some(library::model::user::create(&db_client, table_name, target_username.as_str()).await)
    } else {
        None
    };
    let signer: User = if signer_domain.starts_with("localhost") {
        dynamodb::create_table_if_not_exists(&db_client, table_name).await;
        library::model::user::create(&db_client, table_name, signer_username.as_str()).await
    } else {
        let get_item_output =
            library::model::user::get_item(signer_username.as_str(), &db_client, table_name).await;
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
            .replace("TARGET_URN_PLACEHOLDER", target_domain.as_str())
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
                let mut request_body = serde_json::json!(&test.request_body_json).to_string();
                if let Some(placeholder) = &test.placeholder {
                    request_body = request_body
                        .to_string()
                        .replace(placeholder, last_regex_capture.as_str());
                }

                let headers = &mut request.headers;
                if let Some(_host) = headers.get("host") {
                    library::activitypub::request::sign(
                        request.request_context.http.method.as_ref(),
                        request.request_context.http.path.as_ref().unwrap(),
                        headers,
                        &request_body,
                        signer.private_key.as_ref().unwrap(),
                        signature_key_id.as_str(),
                    );
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
                    .headers(headers.clone())
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
                i64::from(actual_response.status().as_u16()),
                test.expected_response.status_code
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
                            public_key.public_key_pem = library::rsa::der_to_pem(public_key_der);
                            if let Some(published) = expected_object.published.as_mut() {
                                OffsetDateTime::parse(published, &Rfc3339).unwrap();
                                expected_object.published = Some(target_user.get_published_time());
                            }
                            test.expected_body_json =
                                Some(serde_json::to_value(expected_object).unwrap());
                        }
                    }
                }
            }
            last_regex_capture = assert::body_matches_with_replacement(&test, &actual_body_text);
        }
    }
}
