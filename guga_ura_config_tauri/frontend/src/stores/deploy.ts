import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import type { DeployChecklistItem } from '@/features/deploy/types';

export const useDeployStore = defineStore('deploy', () => {
  const checklist = ref<DeployChecklistItem[]>([
    {
      key: 'detector',
      title: '目录识别与版本确认',
      description: '当前目录检测结果可直接用于部署流程。',
      status: 'ready',
    },
    {
      key: 'installer',
      title: 'DLL 安装与卸载流程',
      description: '安装与卸载操作可集中在统一入口中处理。',
      status: 'planned',
    },
    {
      key: 'config',
      title: '配置读写与保存反馈',
      description: '配置保存结果会统一反馈到当前入口。',
      status: 'planned',
    },
  ]);

  const readyCount = computed(() => checklist.value.filter((item) => item.status === 'ready').length);

  return {
    checklist,
    readyCount,
  };
});
