import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { workspaceApi, type WorkspaceMcpApps } from "@/lib/api/workspace";
import { toast } from "sonner";

// Query key factory
export const workspaceKeys = {
  all: ["workspace"] as const,
  mcpConfig: () => [...workspaceKeys.all, "mcp-config"] as const,
  skills: () => [...workspaceKeys.all, "skills"] as const,
  hooks: () => [...workspaceKeys.all, "hooks"] as const,
};

// 获取 MCP 配置
export function useWorkspaceMcpConfig() {
  return useQuery({
    queryKey: workspaceKeys.mcpConfig(),
    queryFn: () => workspaceApi.getMcpConfig(),
    staleTime: 30000, // 30 seconds
  });
}

// 更新/添加 MCP 服务器
export function useUpsertWorkspaceMcpServer() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (params: {
      id: string;
      name?: string;
      enabled?: boolean;
      apps?: WorkspaceMcpApps;
      server: Record<string, any>;
    }) => {
      return workspaceApi.upsertMcpServer(
        params.id,
        params.name,
        params.enabled,
        params.apps,
        params.server,
      );
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: workspaceKeys.mcpConfig() });
    },
  });
}

// 删除 MCP 服务器
export function useDeleteWorkspaceMcpServer() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => workspaceApi.deleteMcpServer(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: workspaceKeys.mcpConfig() });
    },
  });
}

// 重排序 MCP 服务器
export function useReorderWorkspaceMcpServers() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (serverIds: string[]) =>
      workspaceApi.reorderMcpServers(serverIds),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: workspaceKeys.mcpConfig() });
    },
  });
}

// 同步所有
export function useSyncWorkspace() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async () => {
      await workspaceApi.ensureLayout();
      await workspaceApi.syncAll();
    },
    onSuccess: () => {
      // 刷新相关数据
      queryClient.invalidateQueries({ queryKey: ["skills"] });
      toast.success("工作区同步成功", { closeButton: true });
    },
    onError: (error: Error) => {
      toast.error(error.message || "工作区同步失败", { closeButton: true });
    },
  });
}

// 获取 Skills 列表
export function useWorkspaceSkills() {
  return useQuery({
    queryKey: workspaceKeys.skills(),
    queryFn: () => workspaceApi.getSkills(),
    staleTime: 30000, // 30 seconds
  });
}

// 更新 Skill 的应用选择
export function useUpdateSkillApps() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (params: { name: string; apps: WorkspaceMcpApps }) => {
      return workspaceApi.updateSkillApps(params.name, params.apps);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: workspaceKeys.skills() });
    },
  });
}

// 获取 Hooks 列表
export function useWorkspaceHooks() {
  return useQuery({
    queryKey: workspaceKeys.hooks(),
    queryFn: () => workspaceApi.getHooks(),
    staleTime: 30000, // 30 seconds
  });
}
