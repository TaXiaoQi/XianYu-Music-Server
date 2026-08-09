<template>
  <div class="login-page">
    <div class="login-container">
      <h1>弦予音乐</h1>
      <p class="subtitle">后台管理系统</p>
      <div v-if="timeoutTip" class="timeout-tip">登录已过期，请重新登录</div>
      <form @submit.prevent="handleLogin">
        <div class="form-group">
          <label class="required">管理员账号</label>
          <input
            v-model="username"
            type="text"
            placeholder="请输入用户名"
            required
            autocomplete="username"
            :disabled="loading"
          />
        </div>
        <div class="form-group">
          <label class="required">登录密码</label>
          <input
            v-model="password"
            type="password"
            placeholder="请输入密码"
            required
            autocomplete="current-password"
            :disabled="loading"
          />
        </div>
        <button type="submit" class="login-btn" :disabled="loading">
          {{ loading ? '登录中...' : '登 录' }}
        </button>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { showToast } from '@/api/client'

const router = useRouter()
const route = useRoute()
const auth = useAuthStore()

const username = ref('')
const password = ref('')
const loading = ref(false)
const timeoutTip = ref(false)

onMounted(() => {
  if (route.query.timeout) {
    timeoutTip.value = true
  }
})

async function handleLogin() {
  if (loading.value) return
  loading.value = true
  try {
    const result = await auth.login(username.value.trim(), password.value)
    if (result.success) {
      showToast('登录成功', 'success')
      router.push('/dashboard')
    } else {
      showToast(result.msg)
    }
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.login-page {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--page-bg);
  overflow: hidden;
}
.login-container {
  background: var(--card);
  border-radius: 24px;
  padding: 48px 40px;
  width: 420px;
  max-width: 90vw;
  border: 1px solid var(--sidebar-border);
  box-shadow: var(--shadow-card);
  backdrop-filter: blur(22px);
  animation: loginEnter 0.42s cubic-bezier(0.16, 1, 0.3, 1) both;
}
.login-container h1 {
  font-size: 26px;
  font-weight: 850;
  text-align: center;
  margin-bottom: 8px;
  color: var(--text);
  letter-spacing: -0.03em;
}
.subtitle {
  color: var(--text-muted);
  font-size: 14px;
  text-align: center;
  margin-bottom: 36px;
}
.timeout-tip {
  background: rgba(236, 65, 65, 0.08);
  border: 1px solid rgba(236, 65, 65, 0.16);
  color: #EC4141;
  padding: 10px 14px;
  border-radius: 14px;
  font-size: 12px;
  text-align: center;
  margin-bottom: 18px;
}
.form-group { margin-bottom: 20px; }
.form-group label {
  display: block;
  color: var(--text-light);
  font-size: 13px;
  margin-bottom: 8px;
  font-weight: 500;
}
.form-group input {
  width: 100%;
  padding: 12px 14px;
  background: var(--control-bg);
  border: 1.5px solid var(--border);
  border-radius: 14px;
  color: var(--text);
  font-size: 14px;
  outline: none;
  transition: border-color 0.2s, box-shadow 0.2s, background 0.2s;
}
.form-group input:focus {
  background: var(--card-solid);
  border-color: #EC4141;
  box-shadow: 0 0 0 4px rgba(236, 65, 65, 0.10);
}
.login-btn {
  width: 100%;
  padding: 12px;
  background: #EC4141;
  color: #fff;
  border: none;
  border-radius: 14px;
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1), background 0.2s, box-shadow 0.2s;
  margin-top: 8px;
  box-shadow: 0 12px 24px rgba(236, 65, 65, 0.22);
}
.login-btn:hover:not(:disabled) {
  background: #d83a3a;
  transform: translateY(-1px);
  box-shadow: 0 16px 30px rgba(236, 65, 65, 0.28);
}
.login-btn:active:not(:disabled) { transform: scale(0.98); }
.login-btn:disabled { opacity: 0.6; cursor: not-allowed; }
@keyframes loginEnter {
  from { opacity: 0; transform: translateY(16px) scale(0.97); }
  to { opacity: 1; transform: translateY(0) scale(1); }
}
</style>
