import { defineStore } from 'pinia';
import { computed } from 'vue';
import { useOverviewStore } from '@/stores/overview';
import { useReceiverStore } from '@/stores/receiver';

export const useBootstrapStore = defineStore('bootstrap', () => {
  const receiverStore = useReceiverStore();
  const overviewStore = useOverviewStore();

  const appState = computed(() => receiverStore.appState);
  const bootstrapLoading = computed(() => receiverStore.loading);
  const detectedGames = computed(() => overviewStore.detectedGames);
  const inspectLoading = computed(() => overviewStore.inspectLoading);
  const inspectResult = computed(() => overviewStore.inspectResult);
  const scanLoading = computed(() => overviewStore.scanLoading);
  const lastError = computed(() => overviewStore.lastError || receiverStore.error);
  const inspectPath = computed({
    get: () => overviewStore.inspectPath,
    set: (value: string) => {
      overviewStore.inspectPath = value;
    },
  });

  async function initialize(): Promise<void> {
    await overviewStore.initialize(true);
  }

  async function loadBootstrapState(): Promise<void> {
    await receiverStore.loadState(true);
  }

  async function refreshDetectedGames(): Promise<void> {
    await overviewStore.refreshDetectedGames();
  }

  async function inspectGameDir(): Promise<void> {
    await overviewStore.inspectGameDir();
  }

  async function inspectDetectedGame(path: string): Promise<void> {
    await overviewStore.inspectDetectedGame(path);
  }

  function clearError(): void {
    receiverStore.clearError();
    overviewStore.clearError();
  }

  return {
    appState,
    bootstrapLoading,
    clearError,
    detectedGames,
    initialize,
    inspectDetectedGame,
    inspectGameDir,
    inspectLoading,
    inspectPath,
    inspectResult,
    lastError,
    loadBootstrapState,
    refreshDetectedGames,
    scanLoading,
  };
});
