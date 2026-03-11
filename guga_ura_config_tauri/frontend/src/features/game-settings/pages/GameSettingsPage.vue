<template>
  <div class="game-settings-page app-page">
    <PageHeader
      eyebrow="性能参数"
      title="游戏设置"
      description="读取并保存 FPS 与垂直同步设置，页面会优先恢复当前可用游戏目录。"
    >
      <template #actions>
        <el-button
          :loading="loading || saving"
          @click="handleRefresh"
        >
          刷新当前设置
        </el-button>
      </template>
    </PageHeader>

    <el-alert
      v-if="pageError"
      class="app-page__alert"
      type="error"
      :closable="true"
      show-icon
      title="游戏设置页操作失败"
      :description="pageError"
      @close="handleClearError"
    />

    <div class="game-settings-page__workspace">
      <SectionCard title="游戏状态">
        <PageState
          v-if="loading && !context"
          state="loading"
          title="正在读取游戏设置"
          description="等待桌面端返回当前 FPS / 垂直同步配置。"
        />

        <template v-else>
          <el-alert
            v-if="!hasValidGameContext"
            type="warning"
            :closable="false"
            show-icon
            title="未检测到有效游戏目录"
          >
            <template #default>
              <p class="game-settings-page__hint">
                当前页面只负责 FPS / 垂直同步读写。若未自动恢复目录，可前往 DLL 注入页手动选择并检测。
              </p>
            </template>
          </el-alert>

          <el-descriptions
            class="game-settings-page__context"
            :column="1"
            border
            size="small"
          >
            <el-descriptions-item label="游戏目录">
              <span class="game-settings-page__code">{{ context?.path || '尚未选择' }}</span>
            </el-descriptions-item>
            <el-descriptions-item label="目录状态">
              <StatusBadge
                :label="context?.isValidGameDir ? '已确认' : '待确认'"
                :type="context?.isValidGameDir ? 'success' : 'warning'"
              />
            </el-descriptions-item>
            <el-descriptions-item label="识别版本">
              <StatusBadge
                :label="context?.detectedVersionLabel ?? '未知版本'"
                :type="resolveGameVersionType(context?.detectedVersion ?? 'unknown')"
              />
            </el-descriptions-item>
            <el-descriptions-item label="当前 FPS">
              <span class="game-settings-page__text">{{ targetFpsLabel }}</span>
            </el-descriptions-item>
            <el-descriptions-item label="当前垂直同步">
              <span class="game-settings-page__text">{{ vsyncLabel }}</span>
            </el-descriptions-item>
          </el-descriptions>

          <div class="game-settings-page__section-actions">
            <el-button
              type="primary"
              plain
              @click="handleGoToDllInjection"
            >
              前往 DLL 注入页
            </el-button>
          </div>
        </template>
      </SectionCard>

      <SectionCard
        title="FPS / 垂直同步设置"
        description="目标 FPS 与垂直同步保存后会同步写回游戏目录和 EXE 同级配置文件。"
      >
        <el-form label-position="top">
          <el-form-item label="目标 FPS">
            <div class="game-settings-page__chip-row">
              <el-button
                v-for="option in fpsOptions"
                :key="option.value"
                :type="form.targetFps === option.value ? 'primary' : 'default'"
                @click="handleSelectPresetFps(option.value)"
              >
                {{ option.label }}
              </el-button>
            </div>
          </el-form-item>

          <el-form-item
            label="自定义 FPS"
            :error="customFpsError || undefined"
          >
            <el-input
              class="game-settings-page__custom-fps-input"
              :model-value="form.customFpsInput"
              placeholder="例如 90"
              @input="handleCustomFpsInput"
            />
            <p class="game-settings-page__hint">
              输入正整数后会自动把目标 FPS 切换为该自定义值；如需回到默认或预设，请点击上方按钮。
            </p>
          </el-form-item>

          <el-form-item label="垂直同步">
            <div class="game-settings-page__chip-row">
              <el-button
                v-for="option in vsyncOptions"
                :key="option.value"
                :type="form.vsyncCount === option.value ? 'primary' : 'default'"
                @click="handleSelectVsync(option.value)"
              >
                {{ option.label }}
              </el-button>
            </div>
          </el-form-item>
        </el-form>

        <div class="game-settings-page__action-row">
          <el-button
            type="primary"
            :disabled="Boolean(saveDisabledReason)"
            :loading="saving"
            @click="handleSave"
          >
            保存游戏设置
          </el-button>
        </div>

        <p class="game-settings-page__hint">
          {{ saveDisabledReason || '本页会优先使用当前有效游戏目录保存 FPS / 垂直同步。Notifier、timeout 与 Fans 配置请在 DLL 注入页调整。' }}
        </p>
      </SectionCard>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ElAlert } from 'element-plus/es/components/alert/index';
import { ElButton } from 'element-plus/es/components/button/index';
import { ElDescriptions, ElDescriptionsItem } from 'element-plus/es/components/descriptions/index';
import { ElForm, ElFormItem } from 'element-plus/es/components/form/index';
import { ElInput } from 'element-plus/es/components/input/index';
import { ElMessage } from 'element-plus/es/components/message/index';
import { storeToRefs } from 'pinia';
import { computed, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import SectionCard from '@/components/SectionCard.vue';
import StatusBadge from '@/components/StatusBadge.vue';
import PageState from '@/components/feedback/PageState.vue';
import PageHeader from '@/components/layout/PageHeader.vue';
import { resolveGameVersionType } from '@/app/utils/status';
import type { GameSettingsVsyncValue } from '@/features/game-settings/types';
import { useGameSettingsStore } from '@/stores/gameSettings';

defineOptions({ name: 'GameSettingsPage' });

const router = useRouter();
const gameSettingsStore = useGameSettingsStore();
const {
  context,
  customFpsError,
  form,
  hasValidGameContext,
  lastError,
  loading,
  saveDisabledReason,
  saving,
} = storeToRefs(gameSettingsStore);

const fpsOptions = [
  { label: '默认', value: -1 },
  { label: '60', value: 60 },
  { label: '120', value: 120 },
  { label: '144', value: 144 },
];
const vsyncOptions: Array<{ label: string; value: GameSettingsVsyncValue }> = [
  { label: '默认', value: -1 },
  { label: '关闭', value: 0 },
  { label: '开启', value: 1 },
];

const pageError = computed(() => lastError.value);
const targetFpsLabel = computed(() => {
  if (form.value.targetFps === -1) {
    return '默认';
  }
  return String(form.value.targetFps);
});
const vsyncLabel = computed(() => {
  if (form.value.vsyncCount === 0) {
    return '关闭';
  }
  if (form.value.vsyncCount === 1) {
    return '开启';
  }
  return '默认';
});

async function handleRefresh(): Promise<void> {
  await gameSettingsStore.refreshCurrentContext();
}

function handleSelectPresetFps(targetFps: number): void {
  gameSettingsStore.selectPresetFps(targetFps);
}

function handleCustomFpsInput(value: string | number): void {
  gameSettingsStore.updateCustomFpsInput(String(value ?? ''));
}

function handleSelectVsync(value: GameSettingsVsyncValue): void {
  gameSettingsStore.selectVsyncCount(value);
}

async function handleSave(): Promise<void> {
  const result = await gameSettingsStore.saveSettings();
  if (!result) {
    return;
  }

  ElMessage.success(result.notice);
}

function handleGoToDllInjection(): void {
  void router.push('/dll-injection');
}

function handleClearError(): void {
  gameSettingsStore.clearError();
}

onMounted(() => {
  void gameSettingsStore.initialize();
});
</script>

<style scoped>
.game-settings-page {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.game-settings-page__workspace {
  display: grid;
  grid-template-columns: minmax(0, 0.95fr) minmax(0, 1.05fr);
  gap: 20px;
  align-items: start;
}

.game-settings-page__context {
  margin-top: 16px;
}

.game-settings-page__section-actions,
.game-settings-page__action-row {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.game-settings-page__section-actions {
  margin-top: 16px;
}

.game-settings-page__chip-row {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.game-settings-page__custom-fps-input {
  width: 220px;
  max-width: 100%;
}

.game-settings-page__hint,
.game-settings-page__text {
  margin: 12px 0 0;
  color: var(--app-text-secondary);
  font-size: 12px;
  line-height: 1.6;
}

.game-settings-page__code {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 11px;
  word-break: break-all;
}

@media (max-width: 1100px) {
  .game-settings-page__workspace {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 760px) {
  .game-settings-page__custom-fps-input {
    width: 100%;
  }
}
</style>
