export interface DetectedGame {
  path: string;
  version: string;
  versionLabel: string;
}

export interface DllInjectionContext {
  path: string;
  hasPath: boolean;
  exists: boolean;
  isValidGameDir: boolean;
  detectedVersion: string;
  detectedVersionLabel: string;
  installStatus: string;
  installStatusLabel: string;
  notifierHost: string;
  timeoutMs: number;
  debugMode: boolean;
  debugOutputDir: string;
  fansEnabled: boolean;
  fansOutputDir: string;
}

export interface SaveDllInjectionConfigInput {
  path: string;
  notifierHost: string;
  timeoutMs: number;
}

export interface SaveDebugModeInput {
  path: string;
  debugMode: boolean;
}

export interface DllInjectionActionResult {
  context: DllInjectionContext;
  notice: string;
}
