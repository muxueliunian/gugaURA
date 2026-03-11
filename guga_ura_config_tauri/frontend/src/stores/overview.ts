import { defineStore } from 'pinia';
import { ref } from 'vue';
import { resolveCommandError } from '@/app/services/tauri';
import {
  inspectOverviewGameDir,
  scanOverviewInstalledGames,
} from '@/features/overview/api/overviewApi';
import type { DetectedGame, InspectGameDirResult } from '@/features/bootstrap/types';
import { useReceiverStore } from '@/stores/receiver';

export const useOverviewStore = defineStore('overview', () => {
  const receiverStore = useReceiverStore();
  const detectedGames = ref<DetectedGame[]>([]);
  const inspectPath = ref('');
  const inspectResult = ref<InspectGameDirResult | null>(null);
  const lastError = ref('');
  const scanLoading = ref(false);
  const inspectLoading = ref(false);

  async function initialize(forceReceiver = false): Promise<void> {
    lastError.value = '';
    await Promise.allSettled([receiverStore.loadState(forceReceiver), refreshDetectedGames()]);
  }

  async function refreshDetectedGames(): Promise<void> {
    scanLoading.value = true;
    try {
      detectedGames.value = await scanOverviewInstalledGames();
    } catch (error) {
      lastError.value = resolveCommandError(error, '扫描已安装游戏失败');
    } finally {
      scanLoading.value = false;
    }
  }

  async function inspectGameDir(): Promise<void> {
    const normalizedPath = inspectPath.value.trim();
    if (!normalizedPath) {
      inspectResult.value = null;
      lastError.value = '请输入要检测的游戏目录';
      return;
    }

    inspectLoading.value = true;
    lastError.value = '';
    try {
      inspectResult.value = await inspectOverviewGameDir(normalizedPath);
    } catch (error) {
      lastError.value = resolveCommandError(error, '检测游戏目录失败');
    } finally {
      inspectLoading.value = false;
    }
  }

  async function inspectDetectedGame(path: string): Promise<void> {
    inspectPath.value = path;
    await inspectGameDir();
  }

  function clearError(): void {
    lastError.value = '';
  }

  return {
    clearError,
    detectedGames,
    initialize,
    inspectDetectedGame,
    inspectGameDir,
    inspectLoading,
    inspectPath,
    inspectResult,
    lastError,
    refreshDetectedGames,
    scanLoading,
  };
});
