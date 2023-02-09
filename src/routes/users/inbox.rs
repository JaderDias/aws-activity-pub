use crate::activitypub::object::Object;
use crate::Settings;
use aws_sdk_dynamodb::model::AttributeValue;

#[rocket::post("/users/<_username>/inbox", data = "<wrapper>")]
pub async fn handler(
    _username: &str,
    wrapper: rocket::serde::json::Json<Object>,
    settings: &rocket::State<Settings>,
) -> Option<String> {
    let object = wrapper.into_inner();
    let object_type = object.r#type.as_str();
    if object_type != "Follow" {
        return None;
    }

    let partition = crate::dynamodb::get_uuid();
    let values = serde_dynamo::to_item(object).unwrap();
    crate::dynamodb::put_item(
        &settings.db_client,
        &settings.table_name,
        partition.as_str(),
        values,
    )
    .await
    .unwrap();
    Some(partition.to_string())
}
