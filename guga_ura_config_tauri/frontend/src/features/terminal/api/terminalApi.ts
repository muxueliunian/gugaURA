import { invokeCommand } from '@/app/services/tauri';
import type { TerminalSnapshot } from '../types';

export async function getTerminalSnapshot(limit = 600): Promise<TerminalSnapshot> {
  return invokeCommand<TerminalSnapshot>('get_terminal_snapshot', { limit });
}

export async function clearTerminalLogs(): Promise<void> {
  return invokeCommand<void>('clear_terminal_logs');
}
