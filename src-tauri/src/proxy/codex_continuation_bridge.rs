use crate::provider::Provider;
use crate::proxy::codex_continuation_store::{CodexContinuationStore, ContinuationRecord};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

pub const PRIVATE_PROMPT_CACHE_KEY_FIELD: &str = "_cc_switch_prompt_cache_key";
pub const PRIVATE_PREVIOUS_RESPONSE_ID_FIELD: &str = "_cc_switch_previous_response_id";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeMetadata {
    pub conversation_id: String,
    pub provider_id: String,
    pub prompt_cache_key: String,
    pub envelope_hash: String,
    pub model: String,
}

pub fn bridge_metadata_for_responses_request(
    request: &Value,
    session_id: &str,
    provider_id: &str,
    explicit_prompt_cache_key: Option<&str>,
) -> BridgeMetadata {
    bridge_metadata_for_model(
        request,
        session_id,
        provider_id,
        explicit_prompt_cache_key,
        None,
    )
}

fn bridge_metadata_for_model(
    request: &Value,
    session_id: &str,
    provider_id: &str,
    explicit_prompt_cache_key: Option<&str>,
    model_override: Option<&str>,
) -> BridgeMetadata {
    let model = request.get("model").and_then(|v| v.as_str());
    let model = model_override.or(model).unwrap_or("unknown").to_string();
    let store = CodexContinuationStore::new();
    let scope = store.scope_for(session_id, provider_id, &model);
    let prompt_cache_key = explicit_prompt_cache_key
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| scope.prompt_cache_key.clone());
    let envelope_hash = hash_request_envelope(request);

    BridgeMetadata {
        conversation_id: scope.conversation_id,
        provider_id: scope.provider_id,
        prompt_cache_key,
        envelope_hash,
        model,
    }
}

#[cfg(test)]
fn apply_request_bridge(request: &mut Value, metadata: &BridgeMetadata) -> Option<String> {
    request["prompt_cache_key"] = json!(metadata.prompt_cache_key.clone());

    load_previous_response_id(metadata).inspect(|previous_response_id| {
        request["previous_response_id"] = json!(previous_response_id.clone());
    })
}

pub fn prepare_claude_responses_request(body: &mut Value, provider: &Provider, session_id: &str) {
    let explicit_prompt_cache_key = provider
        .meta
        .as_ref()
        .and_then(|meta| meta.prompt_cache_key.as_deref());
    let metadata = bridge_metadata_for_responses_request(
        body,
        session_id,
        &provider.id,
        explicit_prompt_cache_key,
    );

    body[PRIVATE_PROMPT_CACHE_KEY_FIELD] = json!(metadata.prompt_cache_key);

    if let Some(previous_response_id) = load_previous_response_id(&metadata) {
        body[PRIVATE_PREVIOUS_RESPONSE_ID_FIELD] = json!(previous_response_id);
    }
}

#[cfg(test)]
fn store_response_id(metadata: &BridgeMetadata, response_id: &str) -> std::io::Result<()> {
    persist_metadata_response(metadata, response_id)
}

pub fn persist_responses_response(
    request: &Value,
    provider: &Provider,
    session_id: &str,
    response_id: &str,
    response_model: Option<&str>,
) -> std::io::Result<()> {
    let explicit_prompt_cache_key = provider
        .meta
        .as_ref()
        .and_then(|meta| meta.prompt_cache_key.as_deref());
    let metadata = bridge_metadata_for_model(
        request,
        session_id,
        &provider.id,
        explicit_prompt_cache_key,
        response_model,
    );
    persist_metadata_response(&metadata, response_id)
}

fn persist_metadata_response(metadata: &BridgeMetadata, response_id: &str) -> std::io::Result<()> {
    if response_id.trim().is_empty() {
        return Ok(());
    }

    let store = CodexContinuationStore::new();
    let scope = metadata_scope(&store, metadata);
    let mut record = store
        .load(&scope)?
        .unwrap_or_else(|| ContinuationRecord::new(&scope));
    record.prompt_cache_key = metadata.prompt_cache_key.clone();
    record.latest_response_id = Some(response_id.trim().to_string());
    record.envelope_hash = Some(metadata.envelope_hash.clone());
    store.save(&scope, record)
}

fn load_previous_response_id(metadata: &BridgeMetadata) -> Option<String> {
    let store = CodexContinuationStore::new();
    let scope = metadata_scope(&store, metadata);
    let record = store.load(&scope).ok()??;
    if record.envelope_hash.as_deref() != Some(metadata.envelope_hash.as_str()) {
        return None;
    }
    record
        .latest_response_id
        .filter(|response_id| !response_id.trim().is_empty())
}

fn metadata_scope(
    store: &CodexContinuationStore,
    metadata: &BridgeMetadata,
) -> crate::proxy::codex_continuation_store::ContinuationScope {
    store.scope_for(
        &metadata.conversation_id,
        &metadata.provider_id,
        &metadata.model,
    )
}

fn hash_request_envelope(request: &Value) -> String {
    let envelope = json!({
        "model": request.get("model").cloned(),
        "instructions": request.get("instructions").cloned(),
        "tools": request.get("tools").cloned(),
        "tool_choice": request.get("tool_choice").cloned(),
        "reasoning": request.get("reasoning").cloned(),
        "temperature": request.get("temperature").cloned(),
        "top_p": request.get("top_p").cloned()
    });
    sha256_hex(
        serde_json::to_string(&envelope)
            .unwrap_or_default()
            .as_bytes(),
    )
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::{Provider, ProviderMeta};
    use crate::proxy::codex_continuation_store::CodexContinuationStore;
    use tempfile::tempdir;

    fn sample_request() -> Value {
        json!({
            "model": "gpt-5",
            "instructions": "You are helpful.",
            "tools": [{
                "type": "function",
                "name": "get_weather",
                "parameters": {"type": "object"}
            }],
            "tool_choice": "auto"
        })
    }

    fn provider_with_prompt_cache_key(prompt_cache_key: Option<&str>) -> Provider {
        Provider {
            id: "provider-a".to_string(),
            name: "Provider A".to_string(),
            settings_config: json!({}),
            website_url: None,
            category: Some("claude".to_string()),
            created_at: None,
            sort_index: None,
            notes: None,
            meta: Some(ProviderMeta {
                prompt_cache_key: prompt_cache_key.map(ToOwned::to_owned),
                api_format: Some("openai_responses".to_string()),
                ..Default::default()
            }),
            icon: None,
            icon_color: None,
            in_failover_queue: false,
        }
    }

    #[test]
    fn metadata_is_stable_for_same_conversation_namespace() {
        let request = sample_request();
        let first =
            bridge_metadata_for_responses_request(&request, "session-1", "provider-a", None);
        let second =
            bridge_metadata_for_responses_request(&request, "session-1", "provider-a", None);

        assert_eq!(first.conversation_id, second.conversation_id);
        assert_eq!(first.provider_id, second.provider_id);
        assert_eq!(first.prompt_cache_key, second.prompt_cache_key);
        assert_eq!(first.envelope_hash, second.envelope_hash);
    }

    #[test]
    #[serial_test::serial]
    fn apply_request_bridge_injects_previous_response_id_when_envelope_matches() {
        let dir = tempdir().unwrap();
        std::env::set_var("CC_SWITCH_TEST_HOME", dir.path());
        let request = sample_request();
        let metadata =
            bridge_metadata_for_responses_request(&request, "session-1", "provider-a", None);

        store_response_id(&metadata, "resp_123").unwrap();

        let mut follow_up = request.clone();
        let follow_up_metadata =
            bridge_metadata_for_responses_request(&follow_up, "session-1", "provider-a", None);
        let injected = apply_request_bridge(&mut follow_up, &follow_up_metadata);

        assert_eq!(injected.as_deref(), Some("resp_123"));
        assert_eq!(follow_up["previous_response_id"], "resp_123");
        assert_eq!(follow_up["prompt_cache_key"], metadata.prompt_cache_key);
        std::env::remove_var("CC_SWITCH_TEST_HOME");
    }

    #[test]
    #[serial_test::serial]
    fn apply_request_bridge_skips_injection_when_envelope_drifts() {
        let dir = tempdir().unwrap();
        std::env::set_var("CC_SWITCH_TEST_HOME", dir.path());
        let request = sample_request();
        let metadata =
            bridge_metadata_for_responses_request(&request, "session-1", "provider-a", None);

        store_response_id(&metadata, "resp_123").unwrap();

        let mut changed = sample_request();
        changed["instructions"] = json!("You are more strict now.");
        let changed_metadata =
            bridge_metadata_for_responses_request(&changed, "session-1", "provider-a", None);
        let injected = apply_request_bridge(&mut changed, &changed_metadata);

        assert!(injected.is_none());
        assert!(changed.get("previous_response_id").is_none());
        assert_eq!(changed["prompt_cache_key"], metadata.prompt_cache_key);
        std::env::remove_var("CC_SWITCH_TEST_HOME");
    }

    #[test]
    #[serial_test::serial]
    fn test_persist_response_uses_response_model_namespace() {
        let dir = tempdir().unwrap();
        std::env::set_var("CC_SWITCH_TEST_HOME", dir.path());

        let provider = provider_with_prompt_cache_key(None);
        let body = json!({
            "model": "gpt-5",
            "system": "same",
            "messages": [{"role": "user", "content": "hello"}]
        });

        persist_responses_response(
            &body,
            &provider,
            "session-1",
            "resp_new",
            Some("gpt-5-mini"),
        )
        .unwrap();

        let store = CodexContinuationStore::new();
        let mini_scope = store.scope_for("session-1", "provider-a", "gpt-5-mini");
        let default_scope = store.scope_for("session-1", "provider-a", "gpt-5");

        let mini_record = store.load(&mini_scope).unwrap().unwrap();
        assert_eq!(mini_record.latest_response_id.as_deref(), Some("resp_new"));
        assert!(store.load(&default_scope).unwrap().is_none());

        std::env::remove_var("CC_SWITCH_TEST_HOME");
    }

    #[test]
    #[serial_test::serial]
    fn test_persist_response_ignores_empty_response_id() {
        let dir = tempdir().unwrap();
        std::env::set_var("CC_SWITCH_TEST_HOME", dir.path());

        let provider = provider_with_prompt_cache_key(None);
        let body = json!({
            "model": "gpt-5",
            "messages": [{"role": "user", "content": "hello"}]
        });

        persist_responses_response(&body, &provider, "session-1", "   ", Some("gpt-5")).unwrap();

        let store = CodexContinuationStore::new();
        let scope = store.scope_for("session-1", "provider-a", "gpt-5");
        assert!(store.load(&scope).unwrap().is_none());

        std::env::remove_var("CC_SWITCH_TEST_HOME");
    }
}
