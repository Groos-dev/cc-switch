use regex::Regex;
use std::sync::LazyLock;

use crate::config::write_text_file;
use crate::openclaw_config::get_openclaw_dir;
use crate::services::workspace::WorkspaceService;
use serde_json::Value;

const ALLOWED_FILES: &[&str] = &[
    "AGENTS.md",
    "SOUL.md",
    "USER.md",
    "IDENTITY.md",
    "TOOLS.md",
    "MEMORY.md",
    "HEARTBEAT.md",
    "BOOTSTRAP.md",
    "BOOT.md",
];

fn validate_filename(filename: &str) -> Result<(), String> {
    if !ALLOWED_FILES.contains(&filename) {
        return Err(format!(
            "Invalid workspace filename: {filename}. Allowed: {}",
            ALLOWED_FILES.join(", ")
        ));
    }
    Ok(())
}

static DAILY_MEMORY_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\d{4}-\d{2}-\d{2}\.md$").unwrap());

fn validate_daily_memory_filename(filename: &str) -> Result<(), String> {
    if !DAILY_MEMORY_RE.is_match(filename) {
        return Err(format!(
            "Invalid daily memory filename: {filename}. Expected: YYYY-MM-DD.md"
        ));
    }
    Ok(())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyMemoryFileInfo {
    pub filename: String,
    pub date: String,
    pub size_bytes: u64,
    pub modified_at: u64,
    pub preview: String,
}

#[tauri::command]
pub async fn workspace_ensure_layout() -> Result<bool, String> {
    WorkspaceService::ensure_workspace_layout()
        .map(|_| true)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn workspace_sync_all() -> Result<bool, String> {
    WorkspaceService::sync_all()
        .map(|_| true)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn workspace_sync_mcp() -> Result<bool, String> {
    WorkspaceService::sync_mcp_from_workspace()
        .map(|_| true)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn workspace_sync_skills() -> Result<bool, String> {
    WorkspaceService::sync_skills_from_workspace()
        .map(|_| true)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn workspace_sync_hooks() -> Result<bool, String> {
    WorkspaceService::sync_hooks_from_workspace()
        .map(|_| true)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn workspace_get_mcp_config(
) -> Result<crate::services::workspace::WorkspaceMcpFile, String> {
    WorkspaceService::get_mcp_config().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn workspace_upsert_mcp_server(
    id: String,
    name: Option<String>,
    enabled: Option<bool>,
    apps: Option<crate::McpApps>,
    server: Value,
) -> Result<bool, String> {
    WorkspaceService::upsert_mcp_server(id, name, enabled, apps, server)
        .map(|_| true)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn workspace_delete_mcp_server(id: String) -> Result<bool, String> {
    WorkspaceService::delete_mcp_server(&id)
        .map(|_| true)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn workspace_reorder_mcp_servers(server_ids: Vec<String>) -> Result<bool, String> {
    WorkspaceService::reorder_mcp_servers(server_ids)
        .map(|_| true)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn workspace_get_skills(
) -> Result<Vec<crate::services::workspace::WorkspaceSkill>, String> {
    WorkspaceService::get_skills().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn workspace_update_skill_apps(
    name: String,
    apps: crate::McpApps,
) -> Result<bool, String> {
    WorkspaceService::update_skill_apps(name, apps)
        .map(|_| true)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn workspace_get_hooks(
) -> Result<Vec<crate::services::workspace::WorkspaceHookApp>, String> {
    WorkspaceService::get_hooks().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn workspace_read_hook_file(file_path: String) -> Result<String, String> {
    std::fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read file {}: {}", file_path, e))
}

#[tauri::command]
pub async fn list_daily_memory_files() -> Result<Vec<DailyMemoryFileInfo>, String> {
    let memory_dir = get_openclaw_dir().join("workspace").join("memory");

    if !memory_dir.exists() {
        return Ok(Vec::new());
    }

    let mut files: Vec<DailyMemoryFileInfo> = Vec::new();
    let entries = std::fs::read_dir(&memory_dir)
        .map_err(|e| format!("Failed to read memory directory: {e}"))?;

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.ends_with(".md") {
            continue;
        }

        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        if !meta.is_file() {
            continue;
        }

        let date = name.trim_end_matches(".md").to_string();
        let size_bytes = meta.len();
        let modified_at = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let preview = std::fs::read_to_string(entry.path())
            .unwrap_or_default()
            .chars()
            .take(200)
            .collect::<String>();

        files.push(DailyMemoryFileInfo {
            filename: name,
            date,
            size_bytes,
            modified_at,
            preview,
        });
    }

    files.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));
    Ok(files)
}

#[tauri::command]
pub async fn read_daily_memory_file(filename: String) -> Result<Option<String>, String> {
    validate_daily_memory_filename(&filename)?;

    let path = get_openclaw_dir()
        .join("workspace")
        .join("memory")
        .join(&filename);

    if !path.exists() {
        return Ok(None);
    }

    std::fs::read_to_string(&path)
        .map(Some)
        .map_err(|e| format!("Failed to read daily memory file {filename}: {e}"))
}

#[tauri::command]
pub async fn write_daily_memory_file(filename: String, content: String) -> Result<(), String> {
    validate_daily_memory_filename(&filename)?;

    let memory_dir = get_openclaw_dir().join("workspace").join("memory");
    std::fs::create_dir_all(&memory_dir)
        .map_err(|e| format!("Failed to create memory directory: {e}"))?;

    let path = memory_dir.join(&filename);
    write_text_file(&path, &content)
        .map_err(|e| format!("Failed to write daily memory file {filename}: {e}"))
}

#[tauri::command]
pub async fn delete_daily_memory_file(filename: String) -> Result<(), String> {
    validate_daily_memory_filename(&filename)?;

    let path = get_openclaw_dir()
        .join("workspace")
        .join("memory")
        .join(&filename);

    if path.exists() {
        std::fs::remove_file(&path)
            .map_err(|e| format!("Failed to delete daily memory file {filename}: {e}"))?;
    }

    Ok(())
}

#[tauri::command]
pub async fn read_workspace_file(filename: String) -> Result<Option<String>, String> {
    validate_filename(&filename)?;

    let path = get_openclaw_dir().join("workspace").join(&filename);

    if !path.exists() {
        return Ok(None);
    }

    std::fs::read_to_string(&path)
        .map(Some)
        .map_err(|e| format!("Failed to read workspace file {filename}: {e}"))
}

#[tauri::command]
pub async fn write_workspace_file(filename: String, content: String) -> Result<(), String> {
    validate_filename(&filename)?;

    let workspace_dir = get_openclaw_dir().join("workspace");
    std::fs::create_dir_all(&workspace_dir)
        .map_err(|e| format!("Failed to create workspace directory: {e}"))?;

    let path = workspace_dir.join(&filename);
    write_text_file(&path, &content)
        .map_err(|e| format!("Failed to write workspace file {filename}: {e}"))
}
