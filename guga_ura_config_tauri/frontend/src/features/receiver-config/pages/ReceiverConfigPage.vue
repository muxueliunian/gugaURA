<template>
  <div class="receiver-config-page app-page">
    <PageHeader
      eyebrow="接收与转发"
      title="接收&转发配置"
      description="负责 Receiver 监听、Relay 二次转发和 Fans 聚合设置；保存后下次启动配置工具生效。"
    >
      <template #actions>
        <el-button
          :loading="runtimeLoading || settingsLoading"
          @click="handleReloadAll"
        >
          重新读取
        </el-button>
      </template>
    </PageHeader>

    <el-alert
      v-if="pageError"
      class="app-page__alert"
      type="error"
      :closable="true"
      show-icon
      title="接收&转发配置操作失败"
      :description="pageError"
      @close="handleClearError"
    />

    <div class="receiver-config-page__metrics-row">
      <InfoCard label="Receiver 状态">
        <StatusBadge
          :label="receiverReady ? '已就绪' : '未就绪'"
          :type="resolveReceiverStatusType(receiverReady)"
        />
        <div class="receiver-config-page__metric-detail">{{ receiverMetricDetail }}</div>
      </InfoCard>
      <InfoCard
        label="当前实际监听地址"
        :value="receiverListenAddr"
        :description="`来源：${receiverListenAddrSourceLabel}`"
      />
      <InfoCard
        label="配置监听地址"
        :value="configuredListenAddr"
        description="保存后下次启动配置工具生效。"
      />
      <InfoCard label="Relay 状态">
        <StatusBadge
          :label="form.relayEnabled ? '已开启' : '已关闭'"
          :type="form.relayEnabled ? 'success' : 'info'"
        />
        <div class="receiver-config-page__metric-detail">{{ relayMetricDetail }}</div>
      </InfoCard>
    </div>

    <div class="receiver-config-page__workspace">
      <SectionCard
        title="当前运行状态"
        description="展示当前进程实际监听地址、启动时配置来源和最近读取时间。"
      >
        <el-descriptions
          :column="1"
          border
          size="small"
        >
          <el-descriptions-item label="运行状态">
            <StatusBadge
              :label="receiverReady ? '已就绪' : '未就绪'"
              :type="resolveReceiverStatusType(receiverReady)"
            />
          </el-descriptions-item>
          <el-descriptions-item label="当前实际监听地址">
            <span class="receiver-config-page__code">{{ receiverListenAddr }}</span>
          </el-descriptions-item>
          <el-descriptions-item label="配置中的监听地址">
            <span class="receiver-config-page__code">{{ runtimeConfiguredListenAddr }}</span>
          </el-descriptions-item>
          <el-descriptions-item label="监听地址来源">
            <span class="receiver-config-page__text">{{ receiverListenAddrSourceLabel }}</span>
          </el-descriptions-item>
          <el-descriptions-item label="当前状态文案">
            <span class="receiver-config-page__code">{{ receiverStatus }}</span>
          </el-descriptions-item>
          <el-descriptions-item label="最近读取">
            <span class="receiver-config-page__text">{{ lastUpdatedLabel }}</span>
          </el-descriptions-item>
          <el-descriptions-item label="生效提示">
            <span class="receiver-config-page__text">{{ runtimeEffectHint }}</span>
          </el-descriptions-item>
        </el-descriptions>
      </SectionCard>

      <div class="receiver-config-page__stack">
        <SectionCard
          title="接收&转发配置"
          description="设置 Receiver 监听谁、是否执行 Relay 二次转发，以及 Relay 发给谁。"
        >
          <el-form label-position="top">
            <el-form-item
              label="Receiver 监听地址"
              :error="receiverListenAddrError || undefined"
            >
              <el-input
                v-model="form.receiverListenAddr"
                placeholder="127.0.0.1:4693"
              />
              <p class="receiver-config-page__field-hint">
                Receiver 在本地监听 DLL 发来的数据；这里保存的是下次启动配置工具时要绑定的 host:port。
              </p>
            </el-form-item>

            <el-form-item label="Relay 二次转发">
              <div class="receiver-config-page__switch-row">
                <el-switch
                  v-model="form.relayEnabled"
                  inline-prompt
                  active-text="开"
                  inactive-text="关"
                />
                <span>开启后，Receiver 在本地处理完成后会把原始 msgpack 再发给下游地址。</span>
              </div>
            </el-form-item>

            <el-form-item
              label="Relay 目标地址"
              :error="relayTargetError || undefined"
            >
              <el-input
                v-model="form.relayTargetHost"
                :disabled="!form.relayEnabled"
                placeholder="http://127.0.0.1:4800"
              />
              <p class="receiver-config-page__field-hint">
                {{ relayHint }}
              </p>
            </el-form-item>
          </el-form>
        </SectionCard>

        <SectionCard
          title="Fans 聚合配置"
          description="独立管理 Fans 聚合保存开关与输出目录，避免与 Relay 配置混在一起。"
        >
          <el-form label-position="top">
            <el-form-item label="Fans 聚合保存">
              <div class="receiver-config-page__switch-row">
                <el-switch
                  v-model="form.fansEnabled"
                  inline-prompt
                  active-text="开"
                  inactive-text="关"
                />
                <span>开启后按 `viewer_id` 覆盖更新 Fans 聚合输出。</span>
              </div>
            </el-form-item>

            <el-form-item label="Fans 输出目录">
              <div class="receiver-config-page__path-bar">
                <el-input
                  v-model="form.fansOutputDir"
                  :disabled="!form.fansEnabled"
                  placeholder="默认: EXE 同级 fans/"
                />
                <el-button
                  :disabled="!form.fansEnabled"
                  :loading="browseFansDirLoading"
                  @click="handleBrowseFansDir"
                >
                  选择目录
                </el-button>
              </div>
              <p class="receiver-config-page__field-hint">
                {{
                  form.fansEnabled
                    ? `当前保存目录：${fansOutputDirDisplay}`
                    : '关闭后会保留配置值，但当前运行不会继续更新 Fans 聚合输出。'
                }}
              </p>
            </el-form-item>
          </el-form>
        </SectionCard>

        <SectionCard
          title="保存操作"
          description="当前运行实例不会立即切换监听参数；保存后的配置会在下次启动配置工具时生效。"
        >
          <div class="receiver-config-page__action-row">
            <el-button
              type="primary"
              :disabled="Boolean(saveSettingsDisabledReason)"
              :loading="settingsSaving"
              @click="handleSaveSettings"
            >
              保存 Receiver 设置
            </el-button>
            <el-button
              :loading="runtimeLoading || settingsLoading"
              @click="handleReloadAll"
            >
              重新读取
            </el-button>
          </div>

          <p class="receiver-config-page__field-hint">
            {{ saveSettingsDisabledReason || operationHint }}
          </p>
        </SectionCard>
      </div>
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
import { ElSwitch } from 'element-plus/es/components/switch/index';
import { storeToRefs } from 'pinia';
import { computed, onMounted } from 'vue';
import SectionCard from '@/components/SectionCard.vue';
import StatusBadge from '@/components/StatusBadge.vue';
import InfoCard from '@/components/display/InfoCard.vue';
import PageHeader from '@/components/layout/PageHeader.vue';
import {
  resolveReceiverListenAddrSourceLabel,
  resolveReceiverStatusType,
} from '@/app/utils/status';
import { useReceiverConfigStore } from '@/stores/receiverConfig';

defineOptions({ name: 'ReceiverConfigPage' });

const receiverConfigStore = useReceiverConfigStore();
const {
  browseFansDirLoading,
  configuredListenAddr,
  fansOutputDirDisplay,
  form,
  lastError,
  lastUpdatedAt,
  receiverListenAddr,
  receiverListenAddrError,
  receiverListenAddrSource,
  receiverReady,
  receiverStatus,
  relayTargetDisplay,
  relayTargetError,
  runtimeConfiguredListenAddr,
  runtimeLoading,
  saveSettingsDisabledReason,
  settingsLoading,
  settingsSaving,
} = storeToRefs(receiverConfigStore);

const pageError = computed(() => lastError.value);
const receiverListenAddrSourceLabel = computed(() =>
  resolveReceiverListenAddrSourceLabel(receiverListenAddrSource.value),
);
const receiverMetricDetail = computed(() => {
  if (receiverReady.value) {
    return receiverStatus.value;
  }

  return `当前未就绪：${receiverStatus.value}`;
});
const relayMetricDetail = computed(() => {
  if (!form.value.relayEnabled) {
    return '关闭后仅在本地处理，不再执行二次转发。';
  }

  return `下游地址：${relayTargetDisplay.value}`;
});
const relayHint = computed(() => {
  if (!form.value.relayEnabled) {
    return '关闭时仅本地处理，不会向下游发起二次转发。';
  }

  return '下游地址用于接收 Receiver 的二次转发原始 msgpack，请勿指向当前 Receiver 自身。';
});
const runtimeEffectHint = computed(() => {
  if (!lastUpdatedAt.value) {
    return '正在读取当前运行实例状态。';
  }

  return '当前运行实例继续沿用启动时配置；保存后的新配置会在下次启动配置工具时接管。';
});
const operationHint = computed(
  () =>
    `当前保存值：监听 ${configuredListenAddr.value}；Relay ${form.value.relayEnabled ? '开启' : '关闭'}；Fans ${form.value.fansEnabled ? '开启' : '关闭'}。`,
);
const lastUpdatedLabel = computed(() => {
  if (!lastUpdatedAt.value) {
    return '尚未读取';
  }

  return new Date(lastUpdatedAt.value).toLocaleString('zh-CN', {
    hour12: false,
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  });
});

async function handleReloadAll(): Promise<void> {
  await Promise.all([
    receiverConfigStore.refreshRuntimeSummary(true),
    receiverConfigStore.loadSettings(true),
  ]);
}

async function handleSaveSettings(): Promise<void> {
  const result = await receiverConfigStore.saveSettings();
  if (!result) {
    return;
  }

  ElMessage.success(result.notice);
}

async function handleBrowseFansDir(): Promise<void> {
  await receiverConfigStore.browseFansOutputDirectory();
}

function handleClearError(): void {
  receiverConfigStore.clearError();
}

onMounted(() => {
  void receiverConfigStore.initialize();
});
</script>

<style scoped>
.receiver-config-page {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.receiver-config-page__metrics-row {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 16px;
}

.receiver-config-page__workspace {
  display: grid;
  grid-template-columns: minmax(0, 0.95fr) minmax(0, 1.05fr);
  gap: 20px;
  align-items: start;
}

.receiver-config-page__stack {
  display: grid;
  gap: 20px;
}

.receiver-config-page__metric-detail,
.receiver-config-page__text {
  margin-top: 8px;
  color: var(--app-text-secondary);
  font-size: 12px;
  line-height: 1.6;
  word-break: break-all;
}

.receiver-config-page__action-row {
  display: flex;
  gap: 8px;
  align-items: center;
  flex-wrap: wrap;
}

.receiver-config-page__switch-row {
  display: inline-flex;
  gap: 8px;
  align-items: center;
  color: var(--app-text-secondary);
  font-size: 12px;
  line-height: 1.6;
}

.receiver-config-page__path-bar {
  display: flex;
  gap: 12px;
  align-items: center;
}

.receiver-config-page__path-bar :deep(.el-input) {
  flex: 1 1 auto;
}

.receiver-config-page__field-hint {
  margin: 8px 0 0;
  color: var(--app-text-secondary);
  font-size: 12px;
  line-height: 1.6;
}

.receiver-config-page__code {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 11px;
  line-height: 1.7;
  color: var(--app-text-primary);
  word-break: break-all;
}

@media (max-width: 1180px) {
  .receiver-config-page__metrics-row {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .receiver-config-page__workspace {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 760px) {
  .receiver-config-page__metrics-row {
    grid-template-columns: 1fr;
  }

  .receiver-config-page__path-bar {
    flex-direction: column;
    align-items: stretch;
  }
}
</style>
