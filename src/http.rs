use anyhow::{Context, Result, anyhow, bail};
use serde::{Serialize, de::DeserializeOwned};
use ureq::config::IpFamily;

pub fn env_key(name: &str) -> Result<String> {
    std::env::var(name).map_err(|_| anyhow!("{name} is not set"))
}

pub fn post<T: DeserializeOwned>(
    url: &str,
    token: &str,
    headers: &[(&str, &str)],
    body: &impl Serialize,
) -> Result<T> {
    let send = |family| {
        let mut r = ureq::post(url)
            .config()
            .http_status_as_error(false)
            .ip_family(family)
            .build()
            .header("Authorization", format!("Bearer {token}"));
        for (k, v) in headers {
            r = r.header(*k, *v);
        }
        r.send_json(body)
    };
    // ureq has no Happy Eyeballs: a flaky IPv6 route stalls seconds on connect.
    // Try IPv4 first, fall back to any family (IPv6-only networks).
    // ponytail: no retry on 429/529, wrap in a backoff loop if it bites
    let mut res = send(IpFamily::Ipv4Only)
        .or_else(|_| send(IpFamily::Any))
        .with_context(|| format!("POST {url}"))?;
    let status = res.status();
    let text = res.body_mut().read_to_string()?;
    if !status.is_success() {
        bail!("HTTP {}: {text}", status.as_u16());
    }
    serde_json::from_str(&text).with_context(|| format!("unexpected response: {text}"))
}
