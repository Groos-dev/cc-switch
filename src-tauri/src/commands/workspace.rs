use crate::services::workspace::WorkspaceService;
use serde_json::Value;

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

// ========== MCP Configuration Management ==========

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

// ========== Skills Management ==========

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

// ========== Hooks Management ==========

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
