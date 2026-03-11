<template>
  <section class="app-sidebar">
    <div class="app-sidebar__brand">
      <div class="app-sidebar__title-row">
        <h1>{{ appTitle }}</h1>
        <span class="app-sidebar__platform">Windows</span>
      </div>
    </div>

    <div class="app-sidebar__nav">
      <el-menu
        class="app-sidebar__menu"
        :default-active="activePath"
        :router="true"
      >
        <el-menu-item
          v-for="item in navigationItems"
          :key="item.path"
          :index="item.path"
        >
          <div class="app-sidebar__menu-item">
            <div class="app-sidebar__menu-main">
              <span>{{ item.title }}</span>
              <small
                v-if="item.badge"
                class="app-sidebar__menu-badge"
              >
                {{ item.badge }}
              </small>
            </div>
          </div>
        </el-menu-item>
      </el-menu>
    </div>

    <AppStatusBar />
  </section>
</template>

<script setup lang="ts">
import { ElMenu, ElMenuItem } from 'element-plus/es/components/menu/index';
import { storeToRefs } from 'pinia';
import { computed } from 'vue';
import { useRoute } from 'vue-router';
import AppStatusBar from '@/app/components/AppStatusBar.vue';
import { useAppStore } from '@/stores/app';

defineOptions({ name: 'AppSidebar' });

const route = useRoute();
const appStore = useAppStore();
const { appTitle, navigationItems } = storeToRefs(appStore);

const activePath = computed(() => route.path);
</script>

<style scoped>
.app-sidebar {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.app-sidebar__brand {
  padding: 24px 16px 16px;
}

.app-sidebar__title-row {
  display: flex;
  gap: 8px;
  align-items: center;
}

.app-sidebar__title-row h1 {
  margin: 0;
  color: var(--app-text-primary);
  font-size: 16px;
  font-weight: 600;
  letter-spacing: -0.01em;
}

.app-sidebar__platform {
  display: inline-flex;
  align-items: center;
  padding: 2px 6px;
  border-radius: 4px;
  background: var(--app-surface-subtle);
  border: 1px solid var(--app-border-soft);
  color: var(--app-text-secondary);
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
}

.app-sidebar__nav {
  flex: 1;
  padding: 0 12px;
  overflow-y: auto;
}

.app-sidebar__menu {
  border: none;
  background: transparent;
}

.app-sidebar__menu :deep(.el-menu-item) {
  height: auto;
  margin: 0 0 4px;
  padding: 0 !important;
  border-radius: 6px;
  background: transparent;
}

.app-sidebar__menu-item {
  width: 100%;
  padding: 8px 12px;
  border-radius: 6px;
  color: var(--app-text-secondary);
  transition: all 150ms ease;
}

.app-sidebar__menu-main {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.app-sidebar__menu-main span {
  font-size: 13px;
  font-weight: 500;
}

.app-sidebar__menu-badge {
  display: inline-flex;
  align-items: center;
  padding: 2px 6px;
  border-radius: 999px;
  background: var(--app-surface-subtle);
  border: 1px solid var(--app-border-soft);
  color: var(--app-text-secondary);
  font-size: 10px;
  font-weight: 700;
}

.app-sidebar__menu :deep(.el-menu-item:hover .app-sidebar__menu-item) {
  background: var(--app-bg-muted);
  color: var(--app-text-primary);
}

.app-sidebar__menu :deep(.el-menu-item.is-active .app-sidebar__menu-item) {
  background: var(--app-bg);
  color: var(--app-text-primary);
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
  border: 1px solid var(--app-border-soft);
  font-weight: 600;
}

.app-sidebar__menu :deep(.el-menu-item.is-active .app-sidebar__menu-badge) {
  background: var(--app-surface-strong);
}
</style>
