<template>
  <div class="email-auth-page">
    <div class="auth-card" :style="{ animationDelay: '0ms' }">
      <div class="brand">
        <div class="brand-icon">M</div>
        <div class="brand-text">
          <h1>弦予邮箱</h1>
          <p>注册登录测试系统</p>
        </div>
      </div>

      <form @submit.prevent="handleLogin">
        <div class="field" :style="{ animationDelay: '60ms' }">
          <label>邮箱地址</label>
          <input v-model="email" type="email" placeholder="请输入邮箱" required :disabled="loading" />
        </div>
        <div class="field" :style="{ animationDelay: '120ms' }">
          <label>登录密码</label>
          <input v-model="password" type="password" placeholder="请输入密码" required :disabled="loading" />
        </div>
        <button type="submit" class="submit-btn" :style="{ animationDelay: '180ms' }" :disabled="loading">
          <span v-if="loading" class="spinner"></span>
          {{ loading ? '登录中...' : '登 录' }}
        </button>
      </form>

      <div class="auth-links" :style="{ animationDelay: '240ms' }">
        <router-link to="/email/register">注册账号</router-link>
        <span class="divider">·</span>
        <router-link to="/email/forgot">忘记密码</router-link>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { emailLogin, setEmailToken, setEmailUser, getEmailToken, emailToast } from '@/api/email'

const router = useRouter()
const email = ref('')
const password = ref('')
const loading = ref(false)

async function handleLogin() {
  if (loading.value) return
  loading.value = true
  try {
    const res = await emailLogin(email.value.trim(), password.value)
    if (res.code === 200 && res.data) {
      setEmailToken(res.data.token)
      setEmailUser(res.data.user)
      emailToast('登录成功', 'success')
      router.push('/email/home')
    } else {
      emailToast(res.msg)
    }
  } finally {
    loading.value = false
  }
}

// 已登录则跳转主页
onMounted(() => {
  if (getEmailToken()) {
    router.replace('/email/home')
  }
})
</script>

<style scoped>
.email-auth-page {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #f0f0f0;
  padding: 20px;
}

.auth-card {
  background: #fff;
  border: 1px solid #e8e8e8;
  border-radius: 12px;
  padding: 44px 36px;
  width: 420px;
  max-width: 100%;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.06);
  animation: cardIn 0.5s ease both;
}

@keyframes cardIn {
  from { opacity: 0; transform: translateY(20px); }
  to { opacity: 1; transform: translateY(0); }
}

.brand {
  display: flex;
  align-items: center;
  gap: 14px;
  margin-bottom: 32px;
}
.brand-icon {
  width: 44px;
  height: 44px;
  border-radius: 10px;
  background: #1a1a1a;
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 20px;
  font-weight: 700;
  flex-shrink: 0;
}
.brand-text h1 {
  font-size: 20px;
  font-weight: 700;
  color: #1a1a1a;
  margin: 0;
}
.brand-text p {
  font-size: 12px;
  color: #999;
  margin: 2px 0 0;
}

.field {
  margin-bottom: 18px;
  animation: fieldIn 0.4s ease both;
}
@keyframes fieldIn {
  from { opacity: 0; transform: translateX(-12px); }
  to { opacity: 1; transform: translateX(0); }
}
.field label {
  display: block;
  font-size: 13px;
  color: #666;
  margin-bottom: 6px;
  font-weight: 500;
}
.field input {
  width: 100%;
  padding: 12px 14px;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  font-size: 14px;
  color: #1a1a1a;
  outline: none;
  transition: all 0.2s;
  background: #fafafa;
}
.field input:focus {
  border-color: #1a1a1a;
  background: #fff;
  box-shadow: 0 0 0 3px rgba(0, 0, 0, 0.04);
}

.submit-btn {
  width: 100%;
  padding: 13px;
  background: #1a1a1a;
  color: #fff;
  border: none;
  border-radius: 8px;
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  animation: fieldIn 0.4s ease both;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
}
.submit-btn:hover:not(:disabled) {
  background: #000;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}
.submit-btn:active:not(:disabled) {
  transform: translateY(0);
}
.submit-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.spinner {
  width: 16px;
  height: 16px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: #fff;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}
@keyframes spin {
  to { transform: rotate(360deg); }
}

.auth-links {
  margin-top: 24px;
  text-align: center;
  font-size: 13px;
  animation: fieldIn 0.4s ease both;
}
.auth-links a {
  color: #666;
  text-decoration: none;
  transition: color 0.15s;
}
.auth-links a:hover {
  color: #1a1a1a;
}
.auth-links .divider {
  color: #ddd;
  margin: 0 12px;
}

@media (max-width: 480px) {
  .auth-card {
    padding: 32px 20px;
  }
}
</style>
