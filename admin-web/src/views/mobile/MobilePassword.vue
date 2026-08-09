<template>
  <div class="mobile-page">
    <div class="mobile-card mobile-form">
      <h3 class="mobile-card-title">修改用户名</h3>
      <input v-model="username" class="mobile-input" placeholder="新用户名" />
      <button class="mobile-btn primary" @click="changeUsername">保存用户名</button>
    </div>
    <div class="mobile-card mobile-form">
      <h3 class="mobile-card-title">修改密码</h3>
      <input v-model="oldPassword" class="mobile-input" type="password" placeholder="原密码" />
      <input v-model="newPassword" class="mobile-input" type="password" placeholder="新密码" />
      <input v-model="confirmPassword" class="mobile-input" type="password" placeholder="确认新密码" />
      <button class="mobile-btn primary" @click="changePassword">保存密码</button>
    </div>
  </div>
</template>
<script setup lang="ts">
import { ref } from 'vue'
import { adminApi, showToast } from '@/api/client'
import './MobilePage.css'
const username = ref(''), oldPassword = ref(''), newPassword = ref(''), confirmPassword = ref('')
async function changeUsername() { const res = await adminApi('change_username', { new_username: username.value.trim() }); res.code === 200 ? showToast('用户名已更新', 'success') : showToast(res.msg || '更新失败') }
async function changePassword() { const res = await adminApi('change_password', { old_password: oldPassword.value, new_password: newPassword.value, confirm_password: confirmPassword.value }); if (res.code === 200) { showToast('密码已更新', 'success'); oldPassword.value = newPassword.value = confirmPassword.value = '' } else showToast(res.msg || '更新失败') }
</script>