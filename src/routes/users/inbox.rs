use crate::activitypub::object::Object;
use crate::dynamodb::DbSettings;
use aws_sdk_dynamodb::model::AttributeValue;

#[rocket::post("/users/<_username>/inbox", data = "<wrapper>")]
pub async fn handler(
    _username: &str,
    wrapper: rocket::serde::json::Json<Object>,
    db_settings: &rocket::State<DbSettings>,
) -> Option<String> {
    let object = wrapper.into_inner();
    let object_type = object.r#type.as_str();
    if object_type != "Follow" {
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
