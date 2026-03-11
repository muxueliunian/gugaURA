<template>
  <el-card
    class="info-card"
    shadow="never"
  >
    <div class="info-card__content">
      <p class="info-card__label">{{ label }}</p>
      <div class="info-card__value">
        <slot>
          {{ value ?? '--' }}
        </slot>
      </div>
      <p
        v-if="description"
        class="info-card__description"
      >
        {{ description }}
      </p>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { ElCard } from 'element-plus/es/components/card/index';

defineOptions({ name: 'InfoCard' });

defineProps<{
  label: string;
  value?: string | number;
  description?: string;
}>();
</script>

<style scoped>
.info-card {
  border: 1px solid var(--app-border-soft);
  border-radius: var(--app-radius-panel);
  background: var(--app-surface);
  box-shadow: var(--app-shadow-panel);
}

.info-card :deep(.el-card__body) {
  padding: 16px;
}

.info-card__content {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.info-card__label {
  margin: 0;
  color: var(--app-text-secondary);
  font-size: 12px;
  font-weight: 500;
}

.info-card__value {
  color: var(--app-text-primary);
  font-size: 24px;
  font-weight: 600;
  line-height: 1.2;
}

.info-card__description {
  margin: 4px 0 0;
  color: var(--app-text-tertiary);
  font-size: 12px;
  line-height: 1.4;
}
</style>
