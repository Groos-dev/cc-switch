use crate::config::get_app_config_dir;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

const STORE_VERSION: u32 = 1;
const DEFAULT_TTL_DAYS: i64 = 7;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContinuationScope {
    pub conversation_id: String,
    pub provider_id: String,
    pub model: String,
    pub storage_key: String,
    pub prompt_cache_key: String,
}

impl ContinuationScope {
    pub fn new(conversation_id: &str, provider_id: &str, model: &str) -> Self {
        let conversation_id = conversation_id.trim().to_string();
        let provider_id = provider_id.trim().to_string();
        let model = model.trim().to_string();
        let storage_key = format!("{provider_id}::{model}::{conversation_id}");
        let prompt_cache_key = conversation_id.clone();

        Self {
            conversation_id,
            provider_id,
            model,
            storage_key,
            prompt_cache_key,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContinuationRecord {
    pub conversation_id: String,
    pub provider_id: String,
    pub model: String,
    pub prompt_cache_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_response_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub envelope_hash: Option<String>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

impl ContinuationRecord {
    pub fn new(scope: &ContinuationScope) -> Self {
        let now = Utc::now();
        Self::with_timestamp(scope, now)
    }

    pub fn with_timestamp(scope: &ContinuationScope, now: DateTime<Utc>) -> Self {
        Self {
            conversation_id: scope.conversation_id.clone(),
            provider_id: scope.provider_id.clone(),
            model: scope.model.clone(),
            prompt_cache_key: scope.prompt_cache_key.clone(),
            latest_response_id: None,
            envelope_hash: None,
            updated_at: now,
            expires_at: Some(now + Duration::days(DEFAULT_TTL_DAYS)),
        }
    }

    pub fn matches_scope(&self, scope: &ContinuationScope) -> bool {
        self.conversation_id == scope.conversation_id
            && self.provider_id == scope.provider_id
            && self.model == scope.model
    }

    fn is_expired(&self, now: DateTime<Utc>) -> bool {
        self.expires_at.is_some_and(|expires_at| expires_at <= now)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ContinuationStoreFile {
    #[serde(default = "default_store_version")]
    version: u32,
    #[serde(default)]
    records: HashMap<String, ContinuationRecord>,
}

fn default_store_version() -> u32 {
    STORE_VERSION
}

#[derive(Debug, Clone)]
pub struct CodexContinuationStore {
    path: PathBuf,
}

impl Default for CodexContinuationStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CodexContinuationStore {
    pub fn new() -> Self {
        Self::with_path(default_store_path())
    }

    pub fn with_path(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn scope_for(
        &self,
        conversation_id: &str,
        provider_id: &str,
        model: &str,
    ) -> ContinuationScope {
        ContinuationScope::new(conversation_id, provider_id, model)
    }

    pub fn load(&self, scope: &ContinuationScope) -> std::io::Result<Option<ContinuationRecord>> {
        let mut file = self.read_store_file()?;
        let now = Utc::now();
        let record = file.records.get(&scope.storage_key).cloned();

        match record {
            Some(record) if record.matches_scope(scope) && !record.is_expired(now) => {
                Ok(Some(record))
            }
            Some(record) if record.is_expired(now) => {
                file.records.remove(&scope.storage_key);
                self.write_store_file(&file)?;
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    pub fn save(
        &self,
        scope: &ContinuationScope,
        mut record: ContinuationRecord,
    ) -> std::io::Result<()> {
        let mut file = self.read_store_file()?;
        let now = Utc::now();

        record.conversation_id = scope.conversation_id.clone();
        record.provider_id = scope.provider_id.clone();
        record.model = scope.model.clone();
        record.prompt_cache_key = scope.prompt_cache_key.clone();
        if record.updated_at.timestamp() == 0 {
            record.updated_at = now;
        }
        if record.expires_at.is_none() {
            record.expires_at = Some(record.updated_at + Duration::days(DEFAULT_TTL_DAYS));
        }

        prune_expired_records(&mut file.records, now);
        file.records.insert(scope.storage_key.clone(), record);
        self.write_store_file(&file)
    }

    pub fn remove(&self, scope: &ContinuationScope) -> std::io::Result<()> {
        let mut file = self.read_store_file()?;
        file.records.remove(&scope.storage_key);
        self.write_store_file(&file)
    }

    fn read_store_file(&self) -> std::io::Result<ContinuationStoreFile> {
        if !self.path.exists() {
            return Ok(ContinuationStoreFile::default());
        }

        let content = fs::read_to_string(&self.path)?;
        let parsed = serde_json::from_str::<ContinuationStoreFile>(&content)
            .unwrap_or_else(|_| ContinuationStoreFile::default());
        Ok(parsed)
    }

    fn write_store_file(&self, file: &ContinuationStoreFile) -> std::io::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(file)?;
        let parent = self
            .path
            .parent()
            .ok_or_else(|| std::io::Error::other("invalid continuation store path"))?;
        let file_name = self
            .path
            .file_name()
            .ok_or_else(|| std::io::Error::other("invalid continuation store file name"))?
            .to_string_lossy()
            .to_string();
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let tmp_path = parent.join(format!("{file_name}.tmp.{ts}"));

        #[cfg(unix)]
        {
            use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

            let mut tmp = fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .mode(0o600)
                .open(&tmp_path)?;
            tmp.write_all(content.as_bytes())?;
            tmp.flush()?;

            fs::rename(&tmp_path, &self.path)?;
            fs::set_permissions(&self.path, fs::Permissions::from_mode(0o600))?;
        }

        #[cfg(windows)]
        {
            let mut tmp = fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&tmp_path)?;
            tmp.write_all(content.as_bytes())?;
            tmp.flush()?;

            if self.path.exists() {
                let _ = fs::remove_file(&self.path);
            }
            fs::rename(&tmp_path, &self.path)?;
        }

        Ok(())
    }
}

fn default_store_path() -> PathBuf {
    get_app_config_dir()
        .join("proxy")
        .join("codex_continuation_store.json")
}

fn prune_expired_records(records: &mut HashMap<String, ContinuationRecord>, now: DateTime<Utc>) {
    records.retain(|_, record| !record.is_expired(now));
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_scope_is_stable_for_same_namespace() {
        let scope1 = ContinuationScope::new("session-1", "provider-a", "gpt-5");
        let scope2 = ContinuationScope::new("session-1", "provider-a", "gpt-5");

        assert_eq!(scope1.storage_key, scope2.storage_key);
        assert_eq!(scope1.prompt_cache_key, scope2.prompt_cache_key);
        assert_eq!(scope1.prompt_cache_key, "session-1");
    }

    #[test]
    fn test_scope_changes_when_provider_or_model_changes() {
        let base = ContinuationScope::new("session-1", "provider-a", "gpt-5");
        let other_provider = ContinuationScope::new("session-1", "provider-b", "gpt-5");
        let other_model = ContinuationScope::new("session-1", "provider-a", "gpt-5-mini");

        assert_ne!(base.storage_key, other_provider.storage_key);
        assert_ne!(base.storage_key, other_model.storage_key);
        assert_eq!(base.prompt_cache_key, other_provider.prompt_cache_key);
        assert_eq!(base.prompt_cache_key, other_model.prompt_cache_key);
    }

    #[test]
    fn test_store_round_trip() {
        let dir = tempdir().unwrap();
        let store = CodexContinuationStore::with_path(dir.path().join("continuations.json"));
        let scope = store.scope_for("claude-session-1", "provider-a", "gpt-5");
        let mut record = ContinuationRecord::new(&scope);
        record.latest_response_id = Some("resp_123".to_string());
        record.envelope_hash = Some("hash_abc".to_string());

        store.save(&scope, record.clone()).unwrap();
        let loaded = store.load(&scope).unwrap().unwrap();

        assert_eq!(loaded.latest_response_id, Some("resp_123".to_string()));
        assert_eq!(loaded.envelope_hash, Some("hash_abc".to_string()));
        assert_eq!(loaded.prompt_cache_key, scope.prompt_cache_key);
    }

    #[test]
    fn test_store_prunes_expired_record_on_load() {
        let dir = tempdir().unwrap();
        let store = CodexContinuationStore::with_path(dir.path().join("continuations.json"));
        let scope = store.scope_for("claude-session-1", "provider-a", "gpt-5");
        let mut record =
            ContinuationRecord::with_timestamp(&scope, Utc::now() - Duration::days(10));
        record.expires_at = Some(Utc::now() - Duration::days(1));

        store.save(&scope, record).unwrap();

        assert!(store.load(&scope).unwrap().is_none());

        let raw = fs::read_to_string(store.path()).unwrap();
        let parsed: ContinuationStoreFile = serde_json::from_str(&raw).unwrap();
        assert!(parsed.records.is_empty());
    }

    #[test]
    fn test_store_remove() {
        let dir = tempdir().unwrap();
        let store = CodexContinuationStore::with_path(dir.path().join("continuations.json"));
        let scope = store.scope_for("claude-session-1", "provider-a", "gpt-5");

        store.save(&scope, ContinuationRecord::new(&scope)).unwrap();
        store.remove(&scope).unwrap();

        assert!(store.load(&scope).unwrap().is_none());
    }
}
