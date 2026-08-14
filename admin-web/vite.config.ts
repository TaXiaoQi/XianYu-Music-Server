import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { fileURLToPath, URL } from 'node:url'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  build: {
    emptyOutDir: false,
  },
  server: {
    port: 3000,
    proxy: {
      '/admin/api': {
        target: 'http://127.0.0.1:8081',
        changeOrigin: true,
      },
      '/api': {
        target: 'http://127.0.0.1:8081',
        changeOrigin: true,
      },
      '/uploads': {
        target: 'http://127.0.0.1:8081',
        changeOrigin: true,
      },
    },
  },
})
