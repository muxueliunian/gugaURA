<template>
  <div class="app-shell">
    <aside class="app-shell__sidebar">
      <AppSidebar />
    </aside>

    <main class="app-shell__main">
      <div class="app-shell__scrollable">
        <RouterView />
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { RouterView } from 'vue-router';
import AppSidebar from '@/app/components/AppSidebar.vue';
import { useReceiverStore } from '@/stores/receiver';

defineOptions({ name: 'AppShell' });

const receiverStore = useReceiverStore();

onMounted(() => {
  void receiverStore.loadState();
});
</script>

<style scoped>
.app-shell {
  display: flex;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
  background: var(--app-bg);
}

.app-shell__sidebar {
  width: 240px;
  flex-shrink: 0;
  border-right: 1px solid var(--app-sidebar-border);
  background: var(--app-sidebar-bg);
  display: flex;
  flex-direction: column;
}

.app-shell__main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  position: relative;
}

.app-shell__scrollable {
  flex: 1;
  overflow-y: auto;
  padding: 24px 32px;
}

@media (max-width: 900px) {
  .app-shell__sidebar {
    width: 200px;
  }
  .app-shell__scrollable {
    padding: 16px 20px;
  }
}
</style>
