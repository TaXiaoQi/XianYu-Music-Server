<template>
  <div class="account-page">
    <!-- 页面头部 -->
    <Transition name="fade-down" appear>
      <div class="page-header">
        <div class="header-info">
          <h2 class="page-title">账户管理</h2>
          <p class="page-desc">查看当前管理员账户信息，并可绑定邮箱用于接收系统通知与安全提醒。</p>
        </div>
      </div>
    </Transition>

    <!-- 加载中 -->
    <div v-if="loading" class="state-box">
      <div class="spinner"></div>
      <span>加载中...</span>
    </div>

    <template v-else>
      <!-- 个人信息卡片 -->
      <Transition name="fade-up" appear>
        <div class="stats-row">
          <div
            v-for="(card, idx) in profileCards"
            :key="card.key"
            class="stat-chip"
            :style="{ animationDelay: `${idx * 70}ms` }"
          >
            <div class="stat-icon" :class="card.iconClass" v-html="card.icon"></div>
            <div class="stat-body">
              <span class="stat-label">{{ card.label }}</span>
              <span
                v-if="card.key === 'role'"
                class="role-badge"
                :class="account.role === 'super_admin' ? 'badge-super' : 'badge-admin'"
              >
                <svg v-if="account.role === 'super_admin'" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2l3 7h7l-5.5 4.5L18 21l-6-4-6 4 1.5-7.5L2 9h7z"/></svg>
                {{ account.role === 'super_admin' ? '超级管理员' : '管理员' }}
              </span>
              <span v-else class="stat-value" :class="{ 'value-muted': card.muted }">{{ card.value }}</span>
            </div>
          </div>
        </div>
      </Transition>

      <!-- 邮箱绑定 -->
      <Transition name="fade-up" appear>
        <div class="bind-card">
          <div class="bind-head">
            <div class="bind-title">
              <span class="bind-title-icon">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"/><polyline points="22,6 12,13 2,6"/></svg>
              </span>
              邮箱绑定
            </div>
            <span class="bind-status" :class="account.email ? 'status-bound' : 'status-unbound'">
              <span class="status-dot"></span>
              {{ account.email ? '已绑定' : '未绑定' }}
            </span>
          </div>
          <p class="bind-desc">绑定邮箱后可用于接收系统通知、安全提醒等重要信息。</p>
          <label class="bind-label required">邮箱地址</label>
          <div class="bind-form">
            <input
              v-model="emailInput"
              type="email"
              placeholder="请输入邮箱地址"
              autocomplete="off"
              :disabled="saving"
              @keyup.enter="doBind"
            />
            <button class="btn-save" :disabled="saving" @click="doBind">
              <span v-if="saving" class="btn-spinner"></span>
              {{ saving ? '保存中...' : '保存' }}
            </button>
          </div>
        </div>
      </Transition>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { adminApi, showToast } from '@/api/client'

interface AccountInfo {
  id: number
  username: string
  email: string
  role: string
  status: number
  created_at: string
  updated_at: string
  operation_count: number
  last_login_ip: string
  last_login_time: string
}

// ===== 状态 =====
const loading = ref(true)
const saving = ref(false)
const account = ref<AccountInfo>({} as AccountInfo)
const emailInput = ref('')

// ===== 个人信息卡片数据 =====
const profileCards = computed(() => [
  {
    key: 'username',
    label: '用户名',
    value: account.value.username || '-',
    iconClass: 'icon-user',
    icon: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>',
    muted: false,
  },
  {
    key: 'role',
    label: '角色',
    value: '',
    iconClass: 'icon-role',
    icon: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2l3 7h7l-5.5 4.5L18 21l-6-4-6 4 1.5-7.5L2 9h7z"/></svg>',
    muted: false,
  },
  {
    key: 'email',
    label: '邮箱',
    value: account.value.email || '未绑定',
    iconClass: 'icon-mail',
    icon: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"/><polyline points="22,6 12,13 2,6"/></svg>',
    muted: !account.value.email,
  },
  {
    key: 'last_login_ip',
    label: '最后登录 IP',
    value: account.value.last_login_ip || '-',
    iconClass: 'icon-ip',
    icon: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>',
    muted: false,
  },
  {
    key: 'operation_count',
    label: '操作次数',
    value: account.value.operation_count ?? 0,
    iconClass: 'icon-op',
    icon: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/></svg>',
    muted: false,
  },
  {
    key: 'last_login_time',
    label: '最后登录时间',
    value: account.value.last_login_time || '-',
    iconClass: 'icon-time',
    icon: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>',
    muted: false,
  },
])

// ===== 加载账户信息 =====
async function loadAccount() {
  loading.value = true
  const res = await adminApi<AccountInfo>('get_account_info')
  if (res.code === 200 && res.data) {
    account.value = res.data
    emailInput.value = res.data.email || ''
  } else {
    showToast(res.msg || '加载账户信息失败')
  }
  loading.value = false
}

// ===== 邮箱校验 =====
function isValidEmail(email: string): boolean {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)
}

// ===== 绑定邮箱 =====
async function doBind() {
  const email = emailInput.value.trim()
  if (!email) {
    showToast('请输入邮箱地址')
    return
  }
  if (!isValidEmail(email)) {
    showToast('邮箱格式不正确')
    return
  }
  saving.value = true
  const res = await adminApi('bind_email', { email })
  saving.value = false
  if (res.code === 200) {
    account.value.email = email
    showToast('邮箱绑定成功', 'success')
  } else {
    showToast(res.msg || '邮箱绑定失败')
  }
}

onMounted(() => {
  loadAccount()
})
</script>

<style scoped>
.account-page {
  max-width: 720px;
  margin: 0 auto;
}

/* ===== 页面头部 ===== */
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
  margin-bottom: 20px;
}
.page-title {
  font-size: 22px;
  font-weight: 800;
  letter-spacing: -0.02em;
  margin: 0 0 6px 0;
}
.page-desc {
  font-size: 13px;
  color: var(--text-muted);
  line-height: 1.6;
  margin: 0;
  max-width: 520px;
}

/* ===== 个人信息卡片 ===== */
.stats-row {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 12px;
  margin-bottom: 20px;
}
.stat-chip {
  background: var(--white);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 14px 16px;
  display: flex;
  align-items: center;
  gap: 12px;
  transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
  animation: cardIn 0.5s cubic-bezier(0.16, 1, 0.3, 1) backwards;
}
.stat-chip:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(0, 0, 0, 0.06);
}
.stat-icon {
  width: 38px;
  height: 38px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.icon-user { background: #eff6ff; color: #3b82f6; }
.icon-role { background: #fffbeb; color: #f59e0b; }
.icon-mail { background: #f0fdf4; color: #16a34a; }
.icon-ip   { background: #f0f9ff; color: #0284c7; }
.icon-op   { background: #fff7ed; color: #f97316; }
.icon-time { background: #f5f3ff; color: #8b5cf6; }
.stat-body {
  display: flex;
  flex-direction: column;
  min-width: 0;
}
.stat-label {
  font-size: 11px;
  color: var(--text-muted);
  margin-bottom: 3px;
}
.stat-value {
  font-size: 15px;
  font-weight: 700;
  line-height: 1.2;
  color: var(--text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.value-muted { color: var(--text-muted); font-weight: 500; }

/* 角色徽章 */
.role-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 3px 10px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 600;
  width: fit-content;
}
.badge-super { background: #fffbeb; color: #f59e0b; }
.badge-admin { background: #eff6ff; color: #3b82f6; }

@keyframes cardIn {
  from { opacity: 0; transform: translateY(16px); }
  to { opacity: 1; transform: translateY(0); }
}

/* ===== 邮箱绑定 ===== */
.bind-card {
  background: var(--white);
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 20px;
}
.bind-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 6px;
}
.bind-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 15px;
  font-weight: 700;
  color: var(--text);
}
.bind-title-icon {
  width: 30px;
  height: 30px;
  border-radius: 8px;
  background: #f0fdf4;
  color: #16a34a;
  display: flex;
  align-items: center;
  justify-content: center;
}
.bind-status {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 12px;
  font-weight: 600;
  padding: 3px 10px;
  border-radius: 20px;
}
.bind-status .status-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
}
.status-bound { background: #f0fdf4; color: #16a34a; }
.status-bound .status-dot { background: #16a34a; box-shadow: 0 0 0 3px rgba(22, 163, 74, 0.15); }
.status-unbound { background: #fffbeb; color: #d97706; }
.status-unbound .status-dot { background: #d97706; box-shadow: 0 0 0 3px rgba(217, 119, 6, 0.15); }
.bind-desc {
  font-size: 13px;
  color: var(--text-muted);
  line-height: 1.6;
  margin: 0 0 16px 0;
}
.bind-label {
  display: inline-block;
  margin: 0 0 8px;
  font-size: 13px;
  font-weight: 700;
  color: var(--text-light);
}
.bind-form {
  display: flex;
  gap: 10px;
}
.bind-form input {
  flex: 1;
  min-width: 0;
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 10px 14px;
  font-size: 14px;
  font-family: inherit;
  outline: none;
  transition: border-color 0.2s;
  box-sizing: border-box;
}
.bind-form input:focus { border-color: var(--accent); }
.bind-form input:disabled { background: #fafafa; cursor: not-allowed; }
.btn-save {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 10px 20px;
  border-radius: 10px;
  border: none;
  background: var(--accent);
  color: #fff;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}
.btn-save:hover { opacity: 0.85; transform: translateY(-1px); }
.btn-save:active { transform: scale(0.96); }
.btn-save:disabled { opacity: 0.5; cursor: not-allowed; transform: none; }
.btn-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: #fff;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

/* ===== 加载状态 ===== */
.state-box {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  color: var(--text-muted);
  gap: 12px;
  font-size: 14px;
}

/* ===== Spinner ===== */
.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid #e5e5e5;
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}
@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }

/* ===== 过渡动画 ===== */
.fade-down-enter-active, .fade-down-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-down-enter-from { opacity: 0; transform: translateY(-12px); }

.fade-up-enter-active, .fade-up-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-up-enter-from { opacity: 0; transform: translateY(12px); }

/* ===== 响应式 ===== */
@media (max-width: 768px) {
  .stats-row { grid-template-columns: 1fr 1fr; gap: 8px; }
  .stat-chip { padding: 12px; flex-direction: column; align-items: flex-start; gap: 8px; }
  .stat-value { font-size: 14px; }
  .bind-form { flex-direction: column; }
  .btn-save { justify-content: center; }
}
@media (max-width: 480px) {
  .stat-icon { width: 34px; height: 34px; }
  .bind-card { padding: 16px; }
}
</style>
