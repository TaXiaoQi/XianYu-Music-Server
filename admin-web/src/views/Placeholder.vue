<template>
  <div class="placeholder-wrap">
    <div class="placeholder-card">
      <div class="placeholder-icon">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z" />
          <polyline points="14 2 14 8 20 8" />
          <line x1="9" y1="13" x2="15" y2="13" />
          <line x1="9" y1="17" x2="13" y2="17" />
        </svg>
      </div>
      <h2>{{ title }}</h2>
      <p>该页面正在重构中，敬请期待</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'

const route = useRoute()
const title = computed(() => (route.meta.title as string) || '页面')
</script>

<style scoped>
.placeholder-wrap {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 400px;
}
.placeholder-card {
  text-align: center;
  padding: 48px;
}
.placeholder-icon {
  color: var(--text-muted);
  margin-bottom: 16px;
}
.placeholder-card h2 {
  font-size: 20px;
  font-weight: 700;
  color: var(--text);
  margin-bottom: 8px;
}
.placeholder-card p {
  font-size: 14px;
  color: var(--text-muted);
}
</style>
