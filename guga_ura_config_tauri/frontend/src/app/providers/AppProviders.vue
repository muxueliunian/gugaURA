<template>
  <el-config-provider
    :button="buttonConfig"
    :locale="zhCn"
    :size="size"
    :z-index="zIndex"
  >
    <slot />
  </el-config-provider>
</template>

<script setup lang="ts">
import { ElConfigProvider } from 'element-plus/es/components/config-provider/index';
import zhCn from 'element-plus/es/locale/lang/zh-cn';

defineOptions({ name: 'AppProviders' });

const buttonConfig = {
  autoInsertSpace: false,
};
const size = 'default';
const zIndex = 3000;
</script>
