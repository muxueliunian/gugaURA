<template>
  <header class="page-header">
    <div class="page-header__content">
      <div class="page-header__title-row">
        <p
          v-if="eyebrow"
          class="page-header__eyebrow"
        >
          {{ eyebrow }}
        </p>
        <h1>{{ title }}</h1>
      </div>
      <p
        v-if="description"
        class="page-header__description"
      >
        {{ description }}
      </p>
    </div>

    <div
      v-if="$slots.actions"
      class="page-header__actions"
    >
      <slot name="actions" />
    </div>
  </header>
</template>

<script setup lang="ts">
defineOptions({ name: 'PageHeader' });

defineProps<{
  title: string;
  description?: string;
  eyebrow?: string;
}>();
</script>

<style scoped>
.page-header {
  display: flex;
  gap: 20px;
  align-items: flex-start;
  justify-content: space-between;
  margin-bottom: 8px;
}

.page-header__content {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}

.page-header__title-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.page-header__eyebrow {
  display: inline-flex;
  align-items: center;
  padding: 2px 8px;
  border-radius: 999px;
  background: var(--app-primary-soft);
  color: var(--app-primary-strong);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.04em;
  margin: 0;
}

.page-header h1 {
  margin: 0;
  color: var(--app-text-primary);
  font-size: 24px;
  font-weight: 600;
  letter-spacing: -0.01em;
  line-height: 1.2;
}

.page-header__description {
  margin: 0;
  color: var(--app-text-secondary);
  font-size: 13px;
  line-height: 1.6;
}

.page-header__actions {
  display: flex;
  gap: 12px;
  align-items: center;
  justify-content: flex-end;
  flex-shrink: 0;
}

@media (max-width: 760px) {
  .page-header {
    flex-direction: column;
    align-items: flex-start;
  }

  .page-header__actions {
    width: 100%;
    justify-content: flex-start;
  }
}
</style>
