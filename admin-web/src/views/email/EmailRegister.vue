<template>
  <div class="email-auth-page">
    <div class="auth-card">
      <div class="brand">
        <div class="brand-icon">M</div>
        <div class="brand-text">
          <h1>注册账号</h1>
          <p>弦予邮箱注册登录测试</p>
        </div>
      </div>

      <form @submit.prevent="handleRegister">
        <div class="field" :style="{ animationDelay: '40ms' }">
          <label>邮箱地址</label>
          <div class="input-with-btn">
            <input v-model="email" type="email" placeholder="请输入邮箱" required :disabled="loading" />
            <button type="button" class="code-btn" :disabled="countdown > 0 || sendingCode" @click="handleSendCode">
              {{ countdown > 0 ? `${countdown}s` : '获取验证码' }}
            </button>
          </div>
        </div>
        <div class="field" :style="{ animationDelay: '80ms' }">
          <label>邮箱验证码</label>
          <input v-model="code" type="text" placeholder="请输入6位验证码" maxlength="6" required :disabled="loading" />
        </div>
        <div class="field" :style="{ animationDelay: '120ms' }">
          <label>设置密码（6-32位）</label>
          <input v-model="password" type="password" placeholder="请输入密码" required :disabled="loading" />
        </div>
        <div class="field" :style="{ animationDelay: '160ms' }">
          <label>确认密码</label>
          <input v-model="password2" type="password" placeholder="请再次输入密码" required :disabled="loading" />
        </div>
        <div class="field" :style="{ animationDelay: '200ms' }">
          <label>昵称（可选）</label>
          <input v-model="nickname" type="text" placeholder="给自己取个名字" :disabled="loading" />
        </div>
        <button type="submit" class="submit-btn" :style="{ animationDelay: '240ms' }" :disabled="loading">
          <span v-if="loading" class="spinner"></span>
          {{ loading ? '注册中...' : '注 册' }}
        </button>
      </form>

      <div class="auth-links" :style="{ animationDelay: '280ms' }">
        已有账号？<router-link to="/email/login">直接登录</router-link>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { sendCode, emailRegister, getEmailToken, emailToast } from '@/api/email'

const router = useRouter()
const email = ref('')
const code = ref('')
const password = ref('')
const password2 = ref('')
const nickname = ref('')
const loading = ref(false)
const sendingCode = ref(false)
const countdown = ref(0)

onMounted(() => {
  if (getEmailToken()) {
    router.replace('/email/home')
  }
})

function startCountdown() {
  countdown.value = 60
  const timer = setInterval(() => {
    countdown.value--
    if (countdown.value <= 0) clearInterval(timer)
  }, 1000)
}

async function handleSendCode() {
  const e = email.value.trim()
  if (!e) { emailToast('请输入邮箱地址'); return }
  if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(e)) { emailToast('邮箱格式不正确'); return }

  sendingCode.value = true
  const res = await sendCode(e)
  sendingCode.value = false

  if (res.code === 200) {
    emailToast(res.msg || '验证码已发送', 'success')
    startCountdown()
  } else {
    emailToast(res.msg || '发送失败')
  }
}

async function handleRegister() {
  if (loading.value) return

  const e = email.value.trim()
  if (!e || !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(e)) { emailToast('请输入合法的邮箱地址'); return }
  if (!code.value.trim()) { emailToast('请输入验证码'); return }
  if (password.value.length < 6 || password.value.length > 32) { emailToast('密码长度需为 6-32 位'); return }
  if (password.value !== password2.value) { emailToast('两次输入的密码不一致'); return }

  loading.value = true
  try {
    const res = await emailRegister(e, code.value.trim(), password.value, password2.value, nickname.value.trim())
    if (res.code === 200) {
      emailToast('注册成功，请登录', 'success')
      setTimeout(() => router.push('/email/login'), 1200)
    } else {
      emailToast(res.msg)
    }
  } finally {
    loading.value = false
  }
}
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
  padding: 40px 36px;
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
  margin-bottom: 28px;
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
.brand-text h1 { font-size: 20px; font-weight: 700; color: #1a1a1a; margin: 0; }
.brand-text p { font-size: 12px; color: #999; margin: 2px 0 0; }

.field {
  margin-bottom: 16px;
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
  padding: 11px 14px;
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

.input-with-btn {
  display: flex;
  gap: 10px;
}
.input-with-btn input {
  flex: 1;
}
.code-btn {
  white-space: nowrap;
  padding: 11px 16px;
  border: 1px solid #1a1a1a;
  border-radius: 8px;
  background: transparent;
  color: #1a1a1a;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}
.code-btn:hover:not(:disabled) {
  background: #1a1a1a;
  color: #fff;
}
.code-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  border-color: #ccc;
  color: #999;
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
  margin-top: 4px;
}
.submit-btn:hover:not(:disabled) {
  background: #000;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}
.submit-btn:disabled { opacity: 0.6; cursor: not-allowed; }

.spinner {
  width: 16px;
  height: 16px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: #fff;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }

.auth-links {
  margin-top: 20px;
  text-align: center;
  font-size: 13px;
  color: #999;
  animation: fieldIn 0.4s ease both;
}
.auth-links a {
  color: #1a1a1a;
  font-weight: 500;
  text-decoration: none;
}
.auth-links a:hover { text-decoration: underline; }

@media (max-width: 480px) {
  .auth-card { padding: 28px 20px; }
}
</style>
