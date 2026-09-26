use super::Provider;
use crate::http;
use crate::model::{Request, Response};
use serde::Deserialize;
use serde_json::json;

pub struct Cloudflare;

/// Cloudflare's API usually wraps payloads in `{"result": ..., "success": ...}`.
#[derive(Deserialize)]
#[serde(untagged)]
enum Envelope {
    Wrapped { result: Response },
    Bare(Response),
}

impl Provider for Cloudflare {
    fn evaluate(&self, req: &Request) -> anyhow::Result<Response> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/ai/run",
            http::env_key("CLOUDFLARE_ACCOUNT_ID")?
        );
        let body = json!({"model": "typesafe/jev", "input": req});
        Ok(match http::post(&url, &http::env_key("CLOUDFLARE_API_TOKEN")?, &[], &body)? {
            Envelope::Wrapped { result } | Envelope::Bare(result) => result,
        })
    }
}
