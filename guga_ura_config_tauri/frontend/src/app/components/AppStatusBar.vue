<template>
  <section class="app-status-bar">
    <div class="app-status-bar__header">
      <div class="app-status-bar__context">
        <p class="app-status-bar__title">运行状态</p>
      </div>
      <StatusBadge
        :label="receiverLabel"
        :type="receiverType"
      />
    </div>

    <div class="app-status-bar__row">
      <span>应用版本</span>
      <strong>{{ appState?.appVersion ?? '--' }}</strong>
    </div>

    <p
      class="app-status-bar__detail"
      :class="{ 'app-status-bar__detail--danger': hasError }"
    >
      {{ receiverDetail }}
    </p>
  </section>
</template>

<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { computed } from 'vue';
import StatusBadge from '@/components/StatusBadge.vue';
import {
  resolveReceiverStatusLabel,
  resolveReceiverStatusType,
} from '@/app/utils/status';
import { useReceiverStore } from '@/stores/receiver';

defineOptions({ name: 'AppStatusBar' });

const receiverStore = useReceiverStore();
const { appState, error } = storeToRefs(receiverStore);

const receiverLabel = computed(() => resolveReceiverStatusLabel(appState.value));
const receiverType = computed(() => resolveReceiverStatusType(appState.value?.receiverReady));
const hasError = computed(() => Boolean(error.value));
const receiverDetail = computed(() => {
  const baseDetail = error.value || appState.value?.receiverStatus || '正在读取桌面状态';
  const listenAddr = appState.value?.receiverListenAddr;
  const detail = listenAddr ? `${baseDetail}\n当前监听：${listenAddr}` : baseDetail;

  if (detail.includes('，输出 ')) {
    return detail.replace('，输出 ', '，\n输出 ');
  }

  return detail;
});
</script>

<style scoped>
.app-status-bar {
  display: grid;
  gap: 12px;
  padding: 16px;
  border-top: 1px solid var(--app-sidebar-border);
  background: var(--app-sidebar-bg);
  margin-top: auto;
}

.app-status-bar__header {
  display: flex;
  gap: 12px;
  align-items: center;
  justify-content: space-between;
}

.app-status-bar__context {
  display: flex;
  flex-direction: column;
}

.app-status-bar__title {
  margin: 0;
  color: var(--app-text-primary);
  font-size: 13px;
  font-weight: 600;
}

.app-status-bar__row {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  gap: 12px;
  align-items: center;
}

.app-status-bar__row span {
  min-width: 0;
  color: var(--app-text-secondary);
  font-size: 12px;
}

.app-status-bar__row strong {
  min-width: 0;
  color: var(--app-text-primary);
  font-size: 12px;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  text-align: right;
  overflow-wrap: anywhere;
  word-break: break-all;
}

.app-status-bar__detail {
  margin: 0;
  color: var(--app-text-secondary);
  font-size: 11px;
  line-height: 1.5;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  word-break: break-word;
}

.app-status-bar__detail--danger {
  color: var(--app-danger);
}
</style>
