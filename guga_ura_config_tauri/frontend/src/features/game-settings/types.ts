export interface GameSettingsContext {
  path: string;
  hasPath: boolean;
  exists: boolean;
  isValidGameDir: boolean;
  detectedVersion: string;
  detectedVersionLabel: string;
  targetFps: number;
  vsyncCount: number;
}

export interface SaveGameSettingsInput {
  path: string;
  targetFps: number;
  vsyncCount: number;
}

export interface GameSettingsActionResult {
  context: GameSettingsContext;
  notice: string;
}

export type GameSettingsVsyncValue = -1 | 0 | 1;
