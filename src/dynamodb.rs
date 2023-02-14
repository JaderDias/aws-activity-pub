use aws_sdk_dynamodb::error::{GetItemError, PutItemError};
use aws_sdk_dynamodb::model::AttributeValue;
use aws_sdk_dynamodb::types::SdkError;
use aws_sdk_dynamodb::Client;

pub const PARTITION_KEY_NAME: &str = "partition_key";
pub const SORT_KEY_NAME: &str = "sort_key";

pub type GetItemResult = Result<aws_sdk_dynamodb::output::GetItemOutput, SdkError<GetItemError>>;
pub type PutItemResult = Result<aws_sdk_dynamodb::output::PutItemOutput, SdkError<PutItemError>>;

/// # Errors
///
/// Will return `Err` if a connection to the database is no properly established.
pub async fn get_item(
    client: &Client,
    dynamodb_table_name: &str,
    partition: &str,
    sort_value: &str,
) -> GetItemResult {
    client
        .get_item()
        .table_name(dynamodb_table_name)
        .key(PARTITION_KEY_NAME, AttributeValue::S(partition.to_owned()))
        .key(SORT_KEY_NAME, AttributeValue::S(sort_value.to_owned()))
        .send()
        .await
}

/// # Errors
///
/// Will return `Err` if a connection to the database is no properly established.
pub async fn put_item<S: std::hash::BuildHasher>(
    client: &Client,
    dynamodb_table_name: &str,
    partition: &str,
    sort_value: &str,
    values: std::collections::HashMap<String, AttributeValue, S>,
) -> PutItemResult {
    let mut table = client
        .put_item()
        .table_name(dynamodb_table_name)
        .item(PARTITION_KEY_NAME, AttributeValue::S(partition.to_owned()))
        .item(SORT_KEY_NAME, AttributeValue::S(sort_value.to_owned()));
    for (key, value) in values {
        table = table.item(key, value);
    }

    table.send().await
}

pub async fn get_client() -> Client {
    if let Ok(url) = std::env::var("LOCAL_DYNAMODB_URL") {
        println!("Using local dynamodb at {url}");
        get_local_client(url).await
    } else {
        let config = aws_config::load_from_env().await;
        Client::new(&config)
    }
}

pub async fn get_local_client(local_dynamodb_url: String) -> Client {
    std::env::set_var("AWS_ACCESS_KEY_ID", "DUMMYIDEXAMPLE");
    std::env::set_var("AWS_SECRET_ACCESS_KEY", "DUMMYEXAMPLEKEY");
    let config = aws_config::from_env().region("us-east-1").load().await;
    let dynamodb_local_config = aws_sdk_dynamodb::config::Builder::from(&config)
        .endpoint_url(local_dynamodb_url)
        .build();
    Client::from_conf(dynamodb_local_config)
}
