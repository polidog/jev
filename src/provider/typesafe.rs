use super::Provider;
use crate::http;
use crate::model::{Request, Response};
use serde_json::json;

pub struct TypeSafe;

impl Provider for TypeSafe {
    fn evaluate(&self, req: &Request) -> anyhow::Result<Response> {
        let body = json!({"model": "jev-latest", "state": req.state, "questions": req.questions});
        http::post("https://api.typesafe.ai/v1/systemone", &http::env_key("TYPESAFE_API_KEY")?, &[], &body)
    }
}
