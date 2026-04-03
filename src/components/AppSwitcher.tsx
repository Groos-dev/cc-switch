import type { AppId } from "@/lib/api";
import type { VisibleApps } from "@/types";
import { useTranslation } from "react-i18next";
import { ProviderIcon } from "@/components/ProviderIcon";
import { WorkspaceSyncIcon } from "@/components/BrandIcons";
import { cn } from "@/lib/utils";

interface AppSwitcherProps {
  activeApp: AppId;
  onSwitch: (app: AppId) => void;
  isWorkspaceActive?: boolean;
  onOpenWorkspace?: () => void;
  visibleApps?: VisibleApps;
  compact?: boolean;
}

const ALL_APPS: AppId[] = ["claude", "codex", "gemini", "opencode", "openclaw"];
const STORAGE_KEY = "cc-switch-last-app";

export function AppSwitcher({
  activeApp,
  onSwitch,
  isWorkspaceActive = false,
  onOpenWorkspace,
  visibleApps,
  compact,
}: AppSwitcherProps) {
  const { t } = useTranslation();

  const handleSwitch = (app: AppId) => {
    if (app === activeApp) return;
    localStorage.setItem(STORAGE_KEY, app);
    onSwitch(app);
  };
  const iconSize = 20;
  const appIconName: Record<AppId, string> = {
    claude: "claude",
    codex: "openai",
    gemini: "gemini",
    opencode: "opencode",
    openclaw: "openclaw",
  };
  const appDisplayName: Record<AppId, string> = {
    claude: "Claude",
    codex: "Codex",
    gemini: "Gemini",
    opencode: "OpenCode",
    openclaw: "OpenClaw",
  };

  // Filter apps based on visibility settings (default all visible)
  const appsToShow = ALL_APPS.filter((app) => {
    if (!visibleApps) return true;
    return visibleApps[app];
  });

  return (
    <div className="inline-flex bg-muted rounded-xl p-1 gap-1">
      <button
        type="button"
        onClick={() => onOpenWorkspace?.()}
        className={cn(
          "group inline-flex items-center px-3 h-8 rounded-md text-sm font-medium transition-all duration-200",
          isWorkspaceActive
            ? "bg-background text-foreground shadow-sm"
            : "text-muted-foreground hover:text-foreground hover:bg-background/50",
        )}
        title={t("workspace.manage", {
          defaultValue: "Workspace Manager",
        })}
      >
        <WorkspaceSyncIcon size={20} />
        <span
          className={cn(
            "transition-all duration-200 whitespace-nowrap overflow-hidden",
            compact
              ? "max-w-0 opacity-0 ml-0"
              : "max-w-[120px] opacity-100 ml-2",
          )}
        >
          {t("workspace.manage", {
            defaultValue: "Workspace",
          })}
        </span>
      </button>

      {appsToShow.map((app) => (
        <button
          key={app}
          type="button"
          onClick={() => handleSwitch(app)}
          className={cn(
            "group inline-flex items-center px-3 h-8 rounded-md text-sm font-medium transition-all duration-200",
            !isWorkspaceActive && activeApp === app
              ? "bg-background text-foreground shadow-sm"
              : "text-muted-foreground hover:text-foreground hover:bg-background/50",
          )}
        >
          <ProviderIcon
            icon={appIconName[app]}
            name={appDisplayName[app]}
            size={iconSize}
          />
          <span
            className={cn(
              "transition-all duration-200 whitespace-nowrap overflow-hidden",
              compact
                ? "max-w-0 opacity-0 ml-0"
                : "max-w-[80px] opacity-100 ml-2",
            )}
          >
            {appDisplayName[app]}
          </span>
        </button>
      ))}
    </div>
  );
}
