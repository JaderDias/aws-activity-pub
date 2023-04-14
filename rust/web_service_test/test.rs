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
    pub cross_request_replace: Option<Replace>,
    pub response_replace: Option<Vec<Replace>>,
}

#[derive(Deserialize)]
pub struct Replace {
    pub regex: String,
    pub placeholder: String,
}
