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
