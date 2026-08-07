<template>
  <div class="login-page">
    <div class="login-container">
      <h1>弦予音乐</h1>
      <p class="subtitle">后台管理系统</p>
      <div v-if="timeoutTip" class="timeout-tip">登录已过期，请重新登录</div>
      <form @submit.prevent="handleLogin">
        <div class="form-group">
          <label>管理员账号</label>
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
          <label>登录密码</label>
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
  background: #f5f5f5;
  overflow: hidden;
}
.login-container {
  background: #fff;
  border-radius: 8px;
  padding: 48px 40px;
  width: 420px;
  max-width: 90vw;
  border: 1px solid #e5e5e5;
  box-shadow: 0 1px 3px rgba(0,0,0,0.08);
}
.login-container h1 {
  font-size: 26px;
  font-weight: 700;
  text-align: center;
  margin-bottom: 8px;
  color: #1a1a1a;
}
.subtitle {
  color: #666;
  font-size: 14px;
  text-align: center;
  margin-bottom: 36px;
}
.timeout-tip {
  background: #fffdf0;
  border: 1px solid #e0d090;
  color: #b8860b;
  padding: 10px 14px;
  border-radius: 6px;
  font-size: 12px;
  text-align: center;
  margin-bottom: 18px;
}
.form-group { margin-bottom: 20px; }
.form-group label {
  display: block;
  color: #666;
  font-size: 13px;
  margin-bottom: 8px;
  font-weight: 500;
}
.form-group input {
  width: 100%;
  padding: 12px 14px;
  background: #fff;
  border: 1px solid #e5e5e5;
  border-radius: 6px;
  color: #1a1a1a;
  font-size: 14px;
  outline: none;
  transition: border-color 0.15s;
}
.form-group input:focus {
  border-color: #1a1a1a;
  box-shadow: 0 0 0 2px rgba(0,0,0,0.05);
}
.login-btn {
  width: 100%;
  padding: 12px;
  background: #1a1a1a;
  color: #fff;
  border: none;
  border-radius: 6px;
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.15s;
  margin-top: 8px;
}
.login-btn:hover:not(:disabled) { background: #000; }
.login-btn:disabled { opacity: 0.6; cursor: not-allowed; }
</style>
