use aws_lambda_events::apigw::{ApiGatewayV2httpRequest, ApiGatewayV2httpResponse};

use serde::Deserialize;
use serde_json::Value;

pub type TestCases = Vec<TestCase>;

#[derive(Deserialize)]
pub struct TestCase {
    pub name: String,
    pub request: ApiGatewayV2httpRequest,
    pub request_body_json: Option<Value>,
    pub expected_response: ApiGatewayV2httpResponse,
    pub expected_body_json: Option<Value>,
    pub regex: Option<String>,
    pub placeholder: Option<String>,
}
