import React from "react";
import { useTranslation } from "react-i18next";
import { FolderOpen } from "lucide-react";
import { AppToggleGroup } from "@/components/common/AppToggleGroup";
import { useUpdateSkillApps } from "@/hooks/useWorkspace";
import type { WorkspaceSkill } from "@/lib/api/workspace";
import type { AppId } from "@/lib/api/types";
import { toast } from "sonner";

const WORKSPACE_APP_IDS: AppId[] = ["claude", "codex", "gemini", "opencode"];

interface WorkspaceSkillsListProps {
  skills: WorkspaceSkill[];
  isLoading: boolean;
}

export const WorkspaceSkillsList: React.FC<WorkspaceSkillsListProps> = ({
  skills,
  isLoading,
}) => {
  const { t } = useTranslation();
  const updateAppsMutation = useUpdateSkillApps();

  const handleToggleApp = async (
    skill: WorkspaceSkill,
    app: AppId,
    enabled: boolean,
  ) => {
    try {
      await updateAppsMutation.mutateAsync({
        name: skill.name,
        apps: {
          claude: skill.apps.claude,
          codex: skill.apps.codex,
          gemini: skill.apps.gemini,
          opencode: skill.apps.opencode,
          [app]: enabled,
        },
      });
      toast.success(
        t("workspace.skills.updateSuccess", { defaultValue: "更新成功" }),
      );
    } catch (error) {
      toast.error(
        t("workspace.skills.updateFailed", { defaultValue: "更新失败" }),
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

  if (skills.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-64 glass rounded-xl border border-white/10">
        <p className="text-muted-foreground mb-4">
          {t("workspace.skills.empty", { defaultValue: "暂无 Skills" })}
        </p>
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
      {skills.map((skill) => (
        <div
          key={skill.name}
          className="glass rounded-xl p-4 border border-white/10 hover:border-white/20 transition-colors"
        >
          <div className="flex items-start gap-3 mb-3">
            <div className="flex-shrink-0 w-10 h-10 rounded-lg bg-primary/10 flex items-center justify-center">
              <FolderOpen className="h-5 w-5 text-primary" />
            </div>
            <div className="flex-1 min-w-0">
              <h3 className="font-semibold text-foreground truncate">
                {skill.name}
              </h3>
              <p className="text-xs text-muted-foreground mt-1 truncate">
                {skill.path}
              </p>
            </div>
          </div>

          {/* Apps Toggle */}
          <div className="flex items-center gap-2 pt-3 border-t border-white/10">
            <span className="text-xs text-muted-foreground">
              {t("workspace.skills.enabledApps", { defaultValue: "启用应用:" })}
            </span>
            <AppToggleGroup
              apps={{
                claude: skill.apps.claude,
                codex: skill.apps.codex,
                gemini: skill.apps.gemini,
                opencode: skill.apps.opencode,
                openclaw: false,
              }}
              appIds={WORKSPACE_APP_IDS}
              onToggle={(app: AppId, enabled: boolean) => {
                handleToggleApp(skill, app, enabled);
              }}
            />
          </div>
        </div>
      ))}
    </div>
  );
};
