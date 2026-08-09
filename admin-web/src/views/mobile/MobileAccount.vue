<template>
  <div class="mobile-page">
    <div class="mobile-card">
      <h3 class="mobile-card-title">{{ info.username || '管理员账户' }}</h3>
      <div class="mobile-item-sub">角色：{{ info.role || '-' }}</div>
      <div class="mobile-item-sub">邮箱：{{ info.email || '未绑定' }}</div>
      <div class="mobile-item-sub">创建时间：{{ info.created_at || '-' }}</div>
    </div>
    <div class="mobile-card mobile-form">
      <input v-model="email" class="mobile-input" placeholder="绑定邮箱" />
      <button class="mobile-btn primary" @click="bindEmail">保存邮箱</button>
    </div>
  </div>
</template>
<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'
import './MobilePage.css'
const info = ref<any>({}), email = ref('')
async function load() { const res = await adminApi<any>('get_account_info'); if (res.code === 200 && res.data) { info.value = res.data; email.value = res.data.email || '' } }
async function bindEmail() { const res = await adminApi('bind_email', { email: email.value.trim() }); res.code === 200 ? showToast('已保存', 'success') : showToast(res.msg || '保存失败') }
onMounted(load)
</script>