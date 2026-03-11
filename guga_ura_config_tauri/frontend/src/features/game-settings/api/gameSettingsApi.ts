import { invokeCommand } from '@/app/services/tauri';
import type {
  GameSettingsActionResult,
  GameSettingsContext,
  SaveGameSettingsInput,
} from '../types';

export async function getGameSettingsContext(
  path?: string | null,
): Promise<GameSettingsContext> {
  return invokeCommand<GameSettingsContext>('get_game_settings_context', {
    path: path ?? null,
  });
}

export async function saveGameSettings(
  input: SaveGameSettingsInput,
): Promise<GameSettingsActionResult> {
  return invokeCommand<GameSettingsActionResult>('save_game_settings', { input });
}
