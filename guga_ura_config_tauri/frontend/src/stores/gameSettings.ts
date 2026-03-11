import { defineStore } from 'pinia';
import { computed, reactive, ref } from 'vue';
import { resolveCommandError } from '@/app/services/tauri';
import {
  getGameSettingsContext,
  saveGameSettings,
} from '@/features/game-settings/api/gameSettingsApi';
import type {
  GameSettingsActionResult,
  GameSettingsContext,
  GameSettingsVsyncValue,
  SaveGameSettingsInput,
} from '@/features/game-settings/types';

export const useGameSettingsStore = defineStore('gameSettings', () => {
  const context = ref<GameSettingsContext | null>(null);
  const lastError = ref('');
  const loading = ref(false);
  const saving = ref(false);
  const hasInitialized = ref(false);

  const form = reactive({
    targetFps: -1,
    customFpsInput: '',
    vsyncCount: -1 as GameSettingsVsyncValue,
  });

  function applyContext(nextContext: GameSettingsContext): void {
    context.value = nextContext;
    form.targetFps = nextContext.targetFps;
    form.customFpsInput = isCustomFps(nextContext.targetFps) ? String(nextContext.targetFps) : '';
    form.vsyncCount = normalizeVsyncCount(nextContext.vsyncCount);
  }

  async function initialize(force = false): Promise<void> {
    if (hasInitialized.value && !force) {
      return;
    }

    await loadContext(resolveCurrentPath() || null);
    hasInitialized.value = true;
  }

  async function loadContext(path?: string | null): Promise<GameSettingsContext | null> {
    loading.value = true;

    try {
      const nextContext = await getGameSettingsContext(path ?? null);
      applyContext(nextContext);
      lastError.value = '';
      return nextContext;
    } catch (error) {
      lastError.value = resolveCommandError(error, '读取游戏设置失败');
      return null;
    } finally {
      loading.value = false;
    }
  }

  async function refreshCurrentContext(): Promise<GameSettingsContext | null> {
    return loadContext(resolveCurrentPath() || null);
  }

  async function saveSettings(): Promise<GameSettingsActionResult | null> {
    const input = buildSaveInput();
    if (!input) {
      return null;
    }

    saving.value = true;

    try {
      const result = await saveGameSettings(input);
      applyContext(result.context);
      lastError.value = '';
      return result;
    } catch (error) {
      lastError.value = resolveCommandError(error, '保存游戏设置失败');
      return null;
    } finally {
      saving.value = false;
    }
  }

  function selectPresetFps(targetFps: number): void {
    form.targetFps = targetFps;
    form.customFpsInput = '';
  }

  function updateCustomFpsInput(value: string): void {
    form.customFpsInput = value;

    const parsed = Number.parseInt(value.trim(), 10);
    if (Number.isInteger(parsed) && parsed > 0) {
      form.targetFps = parsed;
    }
  }

  function selectVsyncCount(value: GameSettingsVsyncValue): void {
    form.vsyncCount = value;
  }

  function clearError(): void {
    lastError.value = '';
  }

  function resolveCurrentPath(): string {
    return context.value?.path.trim() || '';
  }

  function buildSaveInput(): SaveGameSettingsInput | null {
    const path = resolveCurrentPath();
    if (!path) {
      lastError.value = '未检测到可用的游戏目录，请先确认游戏目录';
      return null;
    }

    if (saveDisabledReason.value) {
      lastError.value = saveDisabledReason.value;
      return null;
    }

    const targetFps = resolveTargetFpsForSave();
    if (targetFps === null) {
      lastError.value = customFpsError.value;
      return null;
    }

    return {
      path,
      targetFps,
      vsyncCount: form.vsyncCount,
    };
  }

  function resolveTargetFpsForSave(): number | null {
    if (isCustomFps(form.targetFps) || form.customFpsInput.trim()) {
      const parsed = Number.parseInt(form.customFpsInput.trim(), 10);
      if (!Number.isInteger(parsed) || parsed <= 0) {
        return null;
      }
      return parsed;
    }

    return form.targetFps;
  }

  const hasValidGameContext = computed(
    () => Boolean(context.value?.isValidGameDir && context.value?.detectedVersion !== 'unknown'),
  );
  const customFpsError = computed(() => {
    if (isCustomFps(form.targetFps) || form.customFpsInput.trim()) {
      const parsed = Number.parseInt(form.customFpsInput.trim(), 10);
      if (!Number.isInteger(parsed) || parsed <= 0) {
        return '自定义 FPS 必须是正整数';
      }
    }

    return '';
  });
  const saveDisabledReason = computed(() => {
    if (!hasValidGameContext.value) {
      return '未检测到有效游戏目录，请先确认游戏目录';
    }
    if (customFpsError.value) {
      return customFpsError.value;
    }
    return '';
  });

  return {
    clearError,
    context,
    customFpsError,
    form,
    hasInitialized,
    hasValidGameContext,
    initialize,
    lastError,
    loadContext,
    loading,
    refreshCurrentContext,
    saveDisabledReason,
    saveSettings,
    saving,
    selectPresetFps,
    selectVsyncCount,
    updateCustomFpsInput,
  };
});

function isCustomFps(value: number): boolean {
  return ![-1, 60, 120, 144].includes(value) && value > 0;
}

function normalizeVsyncCount(value: number): GameSettingsVsyncValue {
  return value === 0 || value === 1 ? value : -1;
}
