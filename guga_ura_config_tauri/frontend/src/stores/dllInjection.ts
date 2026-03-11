import { defineStore } from 'pinia';
import { computed, reactive, ref } from 'vue';
import { resolveCommandError } from '@/app/services/tauri';
import {
  getDllInjectionContext,
  installDllInjection,
  saveDebugMode as saveDebugModeRequest,
  saveDllInjectionConfig,
  scanDllInjectionInstalledGames,
  uninstallDllInjection,
  pickDirectory,
} from '@/features/dll-injection/api/dllInjectionApi';
import type {
  DetectedGame,
  DllInjectionActionResult,
  DllInjectionContext,
  SaveDllInjectionConfigInput,
} from '@/features/dll-injection/types';

export const useDllInjectionStore = defineStore('dllInjection', () => {
  const context = ref<DllInjectionContext | null>(null);
  const detectedGames = ref<DetectedGame[]>([]);
  const pathInput = ref('');
  const lastError = ref('');
  const hasInitialized = ref(false);

  const contextLoading = ref(false);
  const scanLoading = ref(false);
  const browseGameDirLoading = ref(false);
  const actionLoading = ref(false);

  const form = reactive({
    notifierHost: '',
    timeoutInput: '100',
  });

  function applyContext(nextContext: DllInjectionContext): void {
    context.value = nextContext;
    pathInput.value = nextContext.path;
    form.notifierHost = nextContext.notifierHost;
    form.timeoutInput = String(nextContext.timeoutMs);
  }

  async function initialize(force = false): Promise<void> {
    if (hasInitialized.value && !force) {
      return;
    }

    await loadContext(pathInput.value.trim() || context.value?.path || null);
    hasInitialized.value = true;
  }

  async function loadContext(path?: string | null): Promise<DllInjectionContext | null> {
    contextLoading.value = true;
    lastError.value = '';

    try {
      const nextContext = await getDllInjectionContext(path ?? null);
      applyContext(nextContext);
      return nextContext;
    } catch (error) {
      lastError.value = resolveCommandError(error, '读取 DLL 注入上下文失败');
      return null;
    } finally {
      contextLoading.value = false;
    }
  }

  async function refreshCurrentContext(): Promise<DllInjectionContext | null> {
    return loadContext(pathInput.value.trim() || context.value?.path || null);
  }

  async function inspectPath(): Promise<DllInjectionContext | null> {
    const normalizedPath = pathInput.value.trim();
    if (!normalizedPath) {
      lastError.value = '请输入要检测的游戏目录';
      return null;
    }

    return loadContext(normalizedPath);
  }

  async function scanGames(): Promise<void> {
    scanLoading.value = true;
    lastError.value = '';

    try {
      detectedGames.value = await scanDllInjectionInstalledGames();
    } catch (error) {
      lastError.value = resolveCommandError(error, '扫描已安装游戏失败');
    } finally {
      scanLoading.value = false;
    }
  }

  async function selectDetectedGame(path: string): Promise<DllInjectionContext | null> {
    pathInput.value = path;
    return loadContext(path);
  }

  async function browseGameDirectory(): Promise<DllInjectionContext | null> {
    browseGameDirLoading.value = true;

    try {
      const pickedPath = await pickDirectory('选择游戏目录 (包含 umamusume.exe)');
      if (!pickedPath) {
        return null;
      }

      pathInput.value = pickedPath;
      return loadContext(pickedPath);
    } catch (error) {
      lastError.value = resolveCommandError(error, '打开目录选择框失败');
      return null;
    } finally {
      browseGameDirLoading.value = false;
    }
  }

  async function saveConfig(): Promise<DllInjectionActionResult | null> {
    const input = buildSaveInput();
    if (!input) {
      return null;
    }

    actionLoading.value = true;
    lastError.value = '';

    try {
      const result = await saveDllInjectionConfig(input);
      applyContext(result.context);
      return result;
    } catch (error) {
      lastError.value = resolveCommandError(error, '保存 DLL 注入配置失败');
      return null;
    } finally {
      actionLoading.value = false;
    }
  }

  async function install(): Promise<DllInjectionActionResult | null> {
    const input = buildSaveInput();
    if (!input) {
      return null;
    }

    actionLoading.value = true;
    lastError.value = '';

    try {
      const result = await installDllInjection(input);
      applyContext(result.context);
      return result;
    } catch (error) {
      lastError.value = resolveCommandError(error, '安装 DLL 失败');
      return null;
    } finally {
      actionLoading.value = false;
    }
  }

  async function saveDebugMode(debugMode: boolean): Promise<DllInjectionActionResult | null> {
    const normalizedPath = pathInput.value.trim() || context.value?.path.trim() || '';
    if (!normalizedPath) {
      lastError.value = '请先选择游戏目录';
      return null;
    }

    actionLoading.value = true;
    lastError.value = '';

    try {
      const result = await saveDebugModeRequest({
        path: normalizedPath,
        debugMode,
      });
      applyContext(result.context);
      return result;
    } catch (error) {
      lastError.value = resolveCommandError(error, '保存 Debug 模式失败');
      return null;
    } finally {
      actionLoading.value = false;
    }
  }

  async function uninstall(): Promise<DllInjectionActionResult | null> {
    const normalizedPath = pathInput.value.trim() || context.value?.path.trim() || '';
    if (!normalizedPath) {
      lastError.value = '请先选择游戏目录';
      return null;
    }

    actionLoading.value = true;
    lastError.value = '';

    try {
      const result = await uninstallDllInjection(normalizedPath);
      applyContext(result.context);
      return result;
    } catch (error) {
      lastError.value = resolveCommandError(error, '卸载 DLL 失败');
      return null;
    } finally {
      actionLoading.value = false;
    }
  }

  function buildSaveInput(): SaveDllInjectionConfigInput | null {
    const normalizedPath = pathInput.value.trim() || context.value?.path.trim() || '';
    if (!normalizedPath) {
      lastError.value = '请先选择游戏目录';
      return null;
    }

    if (notifierError.value) {
      lastError.value = notifierError.value;
      return null;
    }

    if (timeoutError.value) {
      lastError.value = timeoutError.value;
      return null;
    }

    return {
      path: normalizedPath,
      notifierHost: form.notifierHost.trim(),
      timeoutMs: Number(form.timeoutInput.trim()),
    };
  }

  function clearError(): void {
    lastError.value = '';
  }

  const hasValidGameContext = computed(
    () => Boolean(context.value?.isValidGameDir && context.value?.detectedVersion !== 'unknown'),
  );
  const isInstalled = computed(() => context.value?.installStatus === 'installed');
  const notifierError = computed(() => {
    const value = form.notifierHost.trim();
    if (!value) {
      return '请输入 Notifier 发送地址';
    }
    if (!value.startsWith('http://') && !value.startsWith('https://')) {
      return 'Notifier 发送地址必须以 http:// 或 https:// 开头';
    }
    return '';
  });
  const timeoutError = computed(() => {
    const value = form.timeoutInput.trim();
    if (!value) {
      return '请输入超时';
    }

    const timeout = Number(value);
    if (!Number.isInteger(timeout) || timeout <= 0) {
      return '超时必须是大于 0 的整数';
    }

    return '';
  });
  const saveDisabledReason = computed(() => {
    if (!hasValidGameContext.value) {
      return '请先选择并检测有效游戏目录';
    }
    if (notifierError.value) {
      return notifierError.value;
    }
    if (timeoutError.value) {
      return timeoutError.value;
    }
    return '';
  });
  const installDisabledReason = computed(() => {
    if (!hasValidGameContext.value) {
      return '请先选择并检测有效游戏目录';
    }
    if (isInstalled.value) {
      return '当前已安装，如需重装请先卸载';
    }
    return '';
  });
  const uninstallDisabledReason = computed(() => {
    if (!hasValidGameContext.value) {
      return '请先选择并检测有效游戏目录';
    }
    if (!isInstalled.value) {
      return '当前未安装，无需卸载';
    }
    return '';
  });

  return {
    actionLoading,
    browseGameDirLoading,
    browseGameDirectory,
    clearError,
    context,
    contextLoading,
    detectedGames,
    form,
    hasValidGameContext,
    initialize,
    install,
    installDisabledReason,
    inspectPath,
    isInstalled,
    lastError,
    loadContext,
    notifierError,
    pathInput,
    refreshCurrentContext,
    saveConfig,
    saveDebugMode,
    saveDisabledReason,
    scanGames,
    scanLoading,
    selectDetectedGame,
    timeoutError,
    uninstall,
    uninstallDisabledReason,
  };
});
