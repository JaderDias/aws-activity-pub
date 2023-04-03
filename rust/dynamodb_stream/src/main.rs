use aws_lambda_events_extended::dynamodb::DynamoDBEvent;
use http::header::{HeaderMap, HeaderValue};
use lambda_runtime::{service_fn, Error, LambdaEvent};
use library::{activitypub::object::Object, dynamodb, model};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{event, Level};

const METHOD: &str = "POST";

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(func);
    lambda_runtime::run(func).await?;
    Ok(())
}

fn dynamodb_event_to_map(
    stream: HashMap<String, aws_lambda_events_extended::dynamodb::AttributeValue>,
) -> HashMap<String, aws_sdk_dynamodb::model::AttributeValue> {
    let mut items = HashMap::new();
    for (key, value) in stream {
        if let Some(s) = value.s {
            items.insert(key, aws_sdk_dynamodb::model::AttributeValue::S(s.clone()));
        } else if let Some(n) = value.n {
            items.insert(key, aws_sdk_dynamodb::model::AttributeValue::N(n.clone()));
        } else if let Some(bool) = value.bool {
            items.insert(
                key,
                aws_sdk_dynamodb::model::AttributeValue::Bool(bool.clone()),
            );
        } else if let Some(ss) = value.ss {
            items.insert(key, aws_sdk_dynamodb::model::AttributeValue::Ss(ss.clone()));
        } else if let Some(ns) = value.ns {
            items.insert(key, aws_sdk_dynamodb::model::AttributeValue::Ns(ns.clone()));
        }
    }
    items
}

async fn func(event: LambdaEvent<Value>) -> Result<Value, Error> {
    let (event, _context) = event.into_parts();
    let json_value = serde_json::to_value(event)?;

    let dynamodb_event: DynamoDBEvent = serde_json::from_value(json_value)?;

    let db_client = dynamodb::get_client().await;
    let http_client = reqwest::Client::new();
    let domain_name = std::env::var("CUSTOM_DOMAIN").unwrap();
    let table_name = std::env::var("DYNAMODB_TABLE").unwrap();
    for record in dynamodb_event.records {
        if let Some(new_image) = record.dynamodb.new_image {
            let partition = new_image
                .get(dynamodb::PARTITION_KEY_NAME)
                .and_then(|v| v.s.clone())
                .unwrap();
            event!(Level::DEBUG, "New item with ID: {partition}");
            let split_partition = partition.split('/').collect::<Vec<&str>>();
            if split_partition.len() != 3 {
                continue;
            }

            if split_partition[2] != "statuses" {
                continue;
            }

            let status = dynamodb_event_to_map(new_image);
            let status: Object = serde_dynamo::from_item(status).unwrap();
            let username = split_partition[1];
            let signature_key_id = format!("https://{domain_name}/users/{username}#main-key");
            let user_partition = format!("users/{username}");
            let get_item_output =
                model::user::get_item(user_partition.as_str(), &db_client, &table_name).await;
            let item = get_item_output.item.unwrap();
            let user: model::user::User = serde_dynamo::from_item(item).unwrap();

            let followers_partition = format!("users/{username}/followers");
            let response = db_client
                .query()
                .table_name(&table_name)
                .key_condition_expression("#partition_key = :valueToMatch")
                .expression_attribute_names(
                    "#partition_key",
                    dynamodb::PARTITION_KEY_NAME.to_owned(),
                )
                .expression_attribute_values(
                    ":valueToMatch",
                    aws_sdk_dynamodb::model::AttributeValue::S(followers_partition),
                )
                .limit(20)
                .scan_index_forward(false)
                .send()
                .await
                .unwrap();
            let items = response.items().unwrap();
            let followers: Vec<Object> = serde_dynamo::from_items(items.to_vec()).unwrap();
            for follower in followers {
                let url = follower.actor.unwrap();
                let url = format!("{url}/inbox");
                let split_url = url.splitn(4, '/').collect::<Vec<&str>>();
                let path = split_url[3];
                let status = status.clone();
                let request_body = serde_json::to_string(&status).unwrap();
                let mut headers = HeaderMap::new();
                headers.insert(
                    "Content-Type",
                    HeaderValue::from_str("application/activity+json").unwrap(),
                );
                headers.insert("Host", HeaderValue::from_str(split_url[2]).unwrap());
                library::activitypub::request::sign(
                    METHOD,
                    path,
                    &mut headers,
                    &request_body,
                    user.private_key.as_ref().unwrap(),
                    signature_key_id.as_str(),
                );
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
                http_client
                    .post(url)
                    .body(request_body)
                    .headers(headers.clone())
                    .send()
                    .await
                    .unwrap();
            }
        }
    }

    Ok(json!({ "message": "Success" }))
}
