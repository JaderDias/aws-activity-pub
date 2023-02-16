use crate::activitypub::object::Object;
use crate::settings::Settings;
use aws_sdk_dynamodb::model::AttributeValue;
use rocket::serde::json::Json;

#[rocket::get("/users/<username>/outbox")]
pub async fn handler(username: &str, settings: &rocket::State<Settings>) -> Json<Vec<Object>> {
    let partition = format!("users/{username}/statuses");
    let response = settings
        .db_client
        .query()
        .table_name(&settings.table_name)
        .key_condition_expression("#partition_key = :valueToMatch")
        .expression_attribute_names(
            "#partition_key",
            crate::dynamodb::PARTITION_KEY_NAME.to_owned(),
        )
        .expression_attribute_values(":valueToMatch", AttributeValue::S(partition))
        .limit(20)
        .scan_index_forward(false)
        .send()
        .await
        .unwrap();
    let items = response.items().unwrap();
    Json(serde_dynamo::from_items(items.to_vec()).unwrap())
}
