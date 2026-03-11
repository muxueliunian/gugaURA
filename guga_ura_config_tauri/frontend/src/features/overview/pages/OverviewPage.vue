<template>
  <div class="overview-page app-page">
    <PageHeader
      eyebrow="运行概况"
      title="总览"
      description="集中查看 Receiver 状态、当前注入上下文和 Debug 开关。"
    >
      <template #actions>
        <el-button
          type="primary"
          :loading="receiverLoading || contextLoading"
          @click="handleRefresh"
        >
          刷新状态
        </el-button>
      </template>
    </PageHeader>

    <el-alert
      v-if="pageError"
      class="app-page__alert"
      type="error"
      :closable="true"
      show-icon
      title="状态读取失败"
      :description="pageError"
      @close="handleClearError"
    />

    <div class="overview-page__metrics-row">
      <InfoCard
        label="应用版本"
        :value="appState?.appVersion ?? '--'"
      />
      <InfoCard label="Receiver 状态">
        <StatusBadge
          :label="resolveReceiverStatusLabel(appState)"
          :type="resolveReceiverStatusType(appState?.receiverReady)"
        />
        <div class="overview-page__metric-detail">{{ receiverSummary }}</div>
      </InfoCard>
      <InfoCard label="当前注入上下文">
        <StatusBadge
          :label="injectionContextLabel"
          :type="injectionContextType"
        />
        <div class="overview-page__metric-detail">{{ injectionContextDetail }}</div>
      </InfoCard>
    </div>

    <el-alert
      class="overview-page__next-step"
      :type="nextStep.type"
      :closable="false"
      show-icon
    >
      <template #title>
        <strong>{{ nextStep.title }}</strong>
      </template>
      <p style="margin: 4px 0 0 0">{{ nextStep.description }}</p>
    </el-alert>

    <div class="overview-page__workspace">
      <SectionCard
        title="当前注入状态"
        description="显示当前目录、版本和安装状态；完整检测与配置请在 DLL 注入页完成。"
      >
        <el-descriptions
          v-if="context"
          :column="1"
          border
          size="small"
        >
          <el-descriptions-item label="游戏目录">
            <span class="overview-page__code">{{ context.path || '尚未选择' }}</span>
          </el-descriptions-item>
          <el-descriptions-item label="目录状态">
            <StatusBadge
              :label="context.isValidGameDir ? '已确认' : '待确认'"
              :type="context.isValidGameDir ? 'success' : 'warning'"
            />
          </el-descriptions-item>
          <el-descriptions-item label="识别版本">
            <StatusBadge
              :label="context.detectedVersionLabel"
              :type="resolveGameVersionType(context.detectedVersion)"
            />
          </el-descriptions-item>
          <el-descriptions-item label="安装状态">
            <StatusBadge
              :label="context.installStatusLabel"
              :type="resolveInstallStatusType(context.installStatus)"
            />
          </el-descriptions-item>
          <el-descriptions-item label="Notifier 地址">
            <span class="overview-page__code">{{ context.notifierHost }}</span>
          </el-descriptions-item>
          <el-descriptions-item label="Fans 聚合">
            <StatusBadge
              :label="context?.fansEnabled ? '已开启' : '已关闭'"
              :type="context?.fansEnabled ? 'success' : 'warning'"
            />
          </el-descriptions-item>
        </el-descriptions>

        <PageState
          v-else
          state="loading"
          title="正在读取注入上下文"
          description="等待桌面端返回当前 DLL 注入配置。"
        />

        <div class="overview-page__section-actions">
          <el-button
            type="primary"
            plain
            @click="handleGoToDllInjection"
          >
            前往 DLL 注入页
          </el-button>
        </div>
      </SectionCard>

      <SectionCard
        title="Debug 模式"
        description="提供 Debug 模式的快速开关与输出目录信息。"
      >
        <div class="overview-page__debug-panel">
          <div class="overview-page__debug-copy">
            <strong>Debug 开关</strong>
            <p>开启后，payload 会解码并写入本地 debug 目录；切换后通过独立入口立即保存。</p>
          </div>

          <el-switch
            :model-value="context?.debugMode ?? false"
            :loading="actionLoading"
            :disabled="!hasValidGameContext"
            inline-prompt
            active-text="开"
            inactive-text="关"
            @change="handleToggleDebug"
          />
        </div>

        <el-descriptions
          :column="1"
          border
          size="small"
        >
          <el-descriptions-item label="当前状态">
            <StatusBadge
              :label="context?.debugMode ? '已开启' : '已关闭'"
              :type="context?.debugMode ? 'success' : 'info'"
            />
          </el-descriptions-item>
          <el-descriptions-item label="输出目录">
            <span class="overview-page__code">{{ context?.debugOutputDir ?? '--' }}</span>
          </el-descriptions-item>
          <el-descriptions-item label="同步方式">
            <span class="overview-page__text">保存后会同时写回游戏目录与 EXE 同级目录。</span>
          </el-descriptions-item>
        </el-descriptions>

        <p class="overview-page__debug-hint">
          {{
            hasValidGameContext
              ? '如需检测目录、安装/卸载 DLL 或修改注入链路配置，请进入 DLL 注入页处理。'
              : '请先在 DLL 注入页选择并检测有效游戏目录，再切换 Debug 模式。'
          }}
        </p>
      </SectionCard>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ElAlert } from 'element-plus/es/components/alert/index';
import { ElButton } from 'element-plus/es/components/button/index';
import { ElDescriptions, ElDescriptionsItem } from 'element-plus/es/components/descriptions/index';
import { ElMessage } from 'element-plus/es/components/message/index';
import { ElSwitch } from 'element-plus/es/components/switch/index';
import { storeToRefs } from 'pinia';
import { computed, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import SectionCard from '@/components/SectionCard.vue';
import StatusBadge from '@/components/StatusBadge.vue';
import InfoCard from '@/components/display/InfoCard.vue';
import PageState from '@/components/feedback/PageState.vue';
import PageHeader from '@/components/layout/PageHeader.vue';
import {
  resolveGameVersionType,
  resolveInstallStatusType,
  resolveReceiverStatusLabel,
  resolveReceiverStatusType,
} from '@/app/utils/status';
import { useDllInjectionStore } from '@/stores/dllInjection';
import { useReceiverStore } from '@/stores/receiver';

defineOptions({ name: 'OverviewPage' });

const router = useRouter();
const receiverStore = useReceiverStore();
const dllInjectionStore = useDllInjectionStore();
const { appState, error: receiverError, loading: receiverLoading } = storeToRefs(receiverStore);
const {
  actionLoading,
  context,
  contextLoading,
  form,
  hasValidGameContext,
  lastError,
} = storeToRefs(dllInjectionStore);

const pageError = computed(() => lastError.value || receiverError.value);
const receiverSummary = computed(() => {
  const base = receiverError.value || appState.value?.receiverStatus || '正在读取桌面状态';
  const listenAddr = appState.value?.receiverListenAddr;
  return listenAddr ? `${base}；当前监听：${listenAddr}` : base;
});
const injectionContextLabel = computed(() => {
  if (!context.value?.path) {
    return '待确认';
  }
  return context.value.isValidGameDir ? '已就绪' : '待修正';
});
const injectionContextType = computed(() => {
  if (!context.value?.path) {
    return 'info';
  }
  return context.value.isValidGameDir ? 'success' : 'warning';
});
const injectionContextDetail = computed(() => {
  if (!context.value?.path) {
    return '尚未选择游戏目录';
  }
  if (!context.value.isValidGameDir) {
    return '当前路径无效，请前往 DLL 注入页重新检测';
  }
  return context.value.path;
});
const nextStep = computed(() => {
  if (receiverError.value || !appState.value?.receiverReady) {
    return {
      title: '先确认 Receiver 状态',
      description: 'Receiver 未就绪时，先排查端口占用、启动失败或桌面环境异常。',
      type: 'warning' as const,
    };
  }

  if (!hasValidGameContext.value) {
    return {
      title: '进入 DLL 注入页确认游戏目录',
      description: '请先在 DLL 注入页确认游戏目录、版本和当前安装状态。',
      type: 'info' as const,
    };
  }

  if (context.value?.installStatus !== 'installed') {
    return {
      title: '继续完成 DLL 安装',
      description: '当前目录已经有效，下一步可在 DLL 注入页执行安装或保存注入链路配置。',
      type: 'warning' as const,
    };
  }

  return {
    title: '当前环境已就绪',
    description: '可在本页快速查看状态，并前往终端或游戏设置页继续处理其他配置。',
    type: 'success' as const,
  };
});

async function handleRefresh(): Promise<void> {
  await Promise.allSettled([receiverStore.loadState(true), dllInjectionStore.refreshCurrentContext()]);
}

async function handleToggleDebug(value: string | number | boolean): Promise<void> {
  if (typeof value !== 'boolean') {
    return;
  }

  const result = await dllInjectionStore.saveDebugMode(value);
  if (!result) {
    return;
  }

  ElMessage.success(result.notice);
}

function handleGoToDllInjection(): void {
  void router.push('/dll-injection');
}

function handleClearError(): void {
  receiverStore.clearError();
  dllInjectionStore.clearError();
}

onMounted(() => {
  void Promise.allSettled([receiverStore.loadState(), dllInjectionStore.initialize()]);
});
</script>

<style scoped>
.overview-page {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.overview-page__metrics-row {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 16px;
}

.overview-page__metric-detail {
  margin-top: 8px;
  font-size: 12px;
  color: var(--app-text-secondary);
  line-height: 1.5;
  word-break: break-all;
}

.overview-page__next-step {
  border-radius: var(--app-radius-panel);
}

.overview-page__next-step strong {
  color: var(--app-text-primary);
}

.overview-page__workspace {
  display: grid;
  grid-template-columns: minmax(0, 1.15fr) minmax(0, 0.85fr);
  gap: 20px;
  align-items: start;
}

.overview-page__section-actions {
  display: flex;
  justify-content: flex-start;
  margin-top: 16px;
}

.overview-page__debug-panel {
  display: flex;
  gap: 20px;
  align-items: flex-start;
  justify-content: space-between;
  margin-bottom: 16px;
}

.overview-page__debug-copy {
  display: grid;
  gap: 6px;
  max-width: 420px;
}

.overview-page__debug-copy strong {
  color: var(--app-text-primary);
  font-size: 14px;
}

.overview-page__debug-copy p,
.overview-page__debug-hint,
.overview-page__text {
  margin: 0;
  color: var(--app-text-secondary);
  font-size: 13px;
  line-height: 1.6;
}

.overview-page__debug-hint {
  margin-top: 16px;
}

.overview-page__code {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 11px;
  word-break: break-all;
}

@media (max-width: 1100px) {
  .overview-page__workspace {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 760px) {
  .overview-page__metrics-row {
    grid-template-columns: 1fr;
  }

  .overview-page__debug-panel {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
