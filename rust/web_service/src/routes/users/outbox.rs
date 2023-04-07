use aws_sdk_dynamodb::types::AttributeValue;
use library::{activitypub::object::Object, settings::Settings};
use rocket::http::ContentType;
use rocket::serde::json::Json;

#[derive(rocket::Responder)]
pub struct Outbox(Json<serde_json::Value>, ContentType);

#[rocket::get("/users/<username>/outbox")]
pub async fn handler(username: &str, settings: &rocket::State<Settings>) -> Outbox {
    let partition = format!("users/{username}/statuses");
    let response = settings
        .db_client
        .query()
        .table_name(&settings.table_name)
        .key_condition_expression("#partition_key = :valueToMatch")
        .expression_attribute_names(
            "#partition_key",
            library::dynamodb::PARTITION_KEY_NAME.to_owned(),
        )
        .expression_attribute_values(":valueToMatch", AttributeValue::S(partition))
        .select(aws_sdk_dynamodb::types::Select::Count)
        .send()
        .await
        .unwrap();
    let id = format!("{}/users/{username}/outbox", settings.base_url);
    let body = Json(serde_json::json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": id,
        "type": "OrderedCollection",
        "totalItems": response.count(),
        "first": format!("{id}?page=true"),
        "last": format!("{id}?min_id=0&page=true"),
    }));
    let content_type =
        ContentType::new("application", "activity+json").with_params([("charset", "utf-8")]);
    return Outbox(body, content_type);
}

#[rocket::get("/users/<username>/outbox?page=true")]
pub async fn page(username: &str, settings: &rocket::State<Settings>) -> Outbox {
    let partition = format!("users/{username}/statuses");
    let response = settings
        .db_client
        .query()
        .table_name(&settings.table_name)
        .key_condition_expression("#partition_key = :valueToMatch")
        .expression_attribute_names(
            "#partition_key",
            library::dynamodb::PARTITION_KEY_NAME.to_owned(),
        )
        .expression_attribute_values(":valueToMatch", AttributeValue::S(partition))
        .limit(20)
        .scan_index_forward(false)
        .send()
        .await
        .unwrap();
    let items = response.items().unwrap();
    let body: Vec<Object> = serde_dynamo::from_items(items.to_vec()).unwrap();
    let id = format!("{}/users/{username}/outbox", settings.base_url);
    let body = Json(serde_json::json!({
        "@context":[
            "https://www.w3.org/ns/activitystreams",
            {
              "ostatus": "http://ostatus.org#",
              "atomUri": "ostatus:atomUri",
              "inReplyToAtomUri": "ostatus:inReplyToAtomUri",
              "conversation": "ostatus:conversation",
              "sensitive": "as:sensitive",
              "toot": "http://joinmastodon.org/ns#",
              "votersCount": "toot:votersCount",
              "blurhash": "toot:blurhash",
              "focalPoint": {
                "@container": "@list",
                "@id": "toot:focalPoint"
              },
              "Hashtag": "as:Hashtag"
            }
          ],
        "id": format!("{id}?page=true"),
        "type": "OrderedCollectionPage",
        "next": format!("{id}?max_id=109635460763016637&page=true"),
        "prev": format!("{id}?min_id=109757651111534104&page=true"),
        "partOf": id,
        "orderedItems": body
    }));
    let content_type =
        ContentType::new("application", "activity+json").with_params([("charset", "utf-8")]);
    return Outbox(body, content_type);
}
