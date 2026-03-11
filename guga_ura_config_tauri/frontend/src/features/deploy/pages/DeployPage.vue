<template>
  <FeaturePlaceholderPage
    eyebrow="安装部署"
    title="部署"
    description="统一处理目录识别、DLL 安装卸载与配置写入。"
    :summary-items="summaryItems"
    :checklist="checklist"
    state-title="部署能力已整合"
    state-description="当前版本可通过 DLL 注入页完成目录检测、安装和配置保存。"
  />
</template>

<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { computed } from 'vue';
import FeaturePlaceholderPage from '@/components/layout/FeaturePlaceholderPage.vue';
import { useDeployStore } from '@/stores/deploy';

defineOptions({ name: 'DeployPage' });

const deployStore = useDeployStore();
const { checklist, readyCount } = storeToRefs(deployStore);

const summaryItems = computed(() => [
  {
    label: '整合能力',
    value: checklist.value.length,
    description: '目录识别、安装状态和配置写入已集中管理。',
  },
  {
    label: '当前可用',
    value: readyCount.value,
    description: '当前目录检测结果可直接用于部署流程。',
  },
  {
    label: '当前目标',
    value: '统一入口',
    description: '部署相关操作已集中到稳定入口中。',
  },
]);
</script>
