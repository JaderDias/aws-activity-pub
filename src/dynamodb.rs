use aws_sdk_dynamodb::error::{GetItemError, PutItemError};
use aws_sdk_dynamodb::model::AttributeValue;
use aws_sdk_dynamodb::model::{
    AttributeDefinition, KeySchemaElement, KeyType, ProvisionedThroughput, ScalarAttributeType,
};
use aws_sdk_dynamodb::types::SdkError;
use aws_sdk_dynamodb::Client;

pub const PARTITION_KEY_NAME: &str = "partition_key";
pub const SORT_KEY_NAME: &str = "sort_key";
pub const DEFAULT_TABLE_NAME: &str = "table_name";

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

async fn table_exists(client: &aws_sdk_dynamodb::Client, table: &str) -> bool {
    let table_list = client.list_tables().send().await.unwrap();
    println!("tables {table_list:?}");
    table_list
        .table_names()
        .as_ref()
        .unwrap()
        .contains(&table.into())
}

pub async fn create_table_if_not_exists(client: &aws_sdk_dynamodb::Client) {
    if table_exists(client, DEFAULT_TABLE_NAME).await {
        return;
    }

    let partition_key = PARTITION_KEY_NAME;
    let sort_key = SORT_KEY_NAME;

    let pad = AttributeDefinition::builder()
        .attribute_name(partition_key)
        .attribute_type(ScalarAttributeType::S)
        .build();

    let sad = AttributeDefinition::builder()
        .attribute_name(SORT_KEY_NAME)
        .attribute_type(ScalarAttributeType::S)
        .build();

    let pks = KeySchemaElement::builder()
        .attribute_name(partition_key)
        .key_type(KeyType::Hash)
        .build();

    let sks = KeySchemaElement::builder()
        .attribute_name(SORT_KEY_NAME)
        .key_type(KeyType::Range)
        .build();

    let pt = ProvisionedThroughput::builder()
        .read_capacity_units(10)
        .write_capacity_units(5)
        .build();

    client
        .create_table()
        .table_name(DEFAULT_TABLE_NAME)
        .key_schema(pks)
        .key_schema(sks)
        .attribute_definitions(pad)
        .attribute_definitions(sad)
        .provisioned_throughput(pt)
        .send()
        .await
        .unwrap();
}
