<template>
  <div class="mobile-page">
    <div class="mobile-card mobile-form">
      <input v-model="form.app_name" class="mobile-input" placeholder="应用名称" />
      <input v-model="form.version" class="mobile-input" placeholder="版本号" />
      <input v-model="form.official_website" class="mobile-input" placeholder="官网地址" />
      <textarea v-model="form.description" class="mobile-textarea" placeholder="关于页说明"></textarea>
      <button class="mobile-btn primary" :disabled="saving" @click="save">保存配置</button>
    </div>
  </div>
</template>
<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'
import './MobilePage.css'
const saving = ref(false)
const form = ref<any>({ app_name: '', version: '', official_website: '', description: '' })
async function load() { const res = await adminApi<any>('get_about_config_admin'); if (res.code === 200 && res.data) form.value = { ...form.value, ...res.data } }
async function save() { saving.value = true; const res = await adminApi('save_about_config', form.value); saving.value = false; res.code === 200 ? showToast('已保存', 'success') : showToast(res.msg || '保存失败') }
onMounted(load)
</script>