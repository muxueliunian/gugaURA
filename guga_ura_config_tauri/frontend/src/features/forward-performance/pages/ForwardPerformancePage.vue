<template>
  <FeaturePlaceholderPage
    eyebrow="转发设置"
    title="转发与性能"
    description="集中查看接收器端口、转发设置与性能参数。"
    :summary-items="summaryItems"
    :checklist="checklist"
    state-title="相关设置已集中管理"
    state-description="当前版本请使用 DLL 注入页和游戏设置页完成相关配置。"
  />
</template>

<script setup lang="ts">
import FeaturePlaceholderPage from '@/components/layout/FeaturePlaceholderPage.vue';

defineOptions({ name: 'ForwardPerformancePage' });

const summaryItems = [
  {
    label: '现有输入',
    value: 'Receiver 状态',
    description: '侧栏运行状态已经能提供当前接收器的基础信息。',
  },
  {
    label: '关键观察点',
    value: '端口与转发',
    description: '端口占用、转发策略与性能参数会集中展示。',
  },
  {
    label: '当前目标',
    value: '统一管理',
    description: '相关设置保持在固定入口中，便于集中处理。',
  },
];

const checklist = [
  {
    key: 'receiver',
    title: 'Receiver 与端口状态',
    description: '当前全局状态可作为端口与运行状态的基础信息。',
    status: 'ready' as const,
  },
  {
    key: 'forward',
    title: '转发开关与网络设置',
    description: '网络设置与保存反馈可在统一入口中集中展示。',
    status: 'planned' as const,
  },
  {
    key: 'performance',
    title: '性能参数与运行策略',
    description: '性能参数会与运行状态一起归拢到统一设置区域。',
    status: 'planned' as const,
  },
];
</script>
