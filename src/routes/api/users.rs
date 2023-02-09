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
    let preferred_username = object.preferred_username.unwrap();
    let name = object.name.unwrap();
    let fields = std::collections::HashMap::from([
        ("type".to_owned(), AttributeValue::S(object_type.to_owned())),
        (
            "@context".to_owned(),
            AttributeValue::S(serde_json::json!(object.context).to_string()),
        ),
        (
            "preferred_username".to_owned(),
            AttributeValue::S(preferred_username),
        ),
        ("name".to_owned(), AttributeValue::S(name)),
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
