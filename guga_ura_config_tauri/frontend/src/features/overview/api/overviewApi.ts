import {
  getBootstrapState,
  inspectGameDir,
  scanInstalledGames,
} from '@/features/bootstrap/api/bootstrapApi';
import type {
  BootstrapState,
  DetectedGame,
  InspectGameDirResult,
} from '@/features/bootstrap/types';

export async function getOverviewBootstrapState(): Promise<BootstrapState> {
  return getBootstrapState();
}

export async function scanOverviewInstalledGames(): Promise<DetectedGame[]> {
  return scanInstalledGames();
}

export async function inspectOverviewGameDir(path: string): Promise<InspectGameDirResult> {
  return inspectGameDir(path);
}
