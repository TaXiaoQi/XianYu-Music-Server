<template>
  <div class="mobile-page">
    <div class="mobile-card mobile-form">
      <select v-model="form.email_provider" class="mobile-select">
        <option value="builtin">内置邮箱机</option>
        <option value="api">外部 API</option>
      </select>
      <input v-model="form.email_sender" class="mobile-input" placeholder="发件邮箱" />
      <input v-model="form.email_api_primary" class="mobile-input" placeholder="邮箱 API 主地址" />
      <input v-model="form.email_api_backup" class="mobile-input" placeholder="邮箱 API 备用地址" />
      <input v-model="form.smtp_username" class="mobile-input" placeholder="SMTP 用户名（留空则使用发件邮箱）" />
      <input v-model="form.smtp_password" class="mobile-input" placeholder="SMTP 密码 / 授权码" type="password" />
      <button class="mobile-btn primary" :disabled="saving" @click="save">保存配置</button>
    </div>
    <div class="mobile-card mobile-form">
      <h3 class="mobile-card-title">发送测试</h3>
      <input v-model="testEmail" class="mobile-input" placeholder="测试收件邮箱" />
      <button class="mobile-btn" :disabled="testing" @click="test">发送测试邮件</button>
    </div>
  </div>
</template>
<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'
import './MobilePage.css'
const saving = ref(false), testing = ref(false), testEmail = ref('')
const form = ref<any>({ email_provider: 'builtin', email_sender: '', email_api_primary: '', email_api_backup: '', smtp_host: '', smtp_port: '465', smtp_username: '', smtp_password: '' })
async function load() { const res = await adminApi<any>('get_email_config'); if (res.code === 200 && res.data) form.value = { ...form.value, ...res.data, smtp_password: '' } }
async function save() { saving.value = true; const res = await adminApi('update_email_config', form.value); saving.value = false; res.code === 200 ? showToast('已保存', 'success') : showToast(res.msg || '保存失败') }
async function test() { if (!testEmail.value.trim()) return showToast('请填写测试邮箱'); testing.value = true; const res = await adminApi('test_email_config', { email: testEmail.value.trim() }); testing.value = false; res.code === 200 ? showToast('发送成功', 'success') : showToast(res.msg || '发送失败') }
onMounted(load)
</script>