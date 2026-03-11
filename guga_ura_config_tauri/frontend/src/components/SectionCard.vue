<template>
  <el-card class="section-card" shadow="never">
    <template #header>
      <div class="section-card__header">
        <div class="section-card__title-area">
          <h2 class="section-card__title">{{ title }}</h2>
          <p
            v-if="description"
            class="section-card__description"
          >
            {{ description }}
          </p>
        </div>
        <slot name="header-extra"></slot>
      </div>
    </template>

    <slot />
  </el-card>
</template>

<script setup lang="ts">
import { ElCard } from 'element-plus/es/components/card/index';

defineOptions({ name: 'SectionCard' });

defineProps<{
  title: string;
  description?: string;
}>();
</script>

<style scoped>
.section-card {
  border: 1px solid var(--app-border-soft);
  border-radius: var(--app-radius-panel);
  background: var(--app-surface);
  box-shadow: var(--app-shadow-panel);
}

.section-card :deep(.el-card__header) {
  padding: 16px 20px;
  border-bottom: 1px solid var(--app-border-soft);
}

.section-card :deep(.el-card__body) {
  padding: 20px;
}

.section-card__header {
  display: flex;
  gap: 16px;
  align-items: center;
  justify-content: space-between;
}

.section-card__title-area {
  display: flex;
  flex-direction: column;
}

.section-card__title {
  margin: 0;
  color: var(--app-text-primary);
  font-size: 15px;
  font-weight: 600;
  line-height: 1.4;
}

.section-card__description {
  margin: 4px 0 0;
  color: var(--app-text-secondary);
  font-size: 12px;
  line-height: 1.5;
}

@media (max-width: 760px) {
  .section-card :deep(.el-card__header),
  .section-card :deep(.el-card__body) {
    padding: 16px;
  }

  .section-card__header {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
