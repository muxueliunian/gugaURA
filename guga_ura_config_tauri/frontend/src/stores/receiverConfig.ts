import { defineStore } from 'pinia';
import { computed, reactive, ref } from 'vue';
import { resolveCommandError } from '@/app/services/tauri';
import { getTerminalSnapshot } from '@/features/terminal/api/terminalApi';
import type { TerminalSnapshot } from '@/features/terminal/types';
import {
  getReceiverRuntimeSettings,
  pickReceiverDirectory,
  saveReceiverRuntimeSettings,
} from '@/features/receiver-config/api/receiverConfigApi';
import type {
  ReceiverRuntimeSettings,
  ReceiverRuntimeSettingsActionResult,
  SaveReceiverRuntimeSettingsInput,
} from '@/features/receiver-config/types';

const SUMMARY_LOG_LIMIT = 1;

export const useReceiverConfigStore = defineStore('receiverConfig', () => {
  const runtimeSnapshot = ref<TerminalSnapshot | null>(null);
  const settings = ref<ReceiverRuntimeSettings | null>(null);
  const lastError = ref('');
  const runtimeLoading = ref(false);
  const settingsLoading = ref(false);
  const settingsSaving = ref(false);
  const browseFansDirLoading = ref(false);
  const hasInitialized = ref(false);
  const lastUpdatedAt = ref<number | null>(null);

  const form = reactive({
    receiverListenAddr: '',
    relayEnabled: false,
    relayTargetHost: '',
    fansEnabled: true,
    fansOutputDir: '',
  });

  let pendingRuntimeLoad: Promise<TerminalSnapshot | null> | null = null;
  let pendingSettingsLoad: Promise<ReceiverRuntimeSettings | null> | null = null;

  function applySettings(nextSettings: ReceiverRuntimeSettings): void {
    settings.value = nextSettings;
    form.receiverListenAddr = nextSettings.receiverListenAddr;
    form.relayEnabled = nextSettings.relayEnabled;
    form.relayTargetHost = nextSettings.relayTargetHost;
    form.fansEnabled = nextSettings.fansEnabled;
    form.fansOutputDir = nextSettings.fansOutputDir;
  }

  async function initialize(force = false): Promise<void> {
    if (hasInitialized.value && !force) {
      return;
    }

    await Promise.all([refreshRuntimeSummary(true), loadSettings(true)]);
    hasInitialized.value = true;
  }

  async function refreshRuntimeSummary(showLoading = false): Promise<TerminalSnapshot | null> {
    if (pendingRuntimeLoad) {
      return pendingRuntimeLoad;
    }

    pendingRuntimeLoad = (async () => {
      if (showLoading) {
        runtimeLoading.value = true;
      }

      try {
        const nextSnapshot = await getTerminalSnapshot(SUMMARY_LOG_LIMIT);
        runtimeSnapshot.value = nextSnapshot;
        lastUpdatedAt.value = Date.now();
        lastError.value = '';
        return nextSnapshot;
      } catch (error) {
        lastError.value = resolveCommandError(error, '读取 Receiver 运行状态失败');
        return null;
      } finally {
        if (showLoading) {
          runtimeLoading.value = false;
        }
      }
    })();

    try {
      return await pendingRuntimeLoad;
    } finally {
      pendingRuntimeLoad = null;
    }
  }

  async function loadSettings(force = false): Promise<ReceiverRuntimeSettings | null> {
    if (pendingSettingsLoad) {
      return pendingSettingsLoad;
    }

    if (settings.value && !force) {
      return settings.value;
    }

    pendingSettingsLoad = (async () => {
      settingsLoading.value = true;

      try {
        const nextSettings = await getReceiverRuntimeSettings();
        applySettings(nextSettings);
        lastError.value = '';
        return nextSettings;
      } catch (error) {
        lastError.value = resolveCommandError(error, '读取接收与转发配置失败');
        return null;
      } finally {
        settingsLoading.value = false;
      }
    })();

    try {
      return await pendingSettingsLoad;
    } finally {
      pendingSettingsLoad = null;
    }
  }

  async function saveSettings(): Promise<ReceiverRuntimeSettingsActionResult | null> {
    const input = buildSaveSettingsInput();
    if (!input) {
      return null;
    }

    settingsSaving.value = true;
    lastError.value = '';

    try {
      const result = await saveReceiverRuntimeSettings(input);
      applySettings(result.settings);
      return result;
    } catch (error) {
      lastError.value = resolveCommandError(error, '保存接收与转发配置失败');
      return null;
    } finally {
      settingsSaving.value = false;
    }
  }

  async function browseFansOutputDirectory(): Promise<string | null> {
    browseFansDirLoading.value = true;

    try {
      const pickedPath = await pickReceiverDirectory('选择 Fans 输出目录');
      if (pickedPath) {
        form.fansOutputDir = pickedPath;
      }
      return pickedPath;
    } catch (error) {
      lastError.value = resolveCommandError(error, '选择 Fans 输出目录失败');
      return null;
    } finally {
      browseFansDirLoading.value = false;
    }
  }

  function clearError(): void {
    lastError.value = '';
  }

  function buildSaveSettingsInput(): SaveReceiverRuntimeSettingsInput | null {
    if (receiverListenAddrError.value) {
      lastError.value = receiverListenAddrError.value;
      return null;
    }

    if (relayTargetError.value) {
      lastError.value = relayTargetError.value;
      return null;
    }

    return {
      receiverListenAddr: form.receiverListenAddr.trim(),
      relayEnabled: form.relayEnabled,
      relayTargetHost: form.relayTargetHost.trim(),
      fansEnabled: form.fansEnabled,
      fansOutputDir: form.fansOutputDir.trim(),
    };
  }

  const receiverReady = computed(() => runtimeSnapshot.value?.receiverReady ?? false);
  const receiverStatus = computed(
    () => runtimeSnapshot.value?.receiverStatus ?? '正在读取 Receiver 状态',
  );
  const receiverListenAddr = computed(() => runtimeSnapshot.value?.receiverListenAddr ?? '--');
  const runtimeConfiguredListenAddr = computed(
    () => runtimeSnapshot.value?.receiverConfiguredListenAddr ?? '--',
  );
  const receiverListenAddrSource = computed(
    () => runtimeSnapshot.value?.receiverListenAddrSource ?? 'unknown',
  );

  const configuredListenAddr = computed(
    () =>
      form.receiverListenAddr.trim()
      || settings.value?.receiverListenAddr
      || runtimeConfiguredListenAddr.value,
  );
  const relayTargetDisplay = computed(
    () => form.relayTargetHost.trim() || settings.value?.relayTargetHost || '未配置',
  );
  const fansOutputDirDisplay = computed(
    () => form.fansOutputDir.trim() || settings.value?.fansOutputDir || '默认: EXE 同级 fans/',
  );

  const receiverListenAddrError = computed(() => {
    const value = form.receiverListenAddr.trim();
    if (!value) {
      return '请输入 Receiver 监听地址';
    }

    const separatorIndex = value.lastIndexOf(':');
    if (separatorIndex <= 0 || separatorIndex === value.length - 1) {
      return '监听地址必须使用 host:port 格式';
    }

    const host = value.slice(0, separatorIndex).trim();
    const port = Number(value.slice(separatorIndex + 1).trim());
    if (!host) {
      return '监听地址缺少 host';
    }
    if (!Number.isInteger(port) || port <= 0 || port > 65535) {
      return '监听地址端口必须是 1-65535 的整数';
    }
    return '';
  });

  const relayTargetError = computed(() => {
    if (!form.relayEnabled) {
      return '';
    }

    const value = form.relayTargetHost.trim();
    if (!value) {
      return '开启 Relay 时必须填写下游地址';
    }
    if (!value.startsWith('http://') && !value.startsWith('https://')) {
      return 'Relay 目标地址必须以 http:// 或 https:// 开头';
    }
    return '';
  });

  const saveSettingsDisabledReason = computed(() => {
    if (settingsLoading.value) {
      return '正在读取接收与转发配置';
    }
    if (receiverListenAddrError.value) {
      return receiverListenAddrError.value;
    }
    if (relayTargetError.value) {
      return relayTargetError.value;
    }
    return '';
  });

  return {
    browseFansDirLoading,
    browseFansOutputDirectory,
    clearError,
    configuredListenAddr,
    fansOutputDirDisplay,
    form,
    hasInitialized,
    initialize,
    lastError,
    lastUpdatedAt,
    loadSettings,
    receiverListenAddr,
    receiverListenAddrError,
    receiverListenAddrSource,
    receiverReady,
    receiverStatus,
    refreshRuntimeSummary,
    relayTargetDisplay,
    relayTargetError,
    runtimeConfiguredListenAddr,
    runtimeLoading,
    saveSettings,
    saveSettingsDisabledReason,
    settings,
    settingsLoading,
    settingsSaving,
  };
});
