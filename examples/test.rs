use aws_lambda_events::apigw::ApiGatewayV2httpRequest;
use aws_lambda_events::apigw::ApiGatewayV2httpResponse;
use aws_lambda_events::encodings::Body;
use aws_sdk_dynamodb::model::{
    AttributeDefinition, KeySchemaElement, KeyType, ProvisionedThroughput, ScalarAttributeType,
};
use regex::Regex;
use serde::Deserialize;
use serde_json::{json, Value};
use std::env;
use std::fs::File;

type TestCases = Vec<TestCase>;

const DB_URL: &str = "http://localhost:8000";
const TABLE_NAME: &str = "table_name";
const UUID_PLACEHOLDER: &str = "abcdef1234567890abcdef1234567890";

#[derive(Deserialize)]
struct TestCase {
    request: ApiGatewayV2httpRequest,
    request_body_json: Option<Value>,
    expected_response: ApiGatewayV2httpResponse,
    expected_body_json: Option<Value>,
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let test_target_url = &args[1];
    if test_target_url.contains("localhost") {
        let db_client = rust_lambda::dynamodb::get_local_client(DB_URL.to_owned()).await;
        create_table_if_not_exists(&db_client).await;
    }

    let paths = std::fs::read_dir("./test-cases").unwrap();

    let http_client = reqwest::Client::new();

    for path in paths {
        let path_value = path.unwrap().path();
        let file = File::open(path_value).unwrap();
        let mut file_deserializer = serde_json::Deserializer::from_reader(file);
        let test_cases = TestCases::deserialize(&mut file_deserializer).unwrap();
        let mut last_uuid = String::new();
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
                let request_body = json!(&test.request_body_json);
                let body = request_body
                    .to_string()
                    .replace(UUID_PLACEHOLDER, last_uuid.as_str());
                actual_response = http_client
                    .post(url)
                    .body(body)
                    .headers(request.headers.to_owned())
                    .send()
                    .await
                    .unwrap();
            } else {
                url = url.replace(UUID_PLACEHOLDER, last_uuid.as_str());
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
            last_uuid = assert_body_matches_with_replacement(&test, &actual_body_text);
        }
    }
}

fn assert_body_matches_with_replacement(test: &TestCase, actual_body_text: &String) -> String {
    let uuid_re = Regex::new(r"[a-f0-9]{32}").unwrap();
    for capture in uuid_re.captures_iter(actual_body_text.as_str()) {
        let replaced_text = uuid_re
            .replace_all(&actual_body_text, UUID_PLACEHOLDER)
            .to_string();
        assert_body_matches(&test, &replaced_text);
        return capture[0].to_owned();
    }

    assert_body_matches(&test, actual_body_text);
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

async fn table_exists(client: &aws_sdk_dynamodb::Client, table: &str) -> bool {
    let table_list = client.list_tables().send().await.unwrap();
    println!("tables {:?}", table_list);
    table_list
        .table_names()
        .as_ref()
        .unwrap()
        .contains(&table.into())
}

async fn create_table_if_not_exists(client: &aws_sdk_dynamodb::Client) {
    if table_exists(client, TABLE_NAME).await {
        return;
    }

    let a_name: String = rust_lambda::dynamodb::PARTITION_KEY_NAME.to_owned();

    let ad = AttributeDefinition::builder()
        .attribute_name(&a_name)
        .attribute_type(ScalarAttributeType::S)
        .build();

    let ks = KeySchemaElement::builder()
        .attribute_name(&a_name)
        .key_type(KeyType::Hash)
        .build();

    let pt = ProvisionedThroughput::builder()
        .read_capacity_units(10)
        .write_capacity_units(5)
        .build();

    client
        .create_table()
        .table_name(TABLE_NAME)
        .key_schema(ks)
        .attribute_definitions(ad)
        .provisioned_throughput(pt)
        .send()
        .await
        .unwrap();
}
