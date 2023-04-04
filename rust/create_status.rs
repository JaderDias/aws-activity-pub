use library::{activitypub, faas_snowflake_id};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        panic!("Usage: create_status username domain dynamodb_table_name")
    }
    let preferred_username = args[1].clone();
    let domain = args[2].clone();
    let table_name = args[3].clone();
    let table_name = table_name.as_str();
    let db_client = library::dynamodb::get_client().await;
    let partition = format!("users/{preferred_username}/statuses");
    let node_id = faas_snowflake_id::get_node_id();
    let sort_value = faas_snowflake_id::get_id(node_id).to_string();
    let status = activitypub::object::Object {
        actor: None,
        atom_uri: None,
        attachment: Some(Vec::new()),
        attributed_to: None,
        cc: None,
        content: Some("test content".to_string()),
        context: activitypub::context::default(),
        conversation: Some(format!(
            "tag:{domain},2019-04-28:objectId=1754000:objectType=Conversation"
        )),
        devices: None,
        discoverable: Some(false),
        followers: None,
        following: None,
        id: Some(format!(
            "https://{domain}/users/test_username/statuses/{sort_value}"
        )),
        in_reply_to: None,
        in_reply_to_atom_uri: None,
        inbox: None,
        manually_approves_followers: None,
        name: None,
        object: None,
        outbox: None,
        partition_key: None,
        preferred_username: None,
        public_key: None,
        published: Some("2023-01-19T00:00:00Z".to_owned()),
        r#type: Some("Note".to_owned()),
        sensitive: Some(false),
        sort_key: None,
        summary: None,
        tag: Some(Vec::new()),
        to: Some(vec![
            "https://www.w3.org/ns/activitystreams#Public".to_owned()
        ]),
        url: Some(format!("https://{domain}/@test_username")),
        extra: serde_json::Value::Null,
    };
    library::dynamodb::put_item(
        &db_client,
        table_name,
        partition.as_str(),
        &sort_value,
        &status,
    )
    .await
    .unwrap();
}
