use aws_sdk_dynamodb::model::AttributeValue;
use library::activitypub::object::Object;
use rocket::serde::json::Json;
use tracing::{event, Level};

#[rocket::get("/users/<username>/statuses/<status_id>")]
pub async fn handler(
    username: &str,
    status_id: &str,
    settings: &rocket::State<library::settings::Settings>,
) -> Option<Json<Object>> {
    event!(Level::DEBUG, username = username, status_id = status_id);
    let partition = format!("users/{username}/statuses");
    let get_item_output = settings.db_client
        .get_item()
        .table_name(&settings.table_name)
        .key(library::dynamodb::PARTITION_KEY_NAME, AttributeValue::S(partition))
        .key(library::dynamodb::SORT_KEY_NAME, AttributeValue::S(status_id.to_owned()))
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
