use lambda_web::{is_running_on_lambda, launch_rocket_on_lambda, LambdaError};
use rand::prelude::*;

mod activitypub;
mod dynamodb;
mod model;
mod routes;
mod snowflake;

pub struct Settings {
    pub db_client: aws_sdk_dynamodb::Client,
    pub domain_name: String,
    pub node_id: u64,
    pub table_name: String,
}

#[rocket::main]
async fn main() -> Result<(), LambdaError> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let node_id: u64;
    {
        let mut rng = rand::thread_rng();
        node_id = rng.gen_range(0..1024);
    }

    let rocket = rocket::build()
        .mount("/", routes::routes())
        .manage(Settings {
            db_client: dynamodb::get_client().await,
            domain_name: std::env::var("CUSTOM_DOMAIN").unwrap(),
            node_id,
            table_name: std::env::var("DYNAMODB_TABLE").unwrap(),
        });

    if is_running_on_lambda() {
        // Launch on AWS Lambda
        return launch_rocket_on_lambda(rocket).await;
    }

    // Launch local server
    let _ = rocket.launch().await?;
    Ok(())
}
