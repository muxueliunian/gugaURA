<template>
  <div class="feature-placeholder-page app-page">
    <PageHeader
      :eyebrow="eyebrow"
      :title="title"
      :description="description"
    />

    <div class="feature-placeholder-page__layout">
      <SectionCard
        title="页面概览"
        description="集中展示页面定位、摘要信息与可用能力。"
      >
        <div class="feature-placeholder-page__summary app-metrics-grid">
          <InfoCard
            v-for="item in summaryItems"
            :key="item.label"
            :label="item.label"
            :value="item.value"
            :description="item.description"
          />
        </div>
      </SectionCard>

      <SectionCard
        title="能力清单"
        description="相关能力会集中保留在当前页面入口与组件结构内。"
      >
        <div class="feature-placeholder-page__checklist">
          <div
            v-for="item in checklist"
            :key="item.key"
            class="feature-placeholder-page__item"
          >
            <div>
              <strong>{{ item.title }}</strong>
              <p>{{ item.description }}</p>
            </div>
            <StatusBadge
              :label="item.status === 'ready' ? '已具备' : '待完善'"
              :type="item.status === 'ready' ? 'success' : 'warning'"
            />
          </div>
        </div>
      </SectionCard>
    </div>

    <PageState
      :title="stateTitle"
      :description="stateDescription"
    />
  </div>
</template>

<script setup lang="ts">
import SectionCard from '@/components/SectionCard.vue';
import StatusBadge from '@/components/StatusBadge.vue';
import InfoCard from '@/components/display/InfoCard.vue';
import PageState from '@/components/feedback/PageState.vue';
import PageHeader from '@/components/layout/PageHeader.vue';

defineOptions({ name: 'FeaturePlaceholderPage' });

interface PlaceholderSummaryItem {
  label: string;
  value: number | string;
  description: string;
}

interface PlaceholderChecklistItem {
  key: string;
  title: string;
  description: string;
  status: 'planned' | 'ready';
}

defineProps<{
  eyebrow: string;
  title: string;
  description: string;
  summaryItems: PlaceholderSummaryItem[];
  checklist: PlaceholderChecklistItem[];
  stateTitle: string;
  stateDescription: string;
}>();
</script>

<style scoped>
.feature-placeholder-page__layout {
  display: grid;
  gap: 20px;
}

.feature-placeholder-page__summary {
  grid-template-columns: repeat(3, minmax(0, 1fr));
}

.feature-placeholder-page__checklist {
  display: grid;
  gap: 12px;
}

.feature-placeholder-page__item {
  display: flex;
  gap: 16px;
  align-items: flex-start;
  justify-content: space-between;
  padding: 14px 16px;
  border: 1px solid var(--app-border-soft);
  border-radius: 16px;
  background: rgba(246, 249, 253, 0.86);
}

.feature-placeholder-page__item strong {
  display: block;
  margin-bottom: 6px;
  color: var(--app-text-primary);
  font-size: 15px;
}

.feature-placeholder-page__item p {
  margin: 0;
  color: var(--app-text-secondary);
  font-size: 13px;
  line-height: 1.6;
}

@media (max-width: 900px) {
  .feature-placeholder-page__summary {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 760px) {
  .feature-placeholder-page__item {
    flex-direction: column;
  }
}
</style>
