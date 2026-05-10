import { invokeCommand } from '@/app/services/tauri';
import type {
  AppUpdateCheckResult,
  SetAutostartEnabledInput,
  ToolSettingsActionResult,
  ToolSettingsContext,
} from '../types';

export async function getToolSettingsContext(): Promise<ToolSettingsContext> {
  return invokeCommand<ToolSettingsContext>('get_tool_settings_context');
}

export async function setAutostartEnabled(
  input: SetAutostartEnabledInput,
): Promise<ToolSettingsActionResult> {
  return invokeCommand<ToolSettingsActionResult>('set_autostart_enabled', { input });
}

export async function checkAppUpdate(): Promise<AppUpdateCheckResult> {
  return invokeCommand<AppUpdateCheckResult>('check_app_update');
}

export async function openLatestReleasePage(url?: string | null): Promise<void> {
  return invokeCommand<void>('open_latest_release_page', { url: url ?? null });
}
