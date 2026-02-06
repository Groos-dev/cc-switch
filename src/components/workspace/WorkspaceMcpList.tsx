import React from "react";
import { useTranslation } from "react-i18next";
import { Edit, Trash2, GripVertical } from "lucide-react";
import { Button } from "@/components/ui/button";
import { AppToggleGroup } from "@/components/common/AppToggleGroup";
import { useDeleteWorkspaceMcpServer } from "@/hooks/useWorkspace";
import type { WorkspaceMcpServer } from "@/lib/api/workspace";
import type { AppId } from "@/lib/api/types";
import { toast } from "sonner";

interface WorkspaceMcpListProps {
  servers: WorkspaceMcpServer[];
  isLoading: boolean;
  onEdit: (server: WorkspaceMcpServer) => void;
}

export const WorkspaceMcpList: React.FC<WorkspaceMcpListProps> = ({
  servers,
  isLoading,
  onEdit,
}) => {
  const { t } = useTranslation();
  const deleteMutation = useDeleteWorkspaceMcpServer();

  const handleDelete = async (server: WorkspaceMcpServer) => {
    if (
      !confirm(
        t("workspace.mcp.confirmDelete", { name: server.name || server.id }),
      )
    ) {
      return;
    }

    try {
      await deleteMutation.mutateAsync(server.id);
      toast.success(
        t("workspace.mcp.deleteSuccess", { defaultValue: "删除成功" }),
      );
    } catch (error) {
      toast.error(
        t("workspace.mcp.deleteFailed", { defaultValue: "删除失败" }),
      );
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-muted-foreground">
          {t("common.loading", { defaultValue: "加载中..." })}
        </div>
      </div>
    );
  }

  if (servers.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-64 glass rounded-xl border border-white/10">
        <p className="text-muted-foreground mb-4">
          {t("workspace.mcp.empty", { defaultValue: "暂无 MCP 服务器配置" })}
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-3">
      {servers.map((server) => (
        <div
          key={server.id}
          className="glass rounded-xl p-4 border border-white/10 hover:border-white/20 transition-colors"
        >
          <div className="flex items-start gap-4">
            {/* Drag Handle */}
            <div className="flex-shrink-0 pt-1 cursor-grab active:cursor-grabbing opacity-50 hover:opacity-100">
              <GripVertical className="h-5 w-5" />
            </div>

            {/* Content */}
            <div className="flex-1 min-w-0">
              <div className="flex items-start justify-between gap-4 mb-3">
                <div className="flex-1 min-w-0">
                  <h3 className="font-semibold text-foreground truncate">
                    {server.name || server.id}
                  </h3>
                  {server.name && server.name !== server.id && (
                    <p className="text-sm text-muted-foreground truncate">
                      ID: {server.id}
                    </p>
                  )}
                </div>

                {/* Actions */}
                <div className="flex items-center gap-2 flex-shrink-0">
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => onEdit(server)}
                    className="h-8 w-8 p-0"
                  >
                    <Edit className="h-4 w-4" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => handleDelete(server)}
                    className="h-8 w-8 p-0 text-red-500 hover:text-red-600 hover:bg-red-500/10"
                  >
                    <Trash2 className="h-4 w-4" />
                  </Button>
                </div>
              </div>

              {/* Apps Toggle */}
              <div className="flex items-center gap-3">
                <span className="text-sm text-muted-foreground">
                  {t("workspace.mcp.enabledApps", {
                    defaultValue: "启用应用:",
                  })}
                </span>
                <AppToggleGroup
                  apps={{
                    claude: server.apps?.claude ?? false,
                    codex: server.apps?.codex ?? false,
                    gemini: server.apps?.gemini ?? false,
                    opencode: server.apps?.opencode ?? false,
                  }}
                  onToggle={(app: AppId, enabled: boolean) => {
                    onEdit({
                      ...server,
                      apps: {
                        claude: server.apps?.claude ?? false,
                        codex: server.apps?.codex ?? false,
                        gemini: server.apps?.gemini ?? false,
                        opencode: server.apps?.opencode ?? false,
                        [app]: enabled,
                      },
                    });
                  }}
                />
              </div>

              {/* Server Config Preview */}
              <div className="mt-3 p-3 bg-black/20 dark:bg-white/5 rounded-lg">
                <pre className="text-xs text-muted-foreground overflow-x-auto">
                  {JSON.stringify(server.server, null, 2)}
                </pre>
              </div>
            </div>
          </div>
        </div>
      ))}
    </div>
  );
};
