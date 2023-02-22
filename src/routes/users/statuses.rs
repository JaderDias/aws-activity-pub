use crate::activitypub::object::Object;
use aws_sdk_dynamodb::model::AttributeValue;
use rocket::serde::json::Json;

#[rocket::get("/users/<username>/statuses/<status_id>")]
pub async fn handler(
    username: &str,
    status_id: &str,
    settings: &rocket::State<crate::settings::Settings>,
) -> Option<Json<Object>> {
    let partition = format!("users/{username}/statuses");
    let get_item_output = settings.db_client
        .get_item()
        .table_name(&settings.table_name)
        .key(crate::dynamodb::PARTITION_KEY_NAME, AttributeValue::S(partition))
        .key(crate::dynamodb::SORT_KEY_NAME, AttributeValue::S(status_id.to_owned()))
        .projection_expression("#context, attachment, id, #type, inReplyToAtomUri, published, #to, #sensitive, conversation, content, tag")
        .expression_attribute_names("#context", "@context")
        .expression_attribute_names("#sensitive", "sensitive")
        .expression_attribute_names("#to", "to")
        .expression_attribute_names("#type", "type")
        .send()
        .await
        .unwrap();
    let item = get_item_output.item.unwrap();
    let object: Object = serde_dynamo::from_item(item).unwrap();
    Some(Json(object))
}
