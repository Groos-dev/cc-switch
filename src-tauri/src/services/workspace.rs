use crate::config::get_home_dir;
use crate::error::AppError;
use crate::{mcp, McpApps, MultiAppConfig};
use crate::{AppType, SkillService};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub struct WorkspaceService;

impl WorkspaceService {
    pub fn workspace_root() -> PathBuf {
        get_home_dir().join(".cc-switch").join("data")
    }

    pub fn skills_dir() -> PathBuf {
        Self::workspace_root().join("skills")
    }

    pub fn mcp_dir() -> PathBuf {
        Self::workspace_root().join("mcp")
    }

    pub fn mcp_file_path() -> PathBuf {
        Self::mcp_dir().join("mcp.json")
    }

    pub fn hooks_root() -> PathBuf {
        Self::workspace_root().join("hooks")
    }

    pub fn hooks_app_dir(app: &str) -> PathBuf {
        Self::hooks_root().join(app)
    }

    pub fn ensure_workspace_layout() -> Result<(), AppError> {
        let root = Self::workspace_root();
        std::fs::create_dir_all(root.join("skills")).map_err(|e| AppError::io(&root, e))?;
        std::fs::create_dir_all(root.join("mcp")).map_err(|e| AppError::io(&root, e))?;
        for app in ["claude", "codex", "gemini", "opencode"] {
            let dir = root.join("hooks").join(app);
            std::fs::create_dir_all(&dir).map_err(|e| AppError::io(&dir, e))?;
        }
        Ok(())
    }

    pub fn is_symlink(path: &Path) -> bool {
        path.symlink_metadata()
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false)
    }

    #[cfg(unix)]
    pub fn create_symlink(src: &Path, dest: &Path) -> Result<(), AppError> {
        std::os::unix::fs::symlink(src, dest).map_err(|e| AppError::io(dest, e))
    }

    #[cfg(windows)]
    pub fn create_symlink(src: &Path, dest: &Path) -> Result<(), AppError> {
        if src.is_dir() {
            std::os::windows::fs::symlink_dir(src, dest).map_err(|e| AppError::io(dest, e))
        } else {
            std::os::windows::fs::symlink_file(src, dest).map_err(|e| AppError::io(dest, e))
        }
    }

    pub fn remove_existing_path(path: &Path) -> Result<(), AppError> {
        if !path.exists() && !Self::is_symlink(path) {
            return Ok(());
        }
        if Self::is_symlink(path) {
            #[cfg(unix)]
            std::fs::remove_file(path).map_err(|e| AppError::io(path, e))?;
            #[cfg(windows)]
            {
                if path.is_dir() {
                    std::fs::remove_dir(path).map_err(|e| AppError::io(path, e))?;
                } else {
                    std::fs::remove_file(path).map_err(|e| AppError::io(path, e))?;
                }
            }
            return Ok(());
        }
        if path.is_dir() {
            std::fs::remove_dir_all(path).map_err(|e| AppError::io(path, e))?;
        } else {
            std::fs::remove_file(path).map_err(|e| AppError::io(path, e))?;
        }
        Ok(())
    }

    pub fn sync_all() -> Result<(), AppError> {
        Self::ensure_workspace_layout()?;
        Self::sync_mcp_from_workspace()?;
        Self::sync_skills_from_workspace()?;
        Self::sync_hooks_from_workspace()?;
        Ok(())
    }

    pub fn sync_mcp_from_workspace() -> Result<(), AppError> {
        let path = Self::mcp_file_path();
        if !path.exists() {
            return Ok(());
        }

        let content = std::fs::read_to_string(&path).map_err(|e| AppError::io(&path, e))?;
        let workspace: WorkspaceMcpFile =
            serde_json::from_str(&content).map_err(|e| AppError::json(&path, e))?;

        let mut config = MultiAppConfig::default();
        let mut opencode_enabled: HashMap<String, Value> = HashMap::new();

        for entry in workspace.servers {
            let enabled = entry.enabled.unwrap_or(true);
            if !enabled {
                continue;
            }

            let apps = entry.apps.unwrap_or_default();
            let name = entry.name.unwrap_or_else(|| entry.id.clone());
            let base_entry = json!({
                "enabled": true,
                "name": name,
                "server": entry.server,
            });

            if apps.claude {
                config
                    .mcp
                    .claude
                    .servers
                    .insert(entry.id.clone(), base_entry.clone());
            }
            if apps.codex {
                config
                    .mcp
                    .codex
                    .servers
                    .insert(entry.id.clone(), base_entry.clone());
            }
            if apps.gemini {
                config
                    .mcp
                    .gemini
                    .servers
                    .insert(entry.id.clone(), base_entry.clone());
            }
            if apps.opencode {
                if let Some(spec) = base_entry.get("server") {
                    opencode_enabled.insert(entry.id.clone(), spec.clone());
                }
            }
        }

        mcp::sync_enabled_to_claude(&config)?;
        mcp::sync_enabled_to_codex(&config)?;
        mcp::sync_enabled_to_gemini(&config)?;

        let desired_ids: HashSet<String> = opencode_enabled.keys().cloned().collect();

        for (id, spec) in opencode_enabled {
            mcp::sync_single_server_to_opencode(&config, &id, &spec)?;
        }

        let current = crate::opencode_config::get_mcp_servers()?;
        for id in current.keys() {
            if !desired_ids.contains(id) {
                mcp::remove_server_from_opencode(id)?;
            }
        }

        Ok(())
    }

    pub fn sync_skills_from_workspace() -> Result<(), AppError> {
        let source_root = Self::skills_dir();
        if !source_root.exists() {
            return Ok(());
        }

        // 读取配置文件获取应用选择
        let config = Self::read_skills_config()?;
        let config_map: HashMap<String, McpApps> = config
            .skills
            .into_iter()
            .map(|s| (s.name.clone(), s.apps))
            .collect();

        let entries = std::fs::read_dir(&source_root).map_err(|e| AppError::io(&source_root, e))?;

        for entry in entries {
            let entry = entry.map_err(|e| AppError::io(&source_root, e))?;
            let src = entry.path();
            if !src.is_dir() {
                continue;
            }

            let name = match src.file_name().and_then(|n| n.to_str()) {
                Some(v) if !v.trim().is_empty() && !v.starts_with('.') => v.to_string(),
                _ => continue,
            };

            // 获取该 skill 的应用选择，默认全部启用
            let apps = config_map.get(&name).cloned().unwrap_or_else(|| McpApps {
                claude: true,
                codex: true,
                gemini: true,
                opencode: true,
            });

            // 根据配置为每个应用创建或删除 symlink
            for app in AppType::all() {
                let app_dir = SkillService::get_app_skills_dir(&app).map_err(|e| {
                    AppError::Config(format!(
                        "failed to resolve app skills dir for {}: {e}",
                        app.as_str()
                    ))
                })?;
                std::fs::create_dir_all(&app_dir).map_err(|e| AppError::io(&app_dir, e))?;
                let dest = app_dir.join(&name);

                // 检查该应用是否启用
                let enabled = match app.as_str() {
                    "claude" => apps.claude,
                    "codex" => apps.codex,
                    "gemini" => apps.gemini,
                    "opencode" => apps.opencode,
                    _ => false,
                };

                if enabled {
                    // 启用：创建 symlink
                    Self::remove_existing_path(&dest)?;
                    Self::create_symlink(&src, &dest)?;
                } else {
                    // 禁用：删除 symlink（如果存在）
                    Self::remove_existing_path(&dest)?;
                }
            }
        }

        Ok(())
    }

    pub fn sync_hooks_from_workspace() -> Result<(), AppError> {
        Self::sync_claude_hooks()?;
        Self::sync_opencode_hooks()?;
        Ok(())
    }

    fn sync_claude_hooks() -> Result<(), AppError> {
        let src = Self::hooks_app_dir("claude");
        if !src.exists() {
            return Ok(());
        }

        let config = src.join("hooks.json");
        let target = crate::config::get_claude_settings_path();

        if config.exists() && target.exists() {
            Self::merge_hooks_config(&config, &target)?;
        }
        Ok(())
    }

    fn sync_opencode_hooks() -> Result<(), AppError> {
        let src = Self::hooks_app_dir("opencode");
        if !src.exists() {
            return Ok(());
        }

        let dest = crate::opencode_config::get_opencode_dir().join("plugin");
        std::fs::create_dir_all(&dest).map_err(|e| AppError::io(&dest, e))?;

        for entry in std::fs::read_dir(&src).map_err(|e| AppError::io(&src, e))? {
            let entry = entry.map_err(|e| AppError::io(&src, e))?;
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) != Some("ts") {
                continue;
            }

            if let Some(name) = path.file_name() {
                let target = dest.join(name);
                Self::remove_existing_path(&target)?;
                Self::create_symlink(&path, &target)?;
            }
        }
        Ok(())
    }

    fn merge_hooks_config(source: &Path, target: &Path) -> Result<(), AppError> {
        let workspace: Value = serde_json::from_str(
            &std::fs::read_to_string(source).map_err(|e| AppError::io(source, e))?,
        )
        .map_err(|e| AppError::json(source, e))?;

        let mut settings: Value = serde_json::from_str(
            &std::fs::read_to_string(target).map_err(|e| AppError::io(target, e))?,
        )
        .map_err(|e| AppError::json(target, e))?;

        let hooks = settings
            .get_mut("hooks")
            .and_then(|v| v.as_object_mut())
            .ok_or_else(|| AppError::Config("missing hooks field".into()))?;

        if let Some(obj) = workspace.as_object() {
            for (key, entries) in obj {
                if let Some(arr) = entries.as_array() {
                    let existing = hooks
                        .entry(key.clone())
                        .or_insert_with(|| json!([]))
                        .as_array_mut()
                        .ok_or_else(|| AppError::Config(format!("hooks.{} not array", key)))?;

                    for entry in arr {
                        let matcher = entry.get("matcher").and_then(|m| m.as_str());
                        let duplicate = matcher.map_or(false, |m| {
                            existing
                                .iter()
                                .any(|e| e.get("matcher").and_then(|v| v.as_str()) == Some(m))
                        });

                        if !duplicate {
                            existing.push(entry.clone());
                        }
                    }
                }
            }
        }

        let tmp = target.with_extension("tmp");
        std::fs::write(
            &tmp,
            serde_json::to_string_pretty(&settings)
                .map_err(|e| AppError::Config(format!("serialize failed: {}", e)))?,
        )
        .map_err(|e| AppError::io(&tmp, e))?;
        std::fs::rename(&tmp, target).map_err(|e| AppError::io(target, e))?;

        Ok(())
    }

    pub fn get_mcp_config() -> Result<WorkspaceMcpFile, AppError> {
        let path = Self::mcp_file_path();
        if !path.exists() {
            return Ok(WorkspaceMcpFile {
                version: Some(1),
                servers: vec![],
            });
        }

        let content = std::fs::read_to_string(&path).map_err(|e| AppError::io(&path, e))?;
        let config: WorkspaceMcpFile =
            serde_json::from_str(&content).map_err(|e| AppError::json(&path, e))?;
        Ok(config)
    }

    fn write_mcp_config(config: &WorkspaceMcpFile) -> Result<(), AppError> {
        let path = Self::mcp_file_path();
        let dir = path
            .parent()
            .ok_or_else(|| AppError::Config("Failed to get mcp directory".to_string()))?;

        std::fs::create_dir_all(dir).map_err(|e| AppError::io(dir, e))?;

        let tmp_path = path.with_extension("tmp");
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| AppError::Config(format!("Failed to serialize mcp config: {}", e)))?;

        std::fs::write(&tmp_path, content).map_err(|e| AppError::io(&tmp_path, e))?;
        std::fs::rename(&tmp_path, &path).map_err(|e| AppError::io(&path, e))?;

        Ok(())
    }

    pub fn upsert_mcp_server(
        id: String,
        name: Option<String>,
        enabled: Option<bool>,
        apps: Option<McpApps>,
        server: Value,
    ) -> Result<(), AppError> {
        let mut config = Self::get_mcp_config()?;

        let entry = WorkspaceMcpEntry {
            id: id.clone(),
            name,
            enabled,
            apps,
            server,
        };

        if let Some(pos) = config.servers.iter().position(|s| s.id == id) {
            config.servers[pos] = entry;
        } else {
            config.servers.push(entry);
        }

        Self::write_mcp_config(&config)?;

        Self::sync_mcp_from_workspace()?;

        Ok(())
    }

    pub fn delete_mcp_server(id: &str) -> Result<(), AppError> {
        let mut config = Self::get_mcp_config()?;

        config.servers.retain(|s| s.id != id);

        Self::write_mcp_config(&config)?;

        Self::sync_mcp_from_workspace()?;

        Ok(())
    }

    pub fn reorder_mcp_servers(server_ids: Vec<String>) -> Result<(), AppError> {
        let mut config = Self::get_mcp_config()?;

        let mut server_map: HashMap<String, WorkspaceMcpEntry> = config
            .servers
            .into_iter()
            .map(|s| (s.id.clone(), s))
            .collect();

        let mut new_servers = Vec::new();
        for id in server_ids {
            if let Some(server) = server_map.remove(&id) {
                new_servers.push(server);
            }
        }

        for (_, server) in server_map {
            new_servers.push(server);
        }

        config.servers = new_servers;
        Self::write_mcp_config(&config)?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSkill {
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub apps: McpApps,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSkillsFile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<u32>,
    pub skills: Vec<WorkspaceSkill>,
}

impl WorkspaceService {
    pub fn skills_file_path() -> PathBuf {
        Self::workspace_root().join("skills.json")
    }

    fn read_skills_config() -> Result<WorkspaceSkillsFile, AppError> {
        let path = Self::skills_file_path();
        if !path.exists() {
            return Ok(WorkspaceSkillsFile {
                version: Some(1),
                skills: vec![],
            });
        }

        let content = std::fs::read_to_string(&path).map_err(|e| AppError::io(&path, e))?;
        let config: WorkspaceSkillsFile =
            serde_json::from_str(&content).map_err(|e| AppError::json(&path, e))?;
        Ok(config)
    }

    fn write_skills_config(config: &WorkspaceSkillsFile) -> Result<(), AppError> {
        let path = Self::skills_file_path();
        let dir = path
            .parent()
            .ok_or_else(|| AppError::Config("Failed to get workspace directory".to_string()))?;

        std::fs::create_dir_all(dir).map_err(|e| AppError::io(dir, e))?;

        let tmp_path = path.with_extension("tmp");
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| AppError::Config(format!("Failed to serialize skills config: {}", e)))?;

        std::fs::write(&tmp_path, content).map_err(|e| AppError::io(&tmp_path, e))?;
        std::fs::rename(&tmp_path, &path).map_err(|e| AppError::io(&path, e))?;

        Ok(())
    }

    pub fn get_skills() -> Result<Vec<WorkspaceSkill>, AppError> {
        let skills_dir = Self::skills_dir();
        if !skills_dir.exists() {
            return Ok(vec![]);
        }

        let config = Self::read_skills_config()?;
        let mut config_map: HashMap<String, McpApps> = config
            .skills
            .into_iter()
            .map(|s| (s.name.clone(), s.apps))
            .collect();

        let entries = std::fs::read_dir(&skills_dir).map_err(|e| AppError::io(&skills_dir, e))?;
        let mut skills = Vec::new();

        for entry in entries {
            let entry = entry.map_err(|e| AppError::io(&skills_dir, e))?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let name = match path.file_name().and_then(|n| n.to_str()) {
                Some(v) if !v.trim().is_empty() && !v.starts_with('.') => v.to_string(),
                _ => continue,
            };

            let apps = config_map.remove(&name).unwrap_or_else(|| McpApps {
                claude: true,
                codex: true,
                gemini: true,
                opencode: true,
            });

            skills.push(WorkspaceSkill {
                name,
                path: path.to_string_lossy().to_string(),
                apps,
            });
        }

        skills.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(skills)
    }

    pub fn update_skill_apps(name: String, apps: McpApps) -> Result<(), AppError> {
        let mut config = Self::read_skills_config()?;

        if let Some(skill) = config.skills.iter_mut().find(|s| s.name == name) {
            skill.apps = apps;
        } else {
            let skills_dir = Self::skills_dir();
            let path = skills_dir.join(&name);

            config.skills.push(WorkspaceSkill {
                name,
                path: path.to_string_lossy().to_string(),
                apps,
            });
        }

        Self::write_skills_config(&config)?;

        Self::sync_skills_from_workspace()?;

        Ok(())
    }

    pub fn get_hooks() -> Result<Vec<WorkspaceHookApp>, AppError> {
        let hooks_root = Self::hooks_root();
        if !hooks_root.exists() {
            return Ok(vec![]);
        }

        let mut result = Vec::new();

        for app in ["claude", "codex", "gemini", "opencode"] {
            let app_dir = Self::hooks_app_dir(app);
            if !app_dir.exists() {
                continue;
            }

            let mut files = Vec::new();
            Self::scan_hooks_dir(&app_dir, &app_dir, &mut files)?;

            let hooks_config = Self::read_hooks_config(app)?;

            if !files.is_empty() || hooks_config.is_some() {
                result.push(WorkspaceHookApp {
                    app: app.to_string(),
                    files,
                    config: hooks_config,
                });
            }
        }

        Ok(result)
    }

    fn read_hooks_config(app: &str) -> Result<Option<Value>, AppError> {
        let hooks_json_path = Self::hooks_app_dir(app).join("hooks.json");
        if !hooks_json_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&hooks_json_path)
            .map_err(|e| AppError::io(&hooks_json_path, e))?;
        let config: Value =
            serde_json::from_str(&content).map_err(|e| AppError::json(&hooks_json_path, e))?;
        Ok(Some(config))
    }

    fn scan_hooks_dir(
        base_dir: &Path,
        current_dir: &Path,
        files: &mut Vec<WorkspaceHookFile>,
    ) -> Result<(), AppError> {
        let entries = std::fs::read_dir(current_dir).map_err(|e| AppError::io(current_dir, e))?;

        for entry in entries {
            let entry = entry.map_err(|e| AppError::io(current_dir, e))?;
            let path = entry.path();

            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.') {
                    continue;
                }
                if name == "hooks.json" {
                    continue;
                }
            }

            if path.is_dir() {
                Self::scan_hooks_dir(base_dir, &path, files)?;
            } else if path.is_file() {
                let relative_path = path
                    .strip_prefix(base_dir)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .to_string();

                let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

                files.push(WorkspaceHookFile {
                    name: path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string(),
                    path: relative_path,
                    full_path: path.to_string_lossy().to_string(),
                    size,
                });
            }
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceHookApp {
    pub app: String,
    pub files: Vec<WorkspaceHookFile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceHookFile {
    pub name: String,
    pub path: String,
    pub full_path: String,
    pub size: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceMcpFile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<u32>,
    pub servers: Vec<WorkspaceMcpEntry>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceMcpEntry {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub apps: Option<McpApps>,
    pub server: Value,
}
