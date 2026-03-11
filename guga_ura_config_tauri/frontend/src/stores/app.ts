import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { APP_NAVIGATION_ITEMS } from '@/app/config/navigation';

export const useAppStore = defineStore('app', () => {
  const appTitle = ref('GugaURA');
  const sidebarCollapsed = ref(false);
  const navigationItems = computed(() => APP_NAVIGATION_ITEMS);

  function setSidebarCollapsed(collapsed: boolean): void {
    sidebarCollapsed.value = collapsed;
  }

  return {
    appTitle,
    navigationItems,
    setSidebarCollapsed,
    sidebarCollapsed,
  };
});
