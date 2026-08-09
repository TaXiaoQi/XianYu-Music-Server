<template>
  <div class="password-page">
    <!-- 页面头部 -->
    <Transition name="fade-down" appear>
      <div class="page-header">
        <h2 class="page-title">修改密码</h2>
        <p class="page-desc">在此更新后台登录用户名与登录密码。为保障账户安全，建议定期更换密码，并使用包含字母、数字与符号的强密码。</p>
      </div>
    </Transition>

    <!-- 卡片区域 -->
    <div class="card-grid">
      <!-- 修改用户名 -->
      <Transition name="fade-up" appear>
        <div class="card">
          <div class="card-head">
            <div class="card-icon icon-user">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
            </div>
            <div class="card-head-text">
              <h3 class="card-title">修改用户名</h3>
              <p class="card-desc">更新登录所用的用户名</p>
            </div>
          </div>
          <div class="card-body">
            <div class="field">
              <label class="required">新用户名</label>
              <input v-model="usernameForm.new_username" type="text" placeholder="请输入新用户名" autocomplete="off" />
            </div>
            <button class="btn-save" :disabled="usernameSaving" @click="submitUsername">
              <span v-if="usernameSaving" class="btn-spinner"></span>
              {{ usernameSaving ? '提交中...' : '保存用户名' }}
            </button>
          </div>
        </div>
      </Transition>

      <!-- 修改密码 -->
      <Transition name="fade-up" appear>
        <div class="card card-delay">
          <div class="card-head">
            <div class="card-icon icon-lock">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
            </div>
            <div class="card-head-text">
              <h3 class="card-title">修改密码</h3>
              <p class="card-desc">新密码至少 6 个字符，建议字母与数字搭配</p>
            </div>
          </div>
          <div class="card-body">
            <div class="field">
              <label class="required">当前密码</label>
              <input v-model="passwordForm.old_password" type="password" placeholder="请输入当前密码" autocomplete="current-password" />
            </div>
            <div class="field">
              <label class="required">新密码</label>
              <input v-model="passwordForm.new_password" type="password" placeholder="至少 6 个字符" autocomplete="new-password" />
              <Transition name="fade-up">
                <div v-if="passwordForm.new_password" class="strength">
                  <div class="strength-bars">
                    <span class="bar" :class="strengthLevel() >= 1 ? 'lv-' + strengthLevel() : ''"></span>
                    <span class="bar" :class="strengthLevel() >= 2 ? 'lv-' + strengthLevel() : ''"></span>
                    <span class="bar" :class="strengthLevel() >= 3 ? 'lv-' + strengthLevel() : ''"></span>
                  </div>
                  <span class="strength-label" :class="'lv-' + strengthLevel()">{{ strengthText() }}</span>
                </div>
              </Transition>
            </div>
            <div class="field">
              <label class="required">确认新密码</label>
              <input v-model="passwordForm.confirm_password" type="password" placeholder="再次输入新密码" autocomplete="new-password" />
              <span v-if="passwordForm.confirm_password && passwordForm.confirm_password !== passwordForm.new_password" class="hint hint-error">两次输入的密码不一致</span>
              <span v-else-if="passwordForm.confirm_password && passwordForm.confirm_password === passwordForm.new_password" class="hint hint-ok">两次密码一致</span>
            </div>
            <button class="btn-save" :disabled="passwordSaving" @click="submitPassword">
              <span v-if="passwordSaving" class="btn-spinner"></span>
              {{ passwordSaving ? '提交中...' : '保存密码' }}
            </button>
          </div>
        </div>
      </Transition>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { adminApi, showToast } from '@/api/client'

// ===== 修改用户名 =====
const usernameForm = ref<{ new_username: string }>({ new_username: '' })
const usernameSaving = ref(false)

async function submitUsername() {
  const uname = usernameForm.value.new_username.trim()
  if (!uname) {
    showToast('请输入新用户名')
    return
  }
  usernameSaving.value = true
  const res = await adminApi('change_username', { new_username: uname })
  usernameSaving.value = false
  if (res.code === 200) {
    showToast('用户名修改成功', 'success')
    usernameForm.value.new_username = ''
  } else {
    showToast(res.msg || '用户名修改失败')
  }
}

// ===== 修改密码 =====
const passwordForm = ref<{ old_password: string; new_password: string; confirm_password: string }>({
  old_password: '',
  new_password: '',
  confirm_password: '',
})
const passwordSaving = ref(false)

// 密码强度：综合长度与字符种类（小写 / 大写 / 数字 / 符号）
function strengthLevel(): number {
  const pw = passwordForm.value.new_password
  if (!pw) return 0
  let variety = 0
  if (/[a-z]/.test(pw)) variety++
  if (/[A-Z]/.test(pw)) variety++
  if (/[0-9]/.test(pw)) variety++
  if (/[^a-zA-Z0-9]/.test(pw)) variety++
  const len = pw.length
  if (len < 6) return 1
  if (len >= 8 && variety >= 3) return 3
  if (len >= 6 && variety >= 2) return 2
  return 1
}

function strengthText(): string {
  const map: Record<number, string> = { 1: '弱', 2: '中', 3: '强' }
  return map[strengthLevel()] || ''
}

async function submitPassword() {
  const { old_password, new_password, confirm_password } = passwordForm.value
  if (!old_password) {
    showToast('请输入当前密码')
    return
  }
  if (new_password.length < 6) {
    showToast('新密码至少需要 6 个字符')
    return
  }
  if (new_password !== confirm_password) {
    showToast('两次输入的新密码不一致')
    return
  }
  passwordSaving.value = true
  const res = await adminApi('change_password', { old_password, new_password, confirm_password })
  passwordSaving.value = false
  if (res.code === 200) {
    showToast('密码修改成功', 'success')
    passwordForm.value = { old_password: '', new_password: '', confirm_password: '' }
  } else {
    showToast(res.msg || '密码修改失败')
  }
}
</script>

<style scoped>
.password-page {
  max-width: 720px;
  margin: 0 auto;
}

/* ===== 页面头部 ===== */
.page-header {
  margin-bottom: 20px;
}
.page-title {
  font-size: 22px;
  font-weight: 800;
  letter-spacing: -0.02em;
  margin: 0 0 6px 0;
  color: var(--text);
}
.page-desc {
  font-size: 13px;
  color: var(--text-muted);
  line-height: 1.6;
  margin: 0;
  max-width: 520px;
}

/* ===== 卡片网格 ===== */
.card-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
  align-items: start;
}

.card {
  background: var(--white);
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 18px;
  transition: box-shadow 0.3s cubic-bezier(0.16, 1, 0.3, 1), transform 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}
.card:hover {
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.06);
  transform: translateY(-2px);
}

/* 卡片头部 */
.card-head {
  display: flex;
  align-items: center;
  gap: 12px;
}
.card-icon {
  width: 40px;
  height: 40px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.icon-user { background: #eff6ff; color: #3b82f6; }
.icon-lock { background: #fffbeb; color: #f59e0b; }
.card-head-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}
.card-title {
  font-size: 16px;
  font-weight: 700;
  margin: 0;
  color: var(--text);
}
.card-desc {
  font-size: 12px;
  color: var(--text-muted);
  margin: 0;
  line-height: 1.5;
}

/* 卡片主体 */
.card-body {
  display: flex;
  flex-direction: column;
}

/* ===== 表单字段 ===== */
.field { margin-bottom: 16px; }
.field:last-child { margin-bottom: 0; }
.field label {
  display: block;
  font-size: 13px;
  font-weight: 600;
  color: var(--text);
  margin-bottom: 8px;
}
.field input {
  width: 100%;
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 10px 14px;
  font-size: 14px;
  font-family: inherit;
  outline: none;
  transition: border-color 0.2s;
  box-sizing: border-box;
  color: var(--text);
  background: var(--white);
}
.field input::placeholder { color: var(--text-light); }
.field input:focus { border-color: var(--accent); }

/* 提示文本 */
.hint {
  display: block;
  font-size: 11px;
  margin-top: 6px;
  font-weight: 500;
}
.hint-error { color: #ef4444; }
.hint-ok { color: #16a34a; }

/* ===== 密码强度 ===== */
.strength {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 8px;
}
.strength-bars {
  display: flex;
  gap: 4px;
  flex: 1;
}
.bar {
  flex: 1;
  height: 4px;
  border-radius: 2px;
  background: #ececf0;
  transition: background 0.25s ease;
}
.bar.lv-1 { background: #ef4444; }
.bar.lv-2 { background: #f59e0b; }
.bar.lv-3 { background: #22c55e; }
.strength-label {
  font-size: 11px;
  font-weight: 600;
  white-space: nowrap;
  color: var(--text-light);
}
.strength-label.lv-1 { color: #ef4444; }
.strength-label.lv-2 { color: #f59e0b; }
.strength-label.lv-3 { color: #22c55e; }

/* ===== 保存按钮 ===== */
.btn-save {
  padding: 12px 20px;
  border-radius: 10px;
  border: none;
  background: var(--accent);
  color: #fff;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  transition: all 0.2s;
  width: 100%;
  justify-content: center;
}
.btn-save:hover:not(:disabled) { opacity: 0.85; }
.btn-save:active:not(:disabled) { transform: scale(0.98); }
.btn-save:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: #fff;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}
@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }

/* ===== 过渡动画 ===== */
.fade-down-enter-active, .fade-down-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-down-enter-from { opacity: 0; transform: translateY(-12px); }

.fade-up-enter-active, .fade-up-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-up-enter-from { opacity: 0; transform: translateY(12px); }

/* 卡片错开延迟 */
.card-delay.fade-up-enter-active { transition-delay: 0.1s; }

/* ===== 响应式 ===== */
@media (max-width: 640px) {
  .card-grid { grid-template-columns: 1fr; }
}
</style>
