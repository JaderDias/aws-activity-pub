use crate::activitypub::object::Object;
use crate::dynamodb::DbSettings;
use aws_sdk_dynamodb::model::AttributeValue;

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![status]
}

#[rocket::post("/api/v1/statuses", data = "<status>")]
pub async fn status(
    status: rocket::serde::json::Json<Object>,
    db_settings: &rocket::State<DbSettings>,
) -> Option<String> {
    let object = status.into_inner();
    let object_type = object.r#type.as_str();
    if object_type != "Note" {
        return None;
    }

    let partition = crate::dynamodb::get_uuid();
    let content = object.content.unwrap();
    let published = object.published.unwrap();
    let sensitive = object.sensitive.unwrap();
    let fields = std::collections::HashMap::from([
        ("type".to_owned(), AttributeValue::S(object_type.to_owned())),
        ("content".to_owned(), AttributeValue::S(content)),
        ("published".to_owned(), AttributeValue::S(published)),
        ("sensitive".to_owned(), AttributeValue::Bool(sensitive)),
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
