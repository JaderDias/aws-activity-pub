use crate::activitypub::object::Object;
use crate::dynamodb::DbSettings;
use aws_sdk_dynamodb::model::AttributeValue;

#[rocket::post("/api/v1/statuses", data = "<status>")]
pub async fn handler(
    status: rocket::serde::json::Json<Object>,
    db_settings: &rocket::State<DbSettings>,
) -> Option<String> {
    let object = status.into_inner();
    let object_type = object.r#type.as_str();
    if object_type != "Note" {
        return None;
    }

    let status_id = crate::dynamodb::get_uuid();
    let username = "test_username"; // TODO: replace with authenticated username
    let partition = format!("users/{username}/statuses/{status_id}");
    let id = format!("https://example.com/{}", partition.as_str()); // TODO: replace with domain name
    let context = serde_json::json!(object.context);
    let fields = std::collections::HashMap::from([
        ("type".to_owned(), AttributeValue::S(object_type.to_owned())),
        (
            "@context".to_owned(),
            AttributeValue::S(context.to_string()),
        ),
        ("id".to_owned(), AttributeValue::S(id)),
        (
            "content".to_owned(),
            AttributeValue::S(object.content.unwrap()),
        ),
        (
            "conversation".to_owned(),
            AttributeValue::S(object.conversation.unwrap()),
        ),
        (
            "published".to_owned(),
            AttributeValue::S(object.published.unwrap()),
        ),
        ("to".to_owned(), AttributeValue::Ss(object.to.unwrap())),
        (
            "sensitive".to_owned(),
            AttributeValue::Bool(object.sensitive.unwrap()),
        ),
    ]);
    crate::dynamodb::put_item(
        &db_settings.client,
        &db_settings.table_name,
        partition.as_str(),
        fields,
    )
    .await
    .unwrap();
    Some(partition.to_string())
}
