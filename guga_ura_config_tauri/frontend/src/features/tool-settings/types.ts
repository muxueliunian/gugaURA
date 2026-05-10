export interface ToolSettingsContext {
  currentVersion: string;
  autostartEnabled: boolean;
}

export interface SetAutostartEnabledInput {
  enabled: boolean;
}

export interface ToolSettingsActionResult {
  context: ToolSettingsContext;
  notice: string;
}

export interface AppUpdateCheckResult {
  currentVersion: string;
  latestVersion: string;
  versionStatus: 'updateAvailable' | 'latest' | 'ahead';
  hasUpdate: boolean;
  releasePageUrl: string;
  downloadAssetUrl: string | null;
  publishedAt: string;
  summary: string;
}
