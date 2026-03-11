import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { resolveCommandError } from '@/app/services/tauri';
import { getOverviewBootstrapState } from '@/features/overview/api/overviewApi';
import type { BootstrapState } from '@/features/bootstrap/types';

export const useReceiverStore = defineStore('receiver', () => {
  const appState = ref<BootstrapState | null>(null);
  const error = ref('');
  const loading = ref(false);
  const hasLoaded = ref(false);
  let pendingLoad: Promise<void> | null = null;

  async function loadState(force = false): Promise<void> {
    if (pendingLoad) {
      return pendingLoad;
    }

    if (hasLoaded.value && !force) {
      return;
    }

    pendingLoad = (async () => {
      loading.value = true;
      error.value = '';
      try {
        appState.value = await getOverviewBootstrapState();
        hasLoaded.value = true;
      } catch (loadError) {
        error.value = resolveCommandError(loadError, '读取应用状态失败');
      } finally {
        loading.value = false;
      }
    })();

    try {
      await pendingLoad;
    } finally {
      pendingLoad = null;
    }
  }

  function clearError(): void {
    error.value = '';
  }

  const receiverReady = computed(() => appState.value?.receiverReady ?? false);

  return {
    appState,
    clearError,
    error,
    loadState,
    loading,
    receiverReady,
  };
});
