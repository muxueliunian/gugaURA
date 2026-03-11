import { invokeCommand } from '@/app/services/tauri';
import type {
  ReceiverRuntimeSettings,
  ReceiverRuntimeSettingsActionResult,
  SaveReceiverRuntimeSettingsInput,
} from '../types';

export async function getReceiverRuntimeSettings(): Promise<ReceiverRuntimeSettings> {
  return invokeCommand<ReceiverRuntimeSettings>('get_receiver_runtime_settings');
}

export async function saveReceiverRuntimeSettings(
  input: SaveReceiverRuntimeSettingsInput,
): Promise<ReceiverRuntimeSettingsActionResult> {
  return invokeCommand<ReceiverRuntimeSettingsActionResult>('save_receiver_runtime_settings', {
    input: {
      receiverListenAddr: input.receiverListenAddr,
      relayEnabled: input.relayEnabled,
      relayTargetHost: input.relayTargetHost || null,
      fansEnabled: input.fansEnabled,
      fansOutputDir: input.fansOutputDir || null,
    },
  });
}

export async function pickReceiverDirectory(title?: string): Promise<string | null> {
  return invokeCommand<string | null>('pick_directory', { title: title ?? null });
}
