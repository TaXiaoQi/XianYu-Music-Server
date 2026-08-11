<template>
  <div class="mobile-page">
    <div class="mobile-card mobile-form">
      <h3 class="mobile-card-title">接口快速测试</h3>
      <input v-model="action" class="mobile-input" placeholder="后台 action，例如 dashboard_stats" />
      <textarea v-model="body" class="mobile-textarea" placeholder='JSON 参数，例如 {"page":1}'></textarea>
      <button class="mobile-btn primary" :disabled="running" @click="run">发送测试</button>
    </div>
    <transition name="expand">
      <pre v-if="result" class="mobile-code">{{ result }}</pre>
    </transition>
  </div>
</template>
<script setup lang="ts">
import { ref } from 'vue'
import { adminApi, showToast } from '@/api/client'
import './MobilePage.css'
const action = ref('dashboard_stats'), body = ref('{}'), result = ref(''), running = ref(false)
async function run() {
  let data: any = {}
  try { data = JSON.parse(body.value || '{}') } catch { return showToast('JSON 参数格式错误') }
  running.value = true
  const res = await adminApi(action.value.trim(), data)
  result.value = JSON.stringify(res, null, 2)
  running.value = false
}
</script>