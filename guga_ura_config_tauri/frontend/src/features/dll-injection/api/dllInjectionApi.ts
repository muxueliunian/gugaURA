import { invokeCommand } from '@/app/services/tauri';
import type {
  DetectedGame,
  DllInjectionActionResult,
  DllInjectionContext,
  SaveDebugModeInput,
  SaveDllInjectionConfigInput,
} from '../types';

export async function scanDllInjectionInstalledGames(): Promise<DetectedGame[]> {
  return invokeCommand<DetectedGame[]>('scan_installed_games');
}

export async function pickDirectory(title?: string): Promise<string | null> {
  return invokeCommand<string | null>('pick_directory', { title: title ?? null });
}

export async function getDllInjectionContext(path?: string | null): Promise<DllInjectionContext> {
  return invokeCommand<DllInjectionContext>('get_dll_injection_context', {
    path: path ?? null,
  });
}

export async function saveDllInjectionConfig(
  input: SaveDllInjectionConfigInput,
): Promise<DllInjectionActionResult> {
  return invokeCommand<DllInjectionActionResult>('save_dll_injection_config', { input });
}

export async function installDllInjection(
  input: SaveDllInjectionConfigInput,
): Promise<DllInjectionActionResult> {
  return invokeCommand<DllInjectionActionResult>('install_dll_injection', { input });
}

export async function saveDebugMode(input: SaveDebugModeInput): Promise<DllInjectionActionResult> {
  return invokeCommand<DllInjectionActionResult>('save_debug_mode', { input });
}

export async function uninstallDllInjection(path: string): Promise<DllInjectionActionResult> {
  return invokeCommand<DllInjectionActionResult>('uninstall_dll_injection', { path });
}
