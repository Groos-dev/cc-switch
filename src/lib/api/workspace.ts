import { invoke } from "@tauri-apps/api/core";

export interface WorkspaceMcpApps {
  claude: boolean;
  codex: boolean;
  gemini: boolean;
  opencode: boolean;
}

export interface WorkspaceMcpServer {
  id: string;
  name?: string;
  enabled?: boolean;
  apps?: WorkspaceMcpApps;
  server: Record<string, any>;
}

export interface WorkspaceMcpConfig {
  version?: number;
  servers: WorkspaceMcpServer[];
}

export interface WorkspaceSkill {
  name: string;
  path: string;
  apps: WorkspaceMcpApps;
}

export interface WorkspaceHookFile {
  name: string;
  path: string;
  fullPath: string;
  size: number;
}

export interface WorkspaceHookApp {
  app: string;
  files: WorkspaceHookFile[];
  config?: Record<string, unknown>;
}

export const workspaceApi = {
  async ensureLayout(): Promise<boolean> {
    return await invoke("workspace_ensure_layout");
  },

  async syncAll(): Promise<boolean> {
    return await invoke("workspace_sync_all");
  },

  async syncMcp(): Promise<boolean> {
    return await invoke("workspace_sync_mcp");
  },

  async syncSkills(): Promise<boolean> {
    return await invoke("workspace_sync_skills");
  },

  async syncHooks(): Promise<boolean> {
    return await invoke("workspace_sync_hooks");
  },

  async getMcpConfig(): Promise<WorkspaceMcpConfig> {
    return await invoke("workspace_get_mcp_config");
  },

  async upsertMcpServer(
    id: string,
    name: string | undefined,
    enabled: boolean | undefined,
    apps: WorkspaceMcpApps | undefined,
    server: Record<string, any>,
  ): Promise<boolean> {
    return await invoke("workspace_upsert_mcp_server", {
      id,
      name,
      enabled,
      apps,
      server,
    });
  },

  async deleteMcpServer(id: string): Promise<boolean> {
    return await invoke("workspace_delete_mcp_server", { id });
  },

  async reorderMcpServers(serverIds: string[]): Promise<boolean> {
    return await invoke("workspace_reorder_mcp_servers", { serverIds });
  },

  async getSkills(): Promise<WorkspaceSkill[]> {
    return await invoke("workspace_get_skills");
  },

  async updateSkillApps(
    name: string,
    apps: WorkspaceMcpApps,
  ): Promise<boolean> {
    return await invoke("workspace_update_skill_apps", { name, apps });
  },

  async getHooks(): Promise<WorkspaceHookApp[]> {
    return await invoke("workspace_get_hooks");
  },

  async readHookFile(filePath: string): Promise<string> {
    return await invoke("workspace_read_hook_file", { filePath });
  },

  async readFile(filename: string): Promise<string | null> {
    return invoke<string | null>("read_workspace_file", { filename });
  },

  async writeFile(filename: string, content: string): Promise<void> {
    return invoke<void>("write_workspace_file", { filename, content });
  },
};
