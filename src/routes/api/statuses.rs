use crate::activitypub::object::Object;

#[rocket::post("/api/v1/statuses", data = "<status>")]
pub async fn handler(
    status: rocket::serde::json::Json<Object>,
    settings: &rocket::State<crate::settings::Settings>,
) -> Option<String> {
    let mut object = status.into_inner();
    let object_type = object.r#type.as_str();
    if object_type != "Note" {
        return None;
    }

    let status_id = crate::faas_snowflake_id::get_id(settings.node_id).to_string();
    let username = "target_username"; // TODO: replace with authenticated username
    let partition = format!("users/{username}/statuses");
    object.id = Some(format!(
        "{}/{}/{}",
        settings.base_url,
        partition.as_str(),
        status_id.as_str(),
    ));

    let values = serde_dynamo::to_item(object).unwrap();
    crate::dynamodb::put_item(
        &settings.db_client,
        &settings.table_name,
        partition.as_str(),
        status_id.as_str(),
        values,
    )
    .await
    .unwrap();
    Some(status_id.to_string())
}
