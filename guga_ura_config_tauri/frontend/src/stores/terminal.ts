import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { resolveCommandError } from '@/app/services/tauri';
import { clearTerminalLogs, getTerminalSnapshot } from '@/features/terminal/api/terminalApi';
import type { TerminalSnapshot } from '@/features/terminal/types';

const DEFAULT_LOG_LIMIT = 600;
const POLLING_INTERVAL_MS = 1500;

export const useTerminalStore = defineStore('terminal', () => {
  const snapshot = ref<TerminalSnapshot | null>(null);
  const lastError = ref('');
  const loading = ref(false);
  const clearing = ref(false);
  const hasInitialized = ref(false);
  const lastUpdatedAt = ref<number | null>(null);
  const showPayloadSaveLogs = ref(true);

  let pendingRefresh: Promise<TerminalSnapshot | null> | null = null;
  let pollingTimer: number | null = null;
  let pollingActive = false;
  let pollingSessionId = 0;
  let snapshotEpoch = 0;

  async function initialize(force = false): Promise<void> {
    if (hasInitialized.value && !force) {
      return;
    }

    await refreshSnapshot(true);
    hasInitialized.value = true;
  }

  async function refreshSnapshot(showLoading = false): Promise<TerminalSnapshot | null> {
    if (pendingRefresh) {
      return pendingRefresh;
    }

    const requestEpoch = snapshotEpoch;
    pendingRefresh = (async () => {
      if (showLoading) {
        loading.value = true;
      }

      try {
        const nextSnapshot = await getTerminalSnapshot(DEFAULT_LOG_LIMIT);
        if (requestEpoch !== snapshotEpoch) {
          return snapshot.value;
        }

        snapshot.value = nextSnapshot;
        lastUpdatedAt.value = Date.now();
        lastError.value = '';
        return nextSnapshot;
      } catch (error) {
        if (requestEpoch === snapshotEpoch) {
          lastError.value = resolveCommandError(error, '读取终端日志快照失败');
        }
        return null;
      } finally {
        if (showLoading) {
          loading.value = false;
        }
      }
    })();

    try {
      return await pendingRefresh;
    } finally {
      pendingRefresh = null;
    }
  }

  async function clearLogs(): Promise<boolean> {
    if (clearing.value) {
      return false;
    }

    clearing.value = true;

    try {
      await clearTerminalLogs();
      snapshotEpoch += 1;
      snapshot.value = {
        receiverReady: receiverReady.value,
        receiverStatus: receiverStatus.value,
        receiverListenAddr: receiverListenAddr.value,
        receiverConfiguredListenAddr: runtimeConfiguredListenAddr.value,
        receiverListenAddrSource: receiverListenAddrSource.value,
        logs: [],
      };
      lastUpdatedAt.value = Date.now();
      lastError.value = '';
      return true;
    } catch (error) {
      lastError.value = resolveCommandError(error, '清空终端日志失败');
      return false;
    } finally {
      clearing.value = false;
    }
  }

  function startPolling(): void {
    if (pollingActive) {
      return;
    }

    pollingActive = true;
    pollingSessionId += 1;
    scheduleNextPolling(pollingSessionId);
  }

  function stopPolling(): void {
    pollingActive = false;
    pollingSessionId += 1;
    clearPollingTimer();
  }

  function clearError(): void {
    lastError.value = '';
  }

  function scheduleNextPolling(sessionId: number): void {
    clearPollingTimer();
    pollingTimer = window.setTimeout(() => {
      void runPollingTick(sessionId);
    }, POLLING_INTERVAL_MS);
  }

  async function runPollingTick(sessionId: number): Promise<void> {
    if (!pollingActive || sessionId !== pollingSessionId) {
      return;
    }

    await refreshSnapshot(false);

    if (!pollingActive || sessionId !== pollingSessionId) {
      return;
    }

    scheduleNextPolling(sessionId);
  }

  function clearPollingTimer(): void {
    if (pollingTimer === null) {
      return;
    }

    window.clearTimeout(pollingTimer);
    pollingTimer = null;
  }

  const logs = computed(() => snapshot.value?.logs ?? []);
  const visibleLogs = computed(() => {
    if (showPayloadSaveLogs.value) {
      return logs.value;
    }

    return logs.value.filter((line) => !isPayloadSaveSuccessLog(line));
  });
  const logCount = computed(() => logs.value.length);
  const visibleLogCount = computed(() => visibleLogs.value.length);
  const hiddenPayloadSaveLogCount = computed(() => logCount.value - visibleLogCount.value);

  const receiverReady = computed(() => snapshot.value?.receiverReady ?? false);
  const receiverStatus = computed(
    () => snapshot.value?.receiverStatus ?? '正在读取 Receiver 状态',
  );
  const receiverListenAddr = computed(() => snapshot.value?.receiverListenAddr ?? '--');
  const runtimeConfiguredListenAddr = computed(
    () => snapshot.value?.receiverConfiguredListenAddr ?? '--',
  );
  const receiverListenAddrSource = computed(
    () => snapshot.value?.receiverListenAddrSource ?? 'unknown',
  );

  return {
    clearError,
    clearLogs,
    clearing,
    hasInitialized,
    hiddenPayloadSaveLogCount,
    initialize,
    lastError,
    lastUpdatedAt,
    loading,
    logCount,
    logs,
    receiverListenAddr,
    receiverListenAddrSource,
    receiverReady,
    receiverStatus,
    refreshSnapshot,
    runtimeConfiguredListenAddr,
    showPayloadSaveLogs,
    startPolling,
    stopPolling,
    visibleLogCount,
    visibleLogs,
  };
});

function isPayloadSaveSuccessLog(line: string): boolean {
  return line.includes('保存 payload 成功:');
}
