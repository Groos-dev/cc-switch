import React, { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Save, Plus, AlertCircle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import JsonEditor from "@/components/JsonEditor";
import { FullScreenPanel } from "@/components/common/FullScreenPanel";
import { useUpsertWorkspaceMcpServer } from "@/hooks/useWorkspace";
import type { WorkspaceMcpServer } from "@/lib/api/workspace";
import { toast } from "sonner";

interface WorkspaceMcpFormModalProps {
  server: WorkspaceMcpServer | null;
  existingIds: string[];
  onClose: () => void;
}

export const WorkspaceMcpFormModal: React.FC<WorkspaceMcpFormModalProps> = ({
  server,
  existingIds,
  onClose,
}) => {
  const { t } = useTranslation();
  const isEditing = !!server;

  const [formId, setFormId] = useState(server?.id || "");
  const [formName, setFormName] = useState(server?.name || "");
  const [formEnabled, setFormEnabled] = useState(server?.enabled ?? true);
  const [enabledApps, setEnabledApps] = useState({
    claude: server?.apps?.claude ?? true,
    codex: server?.apps?.codex ?? true,
    gemini: server?.apps?.gemini ?? true,
    opencode: server?.apps?.opencode ?? false,
  });
  const [formConfig, setFormConfig] = useState(
    server?.server ? JSON.stringify(server.server, null, 2) : "",
  );
  const [configError, setConfigError] = useState("");
  const [idError, setIdError] = useState("");
  const [isDarkMode, setIsDarkMode] = useState(false);

  const upsertMutation = useUpsertWorkspaceMcpServer();

  useEffect(() => {
    setIsDarkMode(document.documentElement.classList.contains("dark"));

    const observer = new MutationObserver(() => {
      setIsDarkMode(document.documentElement.classList.contains("dark"));
    });

    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["class"],
    });

    return () => observer.disconnect();
  }, []);

  const handleIdChange = (value: string) => {
    setFormId(value);
    if (!isEditing) {
      const exists = existingIds.includes(value.trim());
      setIdError(exists ? t("mcp.error.idExists") : "");
    }
  };

  const handleConfigChange = (value: string) => {
    setFormConfig(value);
    try {
      if (value.trim()) {
        JSON.parse(value);
      }
      setConfigError("");
    } catch (err: any) {
      setConfigError(t("mcp.error.jsonInvalid") + ": " + err.message);
    }
  };

  const handleSubmit = async () => {
    const trimmedId = formId.trim();
    if (!trimmedId) {
      toast.error(t("mcp.error.idRequired"), { duration: 3000 });
      return;
    }

    if (!isEditing && existingIds.includes(trimmedId)) {
      setIdError(t("mcp.error.idExists"));
      return;
    }

    let serverConfig: Record<string, any>;
    try {
      serverConfig = formConfig.trim() ? JSON.parse(formConfig) : {};
    } catch (err: any) {
      setConfigError(t("mcp.error.jsonInvalid") + ": " + err.message);
      toast.error(t("mcp.error.jsonInvalid"), { duration: 4000 });
      return;
    }

    try {
      await upsertMutation.mutateAsync({
        id: trimmedId,
        name: formName.trim() || undefined,
        enabled: formEnabled,
        apps: enabledApps,
        server: serverConfig,
      });

      toast.success(t("common.success"), { closeButton: true });
      onClose();
    } catch (error: any) {
      const msg = error?.message || t("mcp.error.saveFailed");
      toast.error(msg, { duration: 4000 });
    }
  };

  return (
    <FullScreenPanel
      isOpen={true}
      title={
        isEditing
          ? t("workspace.mcp.edit", { defaultValue: "编辑 MCP 服务器" })
          : t("workspace.mcp.add", { defaultValue: "添加 MCP 服务器" })
      }
      onClose={onClose}
      footer={
        <Button
          type="button"
          onClick={handleSubmit}
          disabled={upsertMutation.isPending || (!isEditing && !!idError)}
          className="bg-primary text-primary-foreground hover:bg-primary/90"
        >
          {isEditing ? <Save size={16} /> : <Plus size={16} />}
          {upsertMutation.isPending
            ? t("common.saving")
            : isEditing
              ? t("common.save")
              : t("common.add")}
        </Button>
      }
    >
      <div className="flex flex-col h-full gap-6">
        {/* 表单字段 */}
        <div className="glass rounded-xl p-6 border border-white/10 space-y-6 flex-shrink-0">
          {/* ID */}
          <div>
            <div className="flex items-center justify-between mb-2">
              <label className="block text-sm font-medium text-foreground">
                {t("mcp.form.title")} <span className="text-red-500">*</span>
              </label>
              {!isEditing && idError && (
                <span className="text-xs text-red-500 dark:text-red-400">
                  {idError}
                </span>
              )}
            </div>
            <Input
              type="text"
              placeholder={t("mcp.form.titlePlaceholder")}
              value={formId}
              onChange={(e) => handleIdChange(e.target.value)}
              disabled={isEditing}
            />
          </div>

          {/* Name */}
          <div>
            <label className="block text-sm font-medium text-foreground mb-2">
              {t("mcp.form.name")}
            </label>
            <Input
              type="text"
              placeholder={t("mcp.form.namePlaceholder")}
              value={formName}
              onChange={(e) => setFormName(e.target.value)}
            />
          </div>

          {/* Enabled */}
          <div className="flex items-center gap-2">
            <Checkbox
              id="enabled"
              checked={formEnabled}
              onCheckedChange={(checked: boolean) => setFormEnabled(checked)}
            />
            <label
              htmlFor="enabled"
              className="text-sm text-foreground cursor-pointer select-none"
            >
              {t("workspace.mcp.enabled", { defaultValue: "启用此服务器" })}
            </label>
          </div>

          {/* 启用到哪些应用 */}
          <div>
            <label className="block text-sm font-medium text-foreground mb-3">
              {t("mcp.form.enabledApps")}
            </label>
            <div className="flex flex-wrap gap-4">
              <div className="flex items-center gap-2">
                <Checkbox
                  id="enable-claude"
                  checked={enabledApps.claude}
                  onCheckedChange={(checked: boolean) =>
                    setEnabledApps({ ...enabledApps, claude: checked })
                  }
                />
                <label
                  htmlFor="enable-claude"
                  className="text-sm text-foreground cursor-pointer select-none"
                >
                  Claude
                </label>
              </div>

              <div className="flex items-center gap-2">
                <Checkbox
                  id="enable-codex"
                  checked={enabledApps.codex}
                  onCheckedChange={(checked: boolean) =>
                    setEnabledApps({ ...enabledApps, codex: checked })
                  }
                />
                <label
                  htmlFor="enable-codex"
                  className="text-sm text-foreground cursor-pointer select-none"
                >
                  Codex
                </label>
              </div>

              <div className="flex items-center gap-2">
                <Checkbox
                  id="enable-gemini"
                  checked={enabledApps.gemini}
                  onCheckedChange={(checked: boolean) =>
                    setEnabledApps({ ...enabledApps, gemini: checked })
                  }
                />
                <label
                  htmlFor="enable-gemini"
                  className="text-sm text-foreground cursor-pointer select-none"
                >
                  Gemini
                </label>
              </div>

              <div className="flex items-center gap-2">
                <Checkbox
                  id="enable-opencode"
                  checked={enabledApps.opencode}
                  onCheckedChange={(checked: boolean) =>
                    setEnabledApps({ ...enabledApps, opencode: checked })
                  }
                />
                <label
                  htmlFor="enable-opencode"
                  className="text-sm text-foreground cursor-pointer select-none"
                >
                  OpenCode
                </label>
              </div>
            </div>
          </div>
        </div>

        {/* JSON 配置编辑器 */}
        <div className="glass rounded-xl p-6 border border-white/10 flex flex-col flex-1 min-h-0">
          <label className="text-sm font-medium text-foreground mb-4">
            {t("mcp.form.jsonConfig")}
          </label>
          <div className="flex-1 min-h-0 flex flex-col">
            <div className="flex-1 min-h-0">
              <JsonEditor
                value={formConfig}
                onChange={handleConfigChange}
                placeholder={t("mcp.form.jsonPlaceholder")}
                darkMode={isDarkMode}
                rows={12}
                showValidation={true}
                language="json"
                height="100%"
              />
            </div>
            {configError && (
              <div className="flex items-center gap-2 mt-2 text-red-500 dark:text-red-400 text-sm flex-shrink-0">
                <AlertCircle size={16} />
                <span>{configError}</span>
              </div>
            )}
          </div>
        </div>
      </div>
    </FullScreenPanel>
  );
};
