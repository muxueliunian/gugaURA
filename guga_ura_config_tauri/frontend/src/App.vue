<template>
  <AppProviders>
    <AppShell />
  </AppProviders>
</template>

<script setup lang="ts">
import AppShell from '@/app/AppShell.vue';
import AppProviders from '@/app/providers/AppProviders.vue';

defineOptions({ name: 'RootApp' });
</script>
