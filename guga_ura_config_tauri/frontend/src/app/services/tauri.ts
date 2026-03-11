type CommandArgs = Record<string, unknown> | undefined;
type TauriInvoke = <T>(command: string, args?: CommandArgs) => Promise<T>;

declare global {
  interface Window {
    __TAURI_INTERNALS__?: {
      invoke?: TauriInvoke;
    };
  }
}

export async function invokeCommand<T>(command: string, args?: CommandArgs): Promise<T> {
  const invoke = window.__TAURI_INTERNALS__?.invoke;
  if (!invoke) {
    throw new Error('window.__TAURI_INTERNALS__.invoke is unavailable');
  }
  return invoke<T>(command, args);
}

export function resolveCommandError(error: unknown, fallbackMessage: string): string {
  if (typeof error === 'string' && error.trim()) {
    return error;
  }

  if (error instanceof Error) {
    if (error.message.includes('__TAURI_INTERNALS__') || error.message.includes('window.__TAURI')) {
      return '当前页面需要在 Tauri 桌面环境中运行';
    }
    if (error.message.trim()) {
      return error.message;
    }
  }

  return fallbackMessage;
}
