use library::{activitypub, faas_snowflake_id};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

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
    let published_time = OffsetDateTime::now_utc();
    let status = activitypub::object::Object {
        actor: Some(format!(
            "https://{domain}/users/{preferred_username}
        "
        )),
        atom_uri: None,
        attachment: Some(Vec::new()),
        attributed_to: None,
        cc: Some(vec![format!(
            "https://{domain}/users/{preferred_username}/followers"
        )]),
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
            "https://{domain}/users/{preferred_username}/statuses/{sort_value}"
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
        published: Some(published_time.format(&Rfc3339).unwrap()),
        r#type: Some("Note".to_owned()),
        sensitive: Some(false),
        sort_key: None,
        summary: None,
        tag: Some(Vec::new()),
        to: Some(vec![
            "https://www.w3.org/ns/activitystreams#Public".to_owned()
        ]),
        url: Some(format!("https://{domain}/@{preferred_username}")),
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
