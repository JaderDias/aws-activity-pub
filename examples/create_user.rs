use openssl::rsa::Rsa;

const KEYSIZE: u32 = 4096;
const DB_URL: &str = "http://localhost:8000";

#[tokio::main]
async fn main() {
    let keypair = Rsa::generate(KEYSIZE).unwrap();
    let preferred_username = "test_username";
    let partition = format!("users/{}", preferred_username);
    let values = serde_dynamo::to_item(rust_lambda::activitypub::object::Object {
        actor: None,
        atom_uri: None,
        attachment: None,
        attributed_to: None,
        cc: None,
        content: None,
        context: serde_json::value::Value::Null,
        conversation: None,
        discoverable: None,
        followers: None,
        following: None,
        id: None,
        in_reply_to: None,
        in_reply_to_atom_uri: None,
        inbox: None,
        manually_approves_followers: None,
        name: None,
        object: None,
        preferred_username: Some(preferred_username.to_owned()),
        private_key: Some(keypair.private_key_to_der().unwrap()),
        public_key: Some(keypair.public_key_to_der().unwrap()),
        published: None,
        r#type: "Person".to_owned(),
        sensitive: None,
        summary: None,
        tag: None,
        to: None,
        url: None,
    })
    .unwrap();

    let db_client = rust_lambda::dynamodb::get_local_client(DB_URL.to_owned()).await;
    rust_lambda::dynamodb::create_table_if_not_exists(&db_client).await;
    rust_lambda::dynamodb::put_item(
        &db_client,
        rust_lambda::dynamodb::DEFAULT_TABLE_NAME,
        partition.as_str(),
        "user",
        values,
    )
    .await
    .unwrap();
}
