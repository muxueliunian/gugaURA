import { fileURLToPath, URL } from 'node:url';
import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (!id.includes('node_modules')) {
            return;
          }

          if (id.includes('element-plus')) {
            return 'element-plus';
          }

          if (id.includes('@tauri-apps')) {
            return 'tauri';
          }

          if (id.includes('pinia')) {
            return 'pinia';
          }

          if (id.includes('vue-router')) {
            return 'vue-router';
          }

          if (id.includes('/vue/') || id.includes('@vue')) {
            return 'vue';
          }

          return 'vendor';
        },
      },
    },
  },
  server: {
    host: '127.0.0.1',
    port: 1420,
    strictPort: true,
  },
});
