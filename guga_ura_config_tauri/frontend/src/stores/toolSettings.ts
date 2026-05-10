import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { resolveCommandError } from '@/app/services/tauri';
import {
  checkAppUpdate,
  getToolSettingsContext,
  openLatestReleasePage,
  setAutostartEnabled,
} from '@/features/tool-settings/api/toolSettingsApi';
import type {
  AppUpdateCheckResult,
  ToolSettingsActionResult,
  ToolSettingsContext,
} from '@/features/tool-settings/types';

export const useToolSettingsStore = defineStore('toolSettings', () => {
  const context = ref<ToolSettingsContext | null>(null);
  const updateResult = ref<AppUpdateCheckResult | null>(null);
  const lastError = ref('');
  const loading = ref(false);
  const savingAutostart = ref(false);
  const checkingUpdate = ref(false);
  const openingReleasePage = ref(false);
  const hasInitialized = ref(false);

  async function initialize(force = false): Promise<void> {
    if (hasInitialized.value && !force) {
      return;
    }

    await loadContext();
    hasInitialized.value = true;
  }

  async function loadContext(): Promise<ToolSettingsContext | null> {
    loading.value = true;

    try {
      const nextContext = await getToolSettingsContext();
      context.value = nextContext;
      lastError.value = '';
      return nextContext;
    } catch (error) {
      lastError.value = resolveCommandError(error, '读取工具设置失败');
      return null;
    } finally {
      loading.value = false;
    }
  }

  async function updateAutostart(enabled: boolean): Promise<ToolSettingsActionResult | null> {
    savingAutostart.value = true;

    try {
      const result = await setAutostartEnabled({ enabled });
      context.value = result.context;
      lastError.value = '';
      return result;
    } catch (error) {
      lastError.value = resolveCommandError(error, '保存开机自启设置失败');
      await loadContext();
      return null;
    } finally {
      savingAutostart.value = false;
    }
  }

  async function runUpdateCheck(): Promise<AppUpdateCheckResult | null> {
    checkingUpdate.value = true;

    try {
      const result = await checkAppUpdate();
      updateResult.value = result;
      lastError.value = '';
      return result;
    } catch (error) {
      lastError.value = resolveCommandError(error, '检查更新失败');
      return null;
    } finally {
      checkingUpdate.value = false;
    }
  }

  async function openReleasePage(): Promise<boolean> {
    openingReleasePage.value = true;

    try {
      await openLatestReleasePage(releasePageUrl.value);
      lastError.value = '';
      return true;
    } catch (error) {
      lastError.value = resolveCommandError(error, '打开下载页面失败');
      return false;
    } finally {
      openingReleasePage.value = false;
    }
  }

  function clearError(): void {
    lastError.value = '';
  }

  const currentVersion = computed(() => context.value?.currentVersion ?? '');
  const autostartEnabled = computed(() => Boolean(context.value?.autostartEnabled));
  const releasePageUrl = computed(() => updateResult.value?.releasePageUrl || null);

  return {
    autostartEnabled,
    checkingUpdate,
    clearError,
    context,
    currentVersion,
    hasInitialized,
    initialize,
    lastError,
    loadContext,
    loading,
    openReleasePage,
    openingReleasePage,
    releasePageUrl,
    runUpdateCheck,
    savingAutostart,
    updateAutostart,
    updateResult,
  };
});
