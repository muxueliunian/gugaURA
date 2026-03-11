import { invokeCommand } from '@/app/services/tauri';
import type { BootstrapState, DetectedGame, InspectGameDirResult } from '../types';

export async function getBootstrapState(): Promise<BootstrapState> {
  return invokeCommand<BootstrapState>('get_bootstrap_state');
}

export async function scanInstalledGames(): Promise<DetectedGame[]> {
  return invokeCommand<DetectedGame[]>('scan_installed_games');
}

export async function inspectGameDir(path: string): Promise<InspectGameDirResult> {
  return invokeCommand<InspectGameDirResult>('inspect_game_dir', { path });
}
