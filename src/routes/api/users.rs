use crate::activitypub::object::Object;
use crate::dynamodb::DbSettings;
use aws_sdk_dynamodb::model::AttributeValue;

#[rocket::post("/api/v1/users", data = "<user>")]
pub async fn handler(
    user: rocket::serde::json::Json<Object>,
    db_settings: &rocket::State<DbSettings>,
) -> Option<String> {
    let object = user.into_inner();
    let object_type = object.r#type.as_str();
    if object_type != "Person" {
        return None;
    }

    let partition = crate::dynamodb::get_uuid();
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
