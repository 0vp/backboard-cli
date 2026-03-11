use crate::tools::builtins::utils::{get_string, get_usize};
use crate::tools::registry::ExecutionContext;
use anyhow::{bail, Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde_json::{json, Value};
use url::Url;

pub async fn websearch(args: Value, ctx: &ExecutionContext) -> Result<Value> {
    let query = get_string(&args, "query").unwrap_or_default();
    if query.trim().is_empty() {
        bail!("query is required");
    }

    let api_key = ctx
        .jina_api_key
        .clone()
        .filter(|v| !v.trim().is_empty())
        .context("JINA_API_KEY is required for websearch")?;

    let max_bytes = get_usize(&args, "max_bytes", 30_000);
    let url = format!("https://s.jina.ai/?q={}", urlencoding::encode(query.trim()));

    let mut headers = auth_headers(&api_key)?;
    headers.insert("X-Respond-With", HeaderValue::from_static("no-content"));
    perform_web_request(url, headers, max_bytes, "query", query).await
}

pub async fn web_fetch(args: Value, ctx: &ExecutionContext) -> Result<Value> {
    let target = get_string(&args, "url").unwrap_or_default();
    if target.trim().is_empty() {
        bail!("url is required");
    }

    let parsed = Url::parse(target.trim()).context("invalid url")?;
    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        bail!("url must start with http:// or https://");
    }

    let api_key = ctx
        .jina_api_key
        .clone()
        .filter(|v| !v.trim().is_empty())
        .context("JINA_API_KEY is required for web_fetch")?;

    let max_bytes = get_usize(&args, "max_bytes", 40_000);
    let url = format!("https://r.jina.ai/{}", target.trim());
    let headers = auth_headers(&api_key)?;
    perform_web_request(url, headers, max_bytes, "url", target).await
}

fn auth_headers(api_key: &str) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    let value = format!("Bearer {api_key}");
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&value).context("failed to construct authorization header")?,
    );
    Ok(headers)
}

async fn perform_web_request(
    url: String,
    headers: HeaderMap,
    max_bytes: usize,
    key: &str,
    value: String,
) -> Result<Value> {
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .headers(headers)
        .send()
        .await
        .context("web request failed")?;

    let status_code = response.status().as_u16();
    let bytes = response
        .bytes()
        .await
        .context("failed to read web response body")?;

    let truncated = bytes.len() > max_bytes;
    let body = if truncated {
        &bytes[..max_bytes]
    } else {
        bytes.as_ref()
    };
    let content = String::from_utf8_lossy(body).to_string();

    if !(200..300).contains(&status_code) {
        bail!("jina request failed ({status_code}): {content}");
    }

    Ok(json!({
        key: value,
        "status_code": status_code,
        "truncated": truncated,
        "content": content
    }))
}
