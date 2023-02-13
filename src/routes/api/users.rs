use crate::activitypub::object::Object;

#[rocket::post("/api/v1/users", data = "<user>")]
pub async fn handler(
    user: rocket::serde::json::Json<Object>,
    settings: &rocket::State<crate::Settings>,
) -> Option<String> {
    let object = user.into_inner();
    let object_type = object.r#type.as_str();
    if object_type != "Person" {
        return None;
    }

    let partition = format!("users/{}", object.preferred_username.as_ref().unwrap());
    let values = serde_dynamo::to_item(object).unwrap();
    crate::dynamodb::put_item(
        &settings.db_client,
        &settings.table_name,
        partition.as_str(),
        "user",
        values,
    )
    .await
    .unwrap();
    Some(partition.to_string())
}
