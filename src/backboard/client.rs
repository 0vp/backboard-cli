use crate::backboard::models::{
    AddMessageRequest, Assistant, CreateAssistantRequest, MessageResponse,
    SubmitToolOutputsRequest, Thread, ToolOutput,
};
use anyhow::{anyhow, Context, Result};
use reqwest::multipart::{Form, Part};
use reqwest::{Client, Method};
use serde::de::DeserializeOwned;
use std::time::Duration;

#[derive(Clone)]
pub struct BackboardClient {
    api_key: String,
    base_url: String,
    http: Client,
}

impl BackboardClient {
    pub fn new(base_url: String, api_key: String, timeout: Duration) -> Result<Self> {
        let http = Client::builder()
            .timeout(timeout)
            .build()
            .context("failed to create reqwest client")?;
        Ok(Self {
            api_key,
            base_url: base_url.trim_end_matches('/').to_string(),
            http,
        })
    }

    pub async fn create_assistant(&self, request: CreateAssistantRequest) -> Result<Assistant> {
        self.request_json(Method::POST, "/assistants", Some(&request))
            .await
    }

    pub async fn create_thread(&self, assistant_id: &str) -> Result<Thread> {
        let path = format!("/assistants/{assistant_id}/threads");
        self.request_json(Method::POST, &path, Some(&serde_json::json!({})))
            .await
    }

    pub async fn add_message(&self, request: AddMessageRequest) -> Result<MessageResponse> {
        let path = format!("/threads/{}/messages", request.thread_id);
        let url = format!("{}{}", self.base_url, path);

        let form = Form::new()
            .part("content", Part::text(request.content))
            .part("llm_provider", Part::text(request.llm_provider))
            .part("model_name", Part::text(request.model_name))
            .part("memory", Part::text(request.memory))
            .part("web_search", Part::text(request.web_search))
            .part("stream", Part::text(request.stream.to_string()))
            .part("send_to_llm", Part::text(request.send_to_llm));

        let response = self
            .http
            .post(url)
            .header("X-API-Key", &self.api_key)
            .multipart(form)
            .send()
            .await
            .context("backboard add_message request failed")?;

        decode_response(response).await
    }

    pub async fn submit_tool_outputs(
        &self,
        thread_id: &str,
        run_id: &str,
        outputs: Vec<ToolOutput>,
    ) -> Result<MessageResponse> {
        let path = format!("/threads/{thread_id}/runs/{run_id}/submit-tool-outputs");
        let body = SubmitToolOutputsRequest {
            tool_outputs: outputs,
        };
        self.request_json(Method::POST, &path, Some(&body)).await
    }

    async fn request_json<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<&impl serde::Serialize>,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let mut request = self
            .http
            .request(method, url)
            .header("X-API-Key", &self.api_key);
        if let Some(payload) = body {
            request = request.json(payload);
        }

        let response = request.send().await.context("backboard request failed")?;
        decode_response(response).await
    }
}

async fn decode_response<T: DeserializeOwned>(response: reqwest::Response) -> Result<T> {
    let status = response.status();
    let body = response
        .text()
        .await
        .context("failed to read response body")?;

    if !status.is_success() {
        let preview = if body.len() > 350 {
            format!("{}...", &body[..350])
        } else {
            body
        };
        return Err(anyhow!(
            "backboard request failed ({}): {}",
            status.as_u16(),
            preview
        ));
    }

    serde_json::from_str::<T>(&body)
        .map_err(|err| anyhow!("failed to decode response JSON: {err}; body={body}"))
}
