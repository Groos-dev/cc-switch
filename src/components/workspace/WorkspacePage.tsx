import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { Plus, RefreshCw, Database } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  useWorkspaceMcpConfig,
  useSyncWorkspace,
  useWorkspaceSkills,
  useWorkspaceHooks,
} from "@/hooks/useWorkspace";
import { WorkspaceMcpList } from "./WorkspaceMcpList";
import { WorkspaceMcpFormModal } from "./WorkspaceMcpFormModal";
import { WorkspaceSkillsList } from "./WorkspaceSkillsList";
import { WorkspaceHooksList } from "./WorkspaceHooksList";
import type { WorkspaceMcpServer } from "@/lib/api/workspace";

export const WorkspacePage: React.FC = () => {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState("mcp");
  const [isFormOpen, setIsFormOpen] = useState(false);
  const [editingServer, setEditingServer] = useState<WorkspaceMcpServer | null>(
    null,
  );

  const { data: mcpConfig, isLoading } = useWorkspaceMcpConfig();
  const { data: skills, isLoading: isLoadingSkills } = useWorkspaceSkills();
  const { data: hooks, isLoading: isLoadingHooks } = useWorkspaceHooks();
  const syncMutation = useSyncWorkspace();

  const handleAddMcp = () => {
    setEditingServer(null);
    setIsFormOpen(true);
  };

  const handleEditMcp = (server: WorkspaceMcpServer) => {
    setEditingServer(server);
    setIsFormOpen(true);
  };

  const handleCloseForm = () => {
    setIsFormOpen(false);
    setEditingServer(null);
  };

  const handleSync = () => {
    syncMutation.mutate();
  };

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between px-6 py-4 border-b border-border/50">
        <div className="flex items-center gap-3">
          <Database className="h-6 w-6 text-primary" />
          <div>
            <h1 className="text-xl font-semibold">
              {t("workspace.title", { defaultValue: "工作区管理" })}
            </h1>
            <p className="text-sm text-muted-foreground">
              {t("workspace.subtitle", {
                defaultValue: "统一管理 Skills/MCP/Hooks 配置",
              })}
            </p>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={handleSync}
            disabled={syncMutation.isPending}
          >
            <RefreshCw
              className={`h-4 w-4 mr-2 ${syncMutation.isPending ? "animate-spin" : ""}`}
            />
            {t("workspace.syncAll", { defaultValue: "同步到工具" })}
          </Button>
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-hidden">
        <Tabs
          value={activeTab}
          onValueChange={setActiveTab}
          className="h-full flex flex-col"
        >
          <div className="px-6 pt-4">
            <TabsList className="grid w-full grid-cols-3">
              <TabsTrigger value="mcp">
                {t("workspace.tabs.mcp", { defaultValue: "MCP 服务器" })}
              </TabsTrigger>
              <TabsTrigger value="skills">
                {t("workspace.tabs.skills", { defaultValue: "Skills" })}
              </TabsTrigger>
              <TabsTrigger value="hooks">
                {t("workspace.tabs.hooks", { defaultValue: "Hooks" })}
              </TabsTrigger>
            </TabsList>
          </div>

          <div className="flex-1 overflow-hidden px-6 pb-6">
            <TabsContent value="mcp" className="h-full mt-4">
              <div className="h-full flex flex-col">
                <div className="flex items-center justify-between mb-4">
                  <p className="text-sm text-muted-foreground">
                    {t("workspace.mcp.description", {
                      defaultValue: "配置文件：~/.cc-switch/data/mcp/mcp.json",
                    })}
                  </p>
                  <Button size="sm" onClick={handleAddMcp}>
                    <Plus className="h-4 w-4 mr-2" />
                    {t("workspace.mcp.add", { defaultValue: "添加 MCP" })}
                  </Button>
                </div>

                <div className="flex-1 overflow-auto">
                  <WorkspaceMcpList
                    servers={mcpConfig?.servers || []}
                    isLoading={isLoading}
                    onEdit={handleEditMcp}
                  />
                </div>
              </div>
            </TabsContent>

            <TabsContent value="skills" className="h-full mt-4">
              <div className="h-full flex flex-col">
                <div className="flex items-center justify-between mb-4">
                  <p className="text-sm text-muted-foreground">
                    {t("workspace.skills.description", {
                      defaultValue:
                        "Skills 目录：~/.cc-switch/data/skills/\n通过 symlink 同步到各工具的 skills 目录",
                    })}
                  </p>
                </div>

                <div className="flex-1 overflow-auto">
                  <WorkspaceSkillsList
                    skills={skills || []}
                    isLoading={isLoadingSkills}
                  />
                </div>
              </div>
            </TabsContent>

            <TabsContent value="hooks" className="h-full mt-4">
              <div className="h-full flex flex-col">
                <div className="flex items-center justify-between mb-4">
                  <p className="text-sm text-muted-foreground">
                    {t("workspace.hooks.description", {
                      defaultValue:
                        "Hooks 目录：~/.cc-switch/data/hooks/<app>/\n通过 symlink 同步到各工具的配置目录",
                    })}
                  </p>
                </div>

                <div className="flex-1 overflow-auto">
                  <WorkspaceHooksList
                    hooks={hooks || []}
                    isLoading={isLoadingHooks}
                  />
                </div>
              </div>
            </TabsContent>
          </div>
        </Tabs>
      </div>

      {/* MCP Form Modal */}
      {isFormOpen && (
        <WorkspaceMcpFormModal
          server={editingServer}
          existingIds={mcpConfig?.servers.map((s) => s.id) || []}
          onClose={handleCloseForm}
        />
      )}
    </div>
  );
};
