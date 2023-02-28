use crate::settings::Settings;
use lambda_web::{is_running_on_lambda, launch_rocket_on_lambda, LambdaError};
use std::env::var;

mod activitypub;
mod dynamodb;
mod faas_snowflake_id;
mod model;
mod routes;
pub mod rsa;
pub mod settings;
mod tracing;

#[rocket::main]
async fn main() -> Result<(), LambdaError> {
    tracing::init();

    let domain_name = var("CUSTOM_DOMAIN").unwrap();
    let rocket = rocket::build()
        .mount("/", routes::routes())
        .manage(Settings {
            base_url: format!("{}://{domain_name}", var("PROTOCOL").unwrap()),
            db_client: dynamodb::get_client().await,
            domain_name,
            node_id: faas_snowflake_id::get_node_id(),
            table_name: var("DYNAMODB_TABLE").unwrap(),
        });

    if is_running_on_lambda() {
        // Launch on AWS Lambda
        return launch_rocket_on_lambda(rocket).await;
    }

    // Launch local server
    let _ = rocket.launch().await?;
    Ok(())
}
