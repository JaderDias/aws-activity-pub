use crate::activitypub::object::Object;
use crate::dynamodb::DbSettings;

#[rocket::post("/api/v1/statuses", data = "<status>")]
pub async fn handler(
    status: rocket::serde::json::Json<Object>,
    db_settings: &rocket::State<DbSettings>,
) -> Option<String> {
    let mut object = status.into_inner();
    let object_type = object.r#type.as_str();
    if object_type != "Note" {
        return None;
    }

    let status_id = crate::dynamodb::get_uuid();
    let username = "test_username"; // TODO: replace with authenticated username
    let partition = format!("users/{username}/statuses/{status_id}");
    object.id = Some(format!("https://example.com/{}", partition.as_str())); // TODO: replace with domain name

    let values = serde_dynamo::to_item(object).unwrap();
    crate::dynamodb::put_item(
        &db_settings.client,
        &db_settings.table_name,
        partition.as_str(),
        values,
    )
    .await
    .unwrap();
    Some(partition.to_string())
}
