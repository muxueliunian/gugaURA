import type { BootstrapState } from '@/features/bootstrap/types';

export type StatusTagType = 'danger' | 'info' | 'primary' | 'success' | 'warning';

export function resolveGameVersionType(version: string): StatusTagType {
  if (version === 'steam') {
    return 'success';
  }
  if (version === 'dmm') {
    return 'warning';
  }
  return 'info';
}

export function resolveInstallStatusType(status: string): StatusTagType {
  if (status === 'installed') {
    return 'success';
  }
  if (status === 'needsUpdate') {
    return 'danger';
  }
  if (status === 'notInstalled') {
    return 'warning';
  }
  return 'info';
}

export function resolveReceiverStatusType(receiverReady?: boolean | null): StatusTagType {
  return receiverReady ? 'success' : 'warning';
}

export function resolveReceiverStatusLabel(
  appState: BootstrapState | null | undefined,
): string {
  if (!appState) {
    return '未读取';
  }
  return appState.receiverReady ? '已就绪' : '启动异常';
}

export function resolveReceiverListenAddrSourceLabel(source: string): string {
  if (source === 'cli') {
    return 'CLI 参数';
  }
  if (source === 'env') {
    return '环境变量';
  }
  if (source === 'exeConfig') {
    return 'EXE 同级配置';
  }
  return '默认值';
}
