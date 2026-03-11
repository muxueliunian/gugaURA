<template>
  <div class="terminal-page app-page">
    <PageHeader
      eyebrow="接收器日志"
      title="终端"
      description="查看 Receiver 当前状态、日志快照与排障信息；监听和转发参数请在接收&转发配置页处理。"
    >
      <template #actions>
        <el-button
          :loading="loading"
          @click="handleRefresh"
        >
          立即刷新
        </el-button>
      </template>
    </PageHeader>

    <el-alert
      v-if="pageError"
      class="app-page__alert"
      type="error"
      :closable="true"
      show-icon
      title="终端页操作失败"
      :description="pageError"
      @close="handleClearError"
    />

    <div class="terminal-page__metrics-row">
      <InfoCard label="Receiver 状态">
        <StatusBadge
          :label="receiverReady ? '已就绪' : '未就绪'"
          :type="resolveReceiverStatusType(receiverReady)"
        />
        <div class="terminal-page__metric-detail">{{ receiverMetricDetail }}</div>
      </InfoCard>
      <InfoCard
        label="当前监听地址"
        :value="receiverListenAddr"
        :description="`来源：${receiverListenAddrSourceLabel}`"
      />
      <InfoCard
        label="启动配置值"
        :value="runtimeConfiguredListenAddr"
        description="展示当前运行实例启动时采用的配置值。"
      />
      <InfoCard
        label="日志快照"
        :value="logCount"
        description="单次最多保留并读取 600 行最新日志。"
      />
    </div>

    <SectionCard
      title="Receiver 当前状态"
      description="这里只展示当前进程状态、地址来源和刷新时间；监听、Relay 与 Fans 设置请前往接收&转发配置页。"
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
          <span class="terminal-page__code">{{ receiverListenAddr }}</span>
        </el-descriptions-item>
        <el-descriptions-item label="启动时配置值">
          <span class="terminal-page__code">{{ runtimeConfiguredListenAddr }}</span>
        </el-descriptions-item>
        <el-descriptions-item label="当前配置来源">
          <span class="terminal-page__text">{{ receiverListenAddrSourceLabel }}</span>
        </el-descriptions-item>
        <el-descriptions-item label="状态文案">
          <span class="terminal-page__code">{{ receiverStatus }}</span>
        </el-descriptions-item>
        <el-descriptions-item label="最近刷新">
          <span class="terminal-page__text">{{ lastUpdatedLabel }}</span>
        </el-descriptions-item>
        <el-descriptions-item label="刷新机制">
          <span class="terminal-page__text">页面驻留时每 1.5 秒串行轮询一次，可随时手动刷新。</span>
        </el-descriptions-item>
      </el-descriptions>
    </SectionCard>

    <SectionCard
      title="日志快照"
      description="日志区支持滚动查看、手动刷新和清空；可按需隐藏 payload 保存成功日志。"
    >
      <template #header-extra>
        <div class="terminal-page__action-row">
          <div class="terminal-page__switch-row">
            <span>显示 payload 保存成功日志</span>
            <el-switch
              v-model="showPayloadSaveLogs"
              inline-prompt
              active-text="开"
              inactive-text="关"
            />
          </div>
          <el-button
            text
            size="small"
            :loading="loading"
            @click="handleRefresh"
          >
            刷新
          </el-button>
          <el-button
            text
            size="small"
            type="danger"
            :loading="clearing"
            @click="handleClearLogs"
          >
            清空日志
          </el-button>
        </div>
      </template>

      <PageState
        v-if="loading && logCount === 0"
        state="loading"
        title="正在读取日志快照"
        description="等待桌面端返回 Receiver 当前日志。"
      />

      <PageState
        v-else-if="pageError && logCount === 0"
        state="error"
        title="日志快照读取失败"
        :description="pageError"
        action-text="重试刷新"
        @action="handleRefresh"
      />

      <PageState
        v-else-if="!receiverReady && logCount === 0"
        title="Receiver 未就绪"
        :description="receiverUnavailableDescription"
        action-text="立即刷新"
        @action="handleRefresh"
      />

      <PageState
        v-else-if="logCount === 0"
        title="当前没有日志"
        :description="emptyLogsDescription"
        action-text="立即刷新"
        @action="handleRefresh"
      />

      <PageState
        v-else-if="visibleLogCount === 0"
        title="当前日志已被过滤"
        description="当前快照中只有“保存 payload 成功”类日志，打开开关后即可查看。"
        action-text="显示这类日志"
        @action="handleShowPayloadLogs"
      />

      <template v-else>
        <div class="terminal-page__log-toolbar">
          <span>显示 {{ visibleLogCount }} / {{ logCount }} 条日志</span>
          <span v-if="hiddenPayloadSaveLogCount > 0">
            已隐藏 {{ hiddenPayloadSaveLogCount }} 条 payload 保存成功日志
          </span>
          <span>最近刷新：{{ lastUpdatedLabel }}</span>
        </div>

        <el-scrollbar
          height="420px"
          class="terminal-page__log-scroll"
        >
          <div class="terminal-page__log-list">
            <div
              v-for="(line, index) in visibleLogs"
              :key="`${index}-${line}`"
              class="terminal-page__log-line"
            >
              {{ line }}
            </div>
          </div>
        </el-scrollbar>
      </template>
    </SectionCard>
  </div>
</template>

<script setup lang="ts">
import { ElAlert } from 'element-plus/es/components/alert/index';
import { ElButton } from 'element-plus/es/components/button/index';
import { ElDescriptions, ElDescriptionsItem } from 'element-plus/es/components/descriptions/index';
import { ElMessage } from 'element-plus/es/components/message/index';
import { ElScrollbar } from 'element-plus/es/components/scrollbar/index';
import { ElSwitch } from 'element-plus/es/components/switch/index';
import { storeToRefs } from 'pinia';
import { computed, onMounted, onUnmounted } from 'vue';
import SectionCard from '@/components/SectionCard.vue';
import StatusBadge from '@/components/StatusBadge.vue';
import InfoCard from '@/components/display/InfoCard.vue';
import PageState from '@/components/feedback/PageState.vue';
import PageHeader from '@/components/layout/PageHeader.vue';
import {
  resolveReceiverListenAddrSourceLabel,
  resolveReceiverStatusType,
} from '@/app/utils/status';
import { useTerminalStore } from '@/stores/terminal';

defineOptions({ name: 'TerminalPage' });

const terminalStore = useTerminalStore();
const {
  clearing,
  hiddenPayloadSaveLogCount,
  lastError,
  lastUpdatedAt,
  loading,
  logCount,
  receiverListenAddr,
  receiverListenAddrSource,
  receiverReady,
  receiverStatus,
  runtimeConfiguredListenAddr,
  showPayloadSaveLogs,
  visibleLogCount,
  visibleLogs,
} = storeToRefs(terminalStore);

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
const receiverUnavailableDescription = computed(
  () => `当前状态：${receiverStatus.value}。请先确认桌面端是否启动成功，页面仍会继续轮询刷新。`,
);
const emptyLogsDescription = computed(() => {
  if (lastUpdatedAt.value) {
    return '日志已清空或当前尚未收到新日志，页面仍会继续轮询刷新。';
  }

  return 'Receiver 尚未收到新日志，页面仍会继续轮询刷新。';
});
const lastUpdatedLabel = computed(() => {
  if (!lastUpdatedAt.value) {
    return '尚未刷新';
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

async function handleRefresh(): Promise<void> {
  await terminalStore.refreshSnapshot(true);
}

async function handleClearLogs(): Promise<void> {
  const cleared = await terminalStore.clearLogs();
  if (!cleared) {
    return;
  }

  ElMessage.success('已清空接收器日志');
}

function handleClearError(): void {
  terminalStore.clearError();
}

function handleShowPayloadLogs(): void {
  showPayloadSaveLogs.value = true;
}

let isPageMounted = false;

onMounted(() => {
  isPageMounted = true;
  void terminalStore.initialize().finally(() => {
    if (isPageMounted) {
      terminalStore.startPolling();
    }
  });
});

onUnmounted(() => {
  isPageMounted = false;
  terminalStore.stopPolling();
});
</script>

<style scoped>
.terminal-page {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.terminal-page__metrics-row {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 16px;
}

.terminal-page__metric-detail,
.terminal-page__text {
  margin-top: 8px;
  color: var(--app-text-secondary);
  font-size: 12px;
  line-height: 1.6;
  word-break: break-all;
}

.terminal-page__action-row {
  display: flex;
  gap: 8px;
  align-items: center;
  flex-wrap: wrap;
}

.terminal-page__switch-row {
  display: inline-flex;
  gap: 8px;
  align-items: center;
  color: var(--app-text-secondary);
  font-size: 12px;
  line-height: 1.6;
}

.terminal-page__log-toolbar {
  display: flex;
  justify-content: flex-start;
  gap: 12px;
  flex-wrap: wrap;
  margin-bottom: 12px;
  color: var(--app-text-secondary);
  font-size: 12px;
  line-height: 1.5;
}

.terminal-page__log-scroll {
  border: 1px solid var(--app-border-soft);
  border-radius: 12px;
  background: var(--app-surface-subtle);
}

.terminal-page__log-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 16px;
}

.terminal-page__log-line,
.terminal-page__code {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 11px;
  line-height: 1.7;
  word-break: break-all;
}

.terminal-page__log-line {
  color: var(--app-text-secondary);
}

.terminal-page__code {
  color: var(--app-text-primary);
}

@media (max-width: 1180px) {
  .terminal-page__metrics-row {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 760px) {
  .terminal-page__metrics-row {
    grid-template-columns: 1fr;
  }

  .terminal-page__log-toolbar {
    flex-direction: column;
    align-items: stretch;
  }
}
</style>
