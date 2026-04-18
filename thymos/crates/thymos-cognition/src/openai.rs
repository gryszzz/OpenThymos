//! OpenAI Chat Completions API cognition adapter.
//!
//! Mirrors the Anthropic adapter pattern: the model emits `tool_calls` in
//! assistant messages, which this adapter translates into `Intent`s.
//! Committed/rejected outcomes are fed back as `tool` messages on the next turn.
//!
//! Environment:
//!   * `OPENAI_API_KEY` — required.
//!   * `OPENAI_MODEL` — optional, defaults to `gpt-4o`.
//!   * `OPENAI_BASE_URL` — optional, for local/custom endpoints (e.g. Ollama, vLLM).

use std::collections::HashMap;
use std::time::Duration;

use serde_json::{json, Value};

use thymos_core::{
    error::{Error, Result},
    ids::IntentId,
    intent::{Intent, IntentBody, IntentKind},
    proposal::RejectionReason,
    writ::Writ,
};
use thymos_tools::ToolRegistry;

use crate::{Cognition, CognitionContext, CognitionStep, HistoryItem};

pub const DEFAULT_MODEL: &str = "gpt-4o";
const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";

pub struct OpenAiCognition {
    client: reqwest::blocking::Client,
    api_key: String,
    base_url: String,
    model: String,
    max_tokens: u32,
    /// Accumulated messages (OpenAI chat format).
    messages: Vec<Value>,
    /// tool_call_id → Intent ID correlation.
    correlations: HashMap<IntentId, String>,
    /// tool_call_ids from the last assistant message that need responses.
    last_tool_call_ids: Vec<String>,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
}

impl OpenAiCognition {
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| Error::Other("OPENAI_API_KEY is not set".into()))?;
        let base_url =
            std::env::var("OPENAI_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.into());
        let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.into());
        Self::new(api_key, base_url, model)
    }

    pub fn new(api_key: String, base_url: String, model: String) -> Result<Self> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .map_err(|e| Error::Other(format!("reqwest client build: {e}")))?;
        Ok(OpenAiCognition {
            client,
            api_key,
            base_url,
            model,
            max_tokens: 4096,
            messages: Vec::new(),
            correlations: HashMap::new(),
            last_tool_call_ids: Vec::new(),
            total_input_tokens: 0,
            total_output_tokens: 0,
        })
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }
}

impl Cognition for OpenAiCognition {
    fn step(&mut self, ctx: &CognitionContext<'_>) -> Result<CognitionStep> {
        // 1. First turn: seed with system + user message.
        //    Subsequent turns: send tool results for all outstanding tool_call_ids.
        if self.messages.is_empty() {
            let system_prompt = build_system_prompt(ctx.writ);
            self.messages.push(json!({
                "role": "system",
                "content": system_prompt,
            }));
            let user_content = build_opening_user_message(ctx);
            self.messages.push(json!({
                "role": "user",
                "content": user_content,
            }));
        } else {
            // Append tool results for each outstanding tool_call_id.
            let tool_messages =
                build_tool_results(ctx, &self.last_tool_call_ids, &mut self.correlations);
            self.messages.extend(tool_messages);
        }

        // 2. Build request.
        let tools_payload = build_tools_payload(ctx.tools);
        let mut req_body = json!({
            "model": self.model,
            "max_tokens": self.max_tokens,
            "messages": self.messages,
        });
        if !tools_payload.is_empty() {
            req_body["tools"] = json!(tools_payload);
        }

        // 3. POST.
        let url = format!("{}/chat/completions", self.base_url);
        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&req_body)
            .send()
            .map_err(|e| Error::Other(format!("openai request failed: {e}")))?;

        let status = resp.status();
        let resp_json: Value = resp
            .json()
            .map_err(|e| Error::Other(format!("openai response parse: {e}")))?;
        if !status.is_success() {
            return Err(Error::Other(format!(
                "openai API error {status}: {resp_json}"
            )));
        }

        // 4. Usage.
        if let Some(usage) = resp_json.get("usage") {
            self.total_input_tokens += usage
                .get("prompt_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            self.total_output_tokens += usage
                .get("completion_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
        }

        // 5. Parse the first choice.
        let choice = resp_json
            .get("choices")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .ok_or_else(|| Error::Other("openai: no choices in response".into()))?;

        let message = choice
            .get("message")
            .ok_or_else(|| Error::Other("openai: no message in choice".into()))?;

        // Append assistant message to history.
        self.messages.push(message.clone());

        let finish_reason = choice
            .get("finish_reason")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // 6. Extract tool_calls as Intents.
        let mut intents: Vec<Intent> = Vec::new();
        let mut new_tool_call_ids: Vec<String> = Vec::new();
        let text_content = message
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if let Some(tool_calls) = message.get("tool_calls").and_then(|v| v.as_array()) {
            for tc in tool_calls {
                let tc_id = tc
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Error::Other("tool_call missing id".into()))?
                    .to_string();
                let function = tc
                    .get("function")
                    .ok_or_else(|| Error::Other("tool_call missing function".into()))?;
                let name = function
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Error::Other("tool_call function missing name".into()))?
                    .to_string();
                let args_str = function
                    .get("arguments")
                    .and_then(|v| v.as_str())
                    .unwrap_or("{}");
                let args: Value = serde_json::from_str(args_str)
                    .map_err(|e| Error::Other(format!("tool_call args parse: {e}")))?;

                let mut nonce = [0u8; 16];
                for (i, b) in tc_id.as_bytes().iter().take(16).enumerate() {
                    nonce[i] = *b;
                }

                let intent = Intent::new(IntentBody {
                    parent_commit: None,
                    author: format!("openai:{}", self.model),
                    kind: IntentKind::Act,
                    target: name,
                    args,
                    rationale: text_content.clone(),
                    nonce,
                })?;

                self.correlations.insert(intent.id, tc_id.clone());
                new_tool_call_ids.push(tc_id);
                intents.push(intent);
            }
        }

        self.last_tool_call_ids = new_tool_call_ids;

        let final_answer = if intents.is_empty() && finish_reason == "stop" {
            if text_content.is_empty() {
                None
            } else {
                Some(text_content)
            }
        } else {
            None
        };

        Ok(CognitionStep {
            intents,
            final_answer,
        })
    }
}

// ---------- helpers ----------------------------------------------------------

fn build_system_prompt(writ: &Writ) -> String {
    let mut scopes: Vec<String> = writ
        .body
        .tool_scopes
        .iter()
        .map(|p| p.tool.clone())
        .collect();
    scopes.sort();
    let scopes_str = scopes.join(", ");
    let b = &writ.body.budget;
    format!(
        "You are a cognition process operating inside the Thymos runtime.\n\
         \n\
         You do not take actions directly. You emit function calls that describe \
         PROPOSED actions. The runtime evaluates policy against a bounded Capability \
         Writ, and either commits the effect or rejects the proposal.\n\
         \n\
         Constraints:\n\
         - You may only call tools matching your Writ scope: [{scopes_str}].\n\
         - Budget: {tokens} tokens, {calls} tool calls, ~{usd} USD (millicents), {time}ms wall-clock.\n\
         - If a proposal is rejected, you will see the reason in the tool response. Adjust accordingly.\n\
         - When done, reply with plain text and no function calls to signal completion.",
        tokens = b.tokens,
        calls = b.tool_calls,
        usd = b.usd_millicents,
        time = b.wall_clock_ms,
    )
}

fn build_opening_user_message(ctx: &CognitionContext<'_>) -> String {
    let world_summary = if ctx.world.resources.is_empty() {
        "(empty)".into()
    } else {
        let mut lines = Vec::new();
        for (key, state) in &ctx.world.resources {
            let v =
                serde_json::to_string(&state.value).unwrap_or_else(|_| "<unprintable>".into());
            lines.push(format!(
                "  {}:{} v{} = {}",
                key.kind, key.id, state.version, v
            ));
        }
        lines.join("\n")
    };
    format!(
        "Task: {}\n\nCurrent world state:\n{}\n\nProceed.",
        ctx.task, world_summary
    )
}

fn build_tools_payload(tools: &ToolRegistry) -> Vec<Value> {
    let mut out = Vec::new();
    for name in tools.names() {
        if let Ok(tool) = tools.get(name) {
            out.push(json!({
                "type": "function",
                "function": {
                    "name": tool.meta().name,
                    "description": tool.description(),
                    "parameters": tool.input_schema(),
                }
            }));
        }
    }
    out
}

fn build_tool_results(
    ctx: &CognitionContext<'_>,
    expected_tool_call_ids: &[String],
    correlations: &mut HashMap<IntentId, String>,
) -> Vec<Value> {
    // Index history by Intent id.
    let mut outcomes: HashMap<IntentId, HistoryOutcome> = HashMap::new();
    for item in &ctx.since_last {
        match item {
            HistoryItem::Committed {
                intent,
                observation,
            } => {
                outcomes.insert(
                    intent.id,
                    HistoryOutcome::Committed(observation.output.clone()),
                );
            }
            HistoryItem::Rejected { intent, reason } => {
                outcomes.insert(intent.id, HistoryOutcome::Rejected(reason.clone()));
            }
        }
    }

    // For every tool_call_id, produce a tool message.
    let mut messages = Vec::with_capacity(expected_tool_call_ids.len());
    for tc_id in expected_tool_call_ids {
        let matching_intent_id = correlations
            .iter()
            .find(|(_, v)| *v == tc_id)
            .map(|(k, _)| *k);

        let content = match matching_intent_id.and_then(|id| outcomes.remove(&id)) {
            Some(HistoryOutcome::Committed(output)) => {
                format!(
                    "Committed. Observation:\n{}",
                    serde_json::to_string_pretty(&output).unwrap_or_default()
                )
            }
            Some(HistoryOutcome::Rejected(reason)) => {
                format!("Rejected by runtime. Reason: {reason:?}")
            }
            None => {
                "Proposal was not executed this turn (runtime deferred or suspended).".into()
            }
        };

        messages.push(json!({
            "role": "tool",
            "tool_call_id": tc_id,
            "content": content,
        }));
    }

    correlations.retain(|_, v| !expected_tool_call_ids.contains(v));
    messages
}

enum HistoryOutcome {
    Committed(serde_json::Value),
    Rejected(RejectionReason),
}
