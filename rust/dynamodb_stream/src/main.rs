use aws_lambda_events_extended::dynamodb::DynamoDBEvent;
use http::header::{HeaderMap, HeaderValue};
use lambda_runtime::{service_fn, Error, LambdaEvent};
use library::{
    activitypub::object::Object,
    dynamodb,
    model::{self, user::User},
};
use serde_json::{json, Value};
use std::collections::HashMap;
use time::OffsetDateTime;
use tracing::{event, Level};

const METHOD: &str = "POST";

#[tokio::main]
async fn main() -> Result<(), Error> {
    library::trace::init();
    let func = service_fn(func);
    lambda_runtime::run(func).await?;
    Ok(())
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

            event!(Level::DEBUG, "dynamodb_event_to_map: {new_image:?}");
            let status = dynamodb_event_to_map(new_image);
            let status: Object = serde_dynamo::from_item(status).unwrap();
            let username = split_partition[1];
            let signature_key_id = format!("https://{domain_name}/users/{username}#main-key");
            let get_item_output = model::user::get_item(username, &db_client, &table_name).await;
            let item = get_item_output.item.unwrap();
            let user: User = serde_dynamo::from_item(item).unwrap();

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
                let (url, request_body, headers) = get_notification(
                    &status,
                    &user,
                    signature_key_id.as_str(),
                    &follower,
                    &OffsetDateTime::UNIX_EPOCH,
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

fn dynamodb_event_to_map(
    stream: HashMap<String, aws_lambda_events_extended::dynamodb::AttributeValue>,
) -> HashMap<String, aws_sdk_dynamodb::model::AttributeValue> {
    let mut items = HashMap::new();
    for (key, value) in stream {
        items.insert(key, convert(&value).clone());
    }
    items
}

fn convert(
    value: &aws_lambda_events_extended::dynamodb::AttributeValue,
) -> aws_sdk_dynamodb::model::AttributeValue {
    if let Some(bool) = &value.bool {
        return aws_sdk_dynamodb::model::AttributeValue::Bool(*bool);
    }
    if let Some(l) = &value.l {
        return aws_sdk_dynamodb::model::AttributeValue::L(
            l.iter().map(convert).collect::<Vec<_>>(),
        );
    }
    if let Some(m) = &value.m {
        return aws_sdk_dynamodb::model::AttributeValue::M(dynamodb_event_to_map(m.clone()));
    }
    if let Some(ns) = &value.ns {
        return aws_sdk_dynamodb::model::AttributeValue::Ns(ns.clone());
    }
    if let Some(s) = &value.s {
        return aws_sdk_dynamodb::model::AttributeValue::S(s.clone());
    }
    if let Some(ss) = &value.ss {
        return aws_sdk_dynamodb::model::AttributeValue::Ss(ss.clone());
    }

    panic!("empty attribute value");
}

fn get_notification(
    status: &Object,
    user: &User,
    signature_key_id: &str,
    follower: &Object,
    time_provider: &dyn library::time_provider::TimeProvider,
) -> (String, String, HeaderMap) {
    let url = follower.actor.as_ref().unwrap();
    let url = format!("{url}/inbox");
    let split_url = url.splitn(4, '/').collect::<Vec<&str>>();
    let path = format!("/{}", split_url[3]);
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
        path.as_str(),
        &mut headers,
        &request_body,
        user.private_key.as_ref().unwrap(),
        signature_key_id,
        time_provider,
    );
    (url, request_body, headers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose, Engine as _};
    use library::activitypub;
    use rsa::pkcs1::DecodeRsaPrivateKey;
    use rsa::pkcs1::EncodeRsaPublicKey;
    use rsa::RsaPrivateKey;

    struct MockTimeProvider {}

    impl library::time_provider::TimeProvider for MockTimeProvider {
        fn now_utc(&self) -> OffsetDateTime {
            OffsetDateTime::UNIX_EPOCH
        }
    }

    #[test]
    fn test_dynamodb_event_to_map() {
        // Arrange
        let mut context =
            HashMap::<String, aws_lambda_events_extended::dynamodb::AttributeValue>::new();
        context.insert(
            "PropertyValue".to_owned(),
            aws_lambda_events_extended::dynamodb::AttributeValue {
                b: None,
                bool: None,
                bs: None,
                l: None,
                m: None,
                n: None,
                ns: None,
                null: None,
                s: Some("schema:PropertyValue".to_owned()),
                ss: None,
            },
        );
        let mut stream =
            HashMap::<String, aws_lambda_events_extended::dynamodb::AttributeValue>::new();
        stream.insert(
            "partition_key".to_owned(),
            aws_lambda_events_extended::dynamodb::AttributeValue {
                b: None,
                bool: None,
                bs: None,
                l: None,
                m: None,
                n: None,
                ns: None,
                null: None,
                s: Some("users/sample_user/statuses".to_owned()),
                ss: None,
            },
        );
        stream.insert(
            "@context".to_owned(),
            aws_lambda_events_extended::dynamodb::AttributeValue {
                b: None,
                bool: None,
                bs: None,
                l: Some(vec![
                    aws_lambda_events_extended::dynamodb::AttributeValue {
                        b: None,
                        bool: None,
                        bs: None,
                        l: None,
                        m: None,
                        n: None,
                        ns: None,
                        null: None,
                        s: Some("https://www.w3.org/ns/activitystreams".to_owned()),
                        ss: None,
                    },
                    aws_lambda_events_extended::dynamodb::AttributeValue {
                        b: None,
                        bool: None,
                        bs: None,
                        l: None,
                        m: Some(context),
                        n: None,
                        ns: None,
                        null: None,
                        s: None,
                        ss: None,
                    },
                ]),
                m: None,
                n: None,
                ns: None,
                null: None,
                s: None,
                ss: None,
            },
        );
        let mut expected_context =
            HashMap::<String, aws_sdk_dynamodb::model::AttributeValue>::new();
        expected_context.insert(
            "PropertyValue".to_string(),
            aws_sdk_dynamodb::model::AttributeValue::S("schema:PropertyValue".to_owned()),
        );
        let mut expected = HashMap::<String, aws_sdk_dynamodb::model::AttributeValue>::new();
        expected.insert(
            "partition_key".to_owned(),
            aws_sdk_dynamodb::model::AttributeValue::S("users/sample_user/statuses".to_owned()),
        );
        expected.insert(
            "@context".to_owned(),
            aws_sdk_dynamodb::model::AttributeValue::L(vec![
                aws_sdk_dynamodb::model::AttributeValue::S(
                    "https://www.w3.org/ns/activitystreams".to_owned(),
                ),
                aws_sdk_dynamodb::model::AttributeValue::M(expected_context),
            ]),
        );

        // Act
        let actual = dynamodb_event_to_map(stream);

        // Assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_notify_follower() {
        // Arrange
        let domain = "example.com";
        let username = "test_username";
        let sort_value = 1234567890;
        let status = Object {
            actor: None,
            atom_uri: None,
            attachment: Some(Vec::new()),
            attributed_to: None,
            cc: None,
            content: Some("test content".to_string()),
            context: activitypub::context::default(),
            conversation: Some(format!(
                "tag:{domain},2019-04-28:objectId=1754000:objectType=Conversation"
            )),
            devices: None,
            discoverable: Some(false),
            followers: None,
            following: None,
            id: Some(format!(
                "https://{domain}/users/{username}/statuses/{sort_value}"
            )),
            in_reply_to: None,
            in_reply_to_atom_uri: None,
            inbox: None,
            manually_approves_followers: None,
            name: None,
            object: None,
            outbox: None,
            partition_key: None,
            preferred_username: None,
            public_key: None,
            published: Some("2023-01-19T00:00:00Z".to_owned()),
            r#type: Some("Note".to_owned()),
            sensitive: Some(false),
            sort_key: None,
            summary: None,
            tag: Some(Vec::new()),
            to: Some(vec![
                "https://www.w3.org/ns/activitystreams#Public".to_owned()
            ]),
            url: Some(format!("https://{domain}/@{username}")),
            extra: serde_json::Value::Null,
        };
        let private_key_der =  general_purpose::STANDARD
        .decode("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").unwrap();
        let private_key = RsaPrivateKey::from_pkcs1_der(&private_key_der).unwrap();
        let user = User {
            preferred_username: Some(username.to_owned()),
            private_key: Some(private_key_der),
            public_key: Some(
                private_key
                    .to_public_key()
                    .to_pkcs1_der()
                    .unwrap()
                    .as_bytes()
                    .to_vec(),
            ),
            published_unix_time_seconds: 0,
        };
        let signature_key_id = "https://example.com/users/test_username#main-key";
        let follower = Object {
            actor: Some(format!("https://{domain}/users/follower")),
            atom_uri: None,
            attachment: Some(Vec::new()),
            attributed_to: None,
            cc: None,
            content: Some("test content".to_string()),
            context: activitypub::context::default(),
            conversation: None,
            devices: None,
            discoverable: Some(false),
            followers: None,
            following: None,
            id: Some(format!("https://{domain}/users/follower")),
            in_reply_to: None,
            in_reply_to_atom_uri: None,
            inbox: None,
            manually_approves_followers: None,
            name: None,
            object: None,
            outbox: None,
            partition_key: None,
            preferred_username: Some("follower".to_owned()),
            public_key: None,
            published: Some("2023-01-19T00:00:00Z".to_owned()),
            r#type: Some("Person".to_owned()),
            sensitive: Some(false),
            sort_key: None,
            summary: None,
            tag: Some(Vec::new()),
            to: None,
            url: Some(format!("https://{domain}/@follower")),
            extra: serde_json::Value::Null,
        };
        let expected_url = "https://example.com/users/follower/inbox";
        let expected_request_body = "{\"attachment\":[],\"content\":\"test content\",\"@context\":[\"https://www.w3.org/ns/activitystreams\",\"https://w3id.org/security/v1\",{\"Curve25519Key\":\"toot:Curve25519Key\",\"Device\":\"toot:Device\",\"Ed25519Key\":\"toot:Ed25519Key\",\"Ed25519Signature\":\"toot:Ed25519Signature\",\"EncryptedMessage\":\"toot:EncryptedMessage\",\"PropertyValue\":\"schema:PropertyValue\",\"alsoKnownAs\":{\"@id\":\"as:alsoKnownAs\",\"@type\":\"@id\"},\"cipherText\":\"toot:cipherText\",\"claim\":{\"@id\":\"toot:claim\",\"@type\":\"@id\"},\"deviceId\":\"toot:deviceId\",\"devices\":{\"@id\":\"toot:devices\",\"@type\":\"@id\"},\"discoverable\":\"toot:discoverable\",\"featured\":{\"@id\":\"toot:featured\",\"@type\":\"@id\"},\"featuredTags\":{\"@id\":\"toot:featuredTags\",\"@type\":\"@id\"},\"fingerprintKey\":{\"@id\":\"toot:fingerprintKey\",\"@type\":\"@id\"},\"identityKey\":{\"@id\":\"toot:identityKey\",\"@type\":\"@id\"},\"manuallyApprovesFollowers\":\"as:manuallyApprovesFollowers\",\"messageFranking\":\"toot:messageFranking\",\"messageType\":\"toot:messageType\",\"movedTo\":{\"@id\":\"as:movedTo\",\"@type\":\"@id\"},\"publicKeyBase64\":\"toot:publicKeyBase64\",\"schema\":\"http://schema.org#\",\"suspended\":\"toot:suspended\",\"toot\":\"http://joinmastodon.org/ns#\",\"value\":\"schema:value\"}],\"conversation\":\"tag:example.com,2019-04-28:objectId=1754000:objectType=Conversation\",\"discoverable\":false,\"id\":\"https://example.com/users/test_username/statuses/1234567890\",\"published\":\"2023-01-19T00:00:00Z\",\"type\":\"Note\",\"sensitive\":false,\"tag\":[],\"to\":[\"https://www.w3.org/ns/activitystreams#Public\"],\"url\":\"https://example.com/@test_username\"}";
        let mut expected_headers = HeaderMap::new();
        expected_headers.append(
            "content-type",
            HeaderValue::from_str("application/activity+json").unwrap(),
        );
        expected_headers.append("host", HeaderValue::from_str("example.com").unwrap());
        expected_headers.append(
            "digest",
            HeaderValue::from_str("SHA-256=cDjf+fYECF0XKY1THmYVgFNRzP5s2M1lx7TGQkWvbxI=").unwrap(),
        );
        expected_headers.append(
            "date",
            HeaderValue::from_str("Thu, 01 Jan 1970 00:00:00 GMT").unwrap(),
        );
        expected_headers.append(
            "signature",
            HeaderValue::from_str("keyId=\"https://example.com/users/test_username#main-key\",algorithm=\"rsa-sha256\",headers=\"(request-target) host date digest content-type\",signature=\"QPNXXvmfOx0jHzRoUCZywql2KdoXgHrdJvkS/tzwD8CIy161GPTCXTnk9daKy3Q1Us6GtbGEJ/ho1rAI0LwDzp2iOLeozvw0mrMYlF4hP/xHLLzQtq2mbYpM9ZvtWRFUOCf0YTcBpBnW9pS3Rqqw+vTKCrMpDggmryvn8Dnn5OQ6j9Q+AiuOABXpGZbmws2jdPxRkWde6j+Otia7fACQyjsetrvFisdoFWBWHZdl5bsSOTZIm9wm3zQOpMHO4xMQ4q6vgVHNStESBDism7Yba/ikqs7hVWmf1Gg4vs+++3BKkQvNVHIrRBJsXmXQ+YsYV+Cxwn+BRTWnoxGQR5P/G6ER4oSeiNHIexchpelrfbi6TL62uQsFL6L5QYALJcUE+aFHgXaYzCTsVteCCrG58+wHAPd5arOZRaAF0GiZQKvZJY/r4eYvkk7hew1ZOpydFysNPIdcbMz18qBtEUMlbDIiez4dHp5VqyJRktKAnUjpWFh7yXBHEC8062JdICWuV7wzpJeEvHRRE718bkleyUcJ96RuGGvfmBj0RsbyNOtQpZZu1WmPvBRZqiQQ40lBI5So+2ao6cUZNMxLch9Saul9YYQkpSXbHiCxmQZrAjWCtN5uWakFF4o+KZL6cZ6OBxkTZhxuFtyiO9ayHlfIyTRZ+pjaefVnSTtooJujQhU=\"").unwrap());
        let time_provider = MockTimeProvider {};

        // Act
        let (actual_url, actual_request_body, actual_headers) =
            get_notification(&status, &user, signature_key_id, &follower, &time_provider);

        // Assert
        assert_eq!(expected_url, actual_url);
        assert_eq!(expected_request_body, actual_request_body);
        assert_eq!(expected_headers, actual_headers);
    }
}
