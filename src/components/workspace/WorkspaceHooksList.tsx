import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import {
  FileCode,
  Folder,
  Settings,
  ChevronDown,
  ChevronRight,
} from "lucide-react";
import type { WorkspaceHookApp, WorkspaceHookFile } from "@/lib/api/workspace";
import { workspaceApi } from "@/lib/api/workspace";

interface WorkspaceHooksListProps {
  hooks: WorkspaceHookApp[];
  isLoading: boolean;
}

const APP_NAMES: Record<string, string> = {
  claude: "Claude Code",
  codex: "Codex",
  gemini: "Gemini Code Assist",
  opencode: "OpenCode",
};

const formatBytes = (bytes: number): string => {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
};

export const WorkspaceHooksList: React.FC<WorkspaceHooksListProps> = ({
  hooks,
  isLoading,
}) => {
  const { t } = useTranslation();
  const [expanded, setExpanded] = useState({
    configs: new Set<string>(),
    files: new Set<string>(),
  });
  const [contents, setContents] = useState(new Map<string, string>());

  const toggle = {
    config: (app: string) => {
      setExpanded((prev) => {
        const configs = new Set(prev.configs);
        configs.has(app) ? configs.delete(app) : configs.add(app);
        return { ...prev, configs };
      });
    },

    file: async (file: WorkspaceHookFile) => {
      const key = file.fullPath;
      const isExpanded = expanded.files.has(key);

      setExpanded((prev) => {
        const files = new Set(prev.files);
        isExpanded ? files.delete(key) : files.add(key);
        return { ...prev, files };
      });

      if (!isExpanded && !contents.has(key)) {
        try {
          const content = await workspaceApi.readHookFile(key);
          setContents((prev) => new Map(prev).set(key, content));
        } catch (err) {
          const msg = err instanceof Error ? err.message : String(err);
          setContents((prev) =>
            new Map(prev).set(key, `Failed to load: ${msg}`),
          );
        }
      }
    },
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
      </div>
    );
  }

  if (hooks.length === 0) {
    return (
      <div className="glass rounded-xl p-8 border border-white/10 text-center">
        <FileCode className="h-12 w-12 mx-auto mb-4 text-muted-foreground" />
        <p className="text-muted-foreground">
          {t("workspace.hooks.empty", {
            defaultValue: "暂无 Hooks 文件",
          })}
        </p>
        <p className="text-sm text-muted-foreground mt-2">
          {t("workspace.hooks.emptyHint", {
            defaultValue:
              "将 Hook 文件放入 ~/.cc-switch/data/hooks/<app>/ 目录",
          })}
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {hooks.map((hookApp) => (
        <div
          key={hookApp.app}
          className="glass rounded-xl border border-white/10 overflow-hidden"
        >
          <div className="bg-primary/5 px-4 py-3 border-b border-white/10">
            <div className="flex items-center gap-2">
              <Folder className="h-5 w-5 text-primary" />
              <h3 className="font-semibold">
                {APP_NAMES[hookApp.app] || hookApp.app}
              </h3>
              <span className="text-sm text-muted-foreground ml-auto">
                {hookApp.files.length}{" "}
                {t("workspace.hooks.filesCount", { defaultValue: "个文件" })}
              </span>
            </div>
          </div>

          {hookApp.config && (
            <div className="border-b border-white/10">
              <button
                type="button"
                className="w-full flex items-center justify-between px-4 py-3 hover:bg-white/5 transition-colors"
                onClick={() => toggle.config(hookApp.app)}
              >
                <div className="flex items-center gap-2">
                  <Settings className="h-4 w-4 text-muted-foreground" />
                  <span className="text-sm font-medium">
                    {t("workspace.hooks.config", {
                      defaultValue: "Hooks 配置",
                    })}
                  </span>
                </div>
                {expanded.configs.has(hookApp.app) ? (
                  <ChevronDown className="h-4 w-4 text-muted-foreground" />
                ) : (
                  <ChevronRight className="h-4 w-4 text-muted-foreground" />
                )}
              </button>
              {expanded.configs.has(hookApp.app) && (
                <div className="px-4 pb-3">
                  <pre className="text-xs bg-black/20 rounded p-3 overflow-x-auto">
                    {JSON.stringify(hookApp.config, null, 2)}
                  </pre>
                </div>
              )}
            </div>
          )}

          <div className="divide-y divide-white/5">
            {hookApp.files.map((file, index) => (
              <div key={`${file.path}-${index}`}>
                <button
                  type="button"
                  className="w-full px-4 py-3 hover:bg-white/5 transition-colors"
                  onClick={() => toggle.file(file)}
                >
                  <div className="flex items-start justify-between gap-4">
                    <div className="flex items-start gap-3 flex-1 min-w-0">
                      <FileCode className="h-5 w-5 text-muted-foreground flex-shrink-0 mt-0.5" />
                      <div className="flex-1 min-w-0">
                        <div className="font-medium truncate text-left">
                          {file.name}
                        </div>
                        <div className="text-sm text-muted-foreground truncate mt-1 text-left">
                          {file.path}
                        </div>
                      </div>
                    </div>
                    <div className="flex items-center gap-2 flex-shrink-0">
                      <span className="text-sm text-muted-foreground">
                        {formatBytes(file.size)}
                      </span>
                      {expanded.files.has(file.fullPath) ? (
                        <ChevronDown className="h-4 w-4 text-muted-foreground" />
                      ) : (
                        <ChevronRight className="h-4 w-4 text-muted-foreground" />
                      )}
                    </div>
                  </div>
                </button>
                {expanded.files.has(file.fullPath) && (
                  <div className="px-4 pb-3 bg-black/10">
                    <pre className="text-xs bg-black/20 rounded p-3 overflow-x-auto max-h-96 overflow-y-auto">
                      {contents.get(file.fullPath) || "加载中..."}
                    </pre>
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
};
