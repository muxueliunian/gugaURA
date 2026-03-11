<template>
  <el-tag
    class="status-badge"
    :type="type"
    effect="plain"
    round
    size="small"
  >
    {{ label }}
  </el-tag>
</template>

<script setup lang="ts">
import { ElTag } from 'element-plus/es/components/tag/index';

defineOptions({ name: 'StatusBadge' });

withDefaults(
  defineProps<{
    label: string;
    type?: 'danger' | 'info' | 'primary' | 'success' | 'warning';
  }>(),
  {
    type: 'info',
  },
);
</script>

<style scoped>
.status-badge {
  font-weight: 700;
  border-radius: 999px;
}
</style>
