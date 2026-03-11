<template>
  <div class="page-state">
    <div
      v-if="state === 'loading'"
      class="page-state__loading"
    >
      <el-skeleton
        animated
        :rows="3"
      />
      <p>{{ description }}</p>
    </div>

    <el-result
      v-else-if="state === 'error'"
      status="error"
      :title="title"
      :sub-title="description"
    >
      <template
        v-if="actionText"
        #extra
      >
        <el-button
          type="primary"
          @click="emit('action')"
        >
          {{ actionText }}
        </el-button>
      </template>
    </el-result>

    <el-empty
      v-else
      :description="description"
      :image-size="80"
    >
      <template #description>
        <div class="page-state__empty">
          <strong>{{ title }}</strong>
          <p>{{ description }}</p>
        </div>
      </template>

      <el-button
        v-if="actionText"
        type="primary"
        plain
        @click="emit('action')"
      >
        {{ actionText }}
      </el-button>
    </el-empty>
  </div>
</template>

<script setup lang="ts">
import { ElButton } from 'element-plus/es/components/button/index';
import { ElEmpty } from 'element-plus/es/components/empty/index';
import { ElResult } from 'element-plus/es/components/result/index';
import { ElSkeleton } from 'element-plus/es/components/skeleton/index';

defineOptions({ name: 'PageState' });

const emit = defineEmits<{
  action: [];
}>();

withDefaults(
  defineProps<{
    title: string;
    description: string;
    state?: 'empty' | 'error' | 'loading';
    actionText?: string;
  }>(),
  {
    state: 'empty',
  },
);
</script>

<style scoped>
.page-state {
  border: 1px solid var(--app-border-soft);
  border-radius: var(--app-radius-panel);
  background: var(--app-surface-subtle);
}

.page-state__loading {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 24px;
}

.page-state__loading p,
.page-state__empty p {
  margin: 0;
  color: var(--app-text-secondary);
  font-size: 13px;
}

.page-state__empty {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.page-state__empty strong {
  color: var(--app-text-primary);
  font-size: 14px;
  font-weight: 600;
}

.page-state :deep(.el-empty),
.page-state :deep(.el-result) {
  padding: 24px;
}

.page-state :deep(.el-empty__description p),
.page-state :deep(.el-result__subtitle p) {
  color: var(--app-text-secondary);
}
</style>
