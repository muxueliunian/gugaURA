<template>
  <FeaturePlaceholderPage
    eyebrow="日志输出"
    title="控制台"
    description="集中查看日志输出、事件记录与控制台视图。"
    :summary-items="summaryItems"
    :checklist="checklist"
    state-title="日志入口已整合"
    state-description="当前版本请使用终端页查看日志、刷新状态并清空记录。"
  />
</template>

<script setup lang="ts">
import FeaturePlaceholderPage from '@/components/layout/FeaturePlaceholderPage.vue';

defineOptions({ name: 'ConsolePage' });

const summaryItems = [
  {
    label: '本页定位',
    value: '输出中心',
    description: '日志、事件记录和运行反馈可集中查看。',
  },
  {
    label: '当前来源',
    value: 'Receiver 日志',
    description: '当前日志与状态信息由桌面端统一提供。',
  },
  {
    label: '当前目标',
    value: '统一查看',
    description: '日志信息保持在固定入口中，便于快速查看。',
  },
];

const checklist = [
  {
    key: 'logs',
    title: '日志输出视图',
    description: '日志列表、筛选和状态反馈适合集中展示。',
    status: 'planned' as const,
  },
  {
    key: 'events',
    title: '事件流与实时反馈',
    description: '实时输出与状态反馈可在统一入口中展示。',
    status: 'planned' as const,
  },
  {
    key: 'status',
    title: '状态与运行信息',
    description: '运行状态与日志入口已保持统一，便于集中查看。',
    status: 'ready' as const,
  },
];
</script>
