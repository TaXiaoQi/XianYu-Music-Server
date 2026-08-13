<template>
  <div class="notify-page">
    <!-- 页面头部 -->
    <Transition name="fade-down" appear>
      <div class="page-header">
        <div class="header-info">
          <h2 class="page-title">外部通知</h2>
          <p class="page-desc">管理用于接收后台状态通知的绑定邮箱。启用后，服务端会将关键状态变更以邮件形式通知到以下邮箱。</p>
        </div>
        <button class="btn-add" @click="openAddModal">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
          </svg>
          新增通知邮箱
        </button>
      </div>
    </Transition>

    <!-- 统计卡片 -->
    <Transition name="fade-up" appear>
      <div class="stats-row">
        <div class="stat-chip">
          <div class="stat-icon stat-icon-total">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="4" width="20" height="16" rx="2"/><path d="M22 7l-10 6L2 7"/></svg>
          </div>
          <div class="stat-body"><span class="stat-num">{{ stats.total }}</span><span class="stat-label">总数</span></div>
        </div>
        <div class="stat-chip">
          <div class="stat-icon stat-icon-active">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
          </div>
          <div class="stat-body"><span class="stat-num">{{ stats.active }}</span><span class="stat-label">启用中</span></div>
        </div>
        <div class="stat-chip">
          <div class="stat-icon stat-icon-disabled">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="4.93" y1="4.93" x2="19.07" y2="19.07"/></svg>
          </div>
          <div class="stat-body"><span class="stat-num">{{ stats.disabled }}</span><span class="stat-label">已停用</span></div>
        </div>
      </div>
    </Transition>

    <!-- 加载中 -->
    <div v-if="loading" class="state-box">
      <div class="spinner"></div>
      <span>加载中...</span>
    </div>

    <template v-else>
      <!-- 空状态 -->
      <Transition name="fade-up" appear v-if="list.length === 0">
        <div class="state-box state-empty">
          <div class="empty-icon">
            <svg width="56" height="56" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2"><rect x="2" y="4" width="20" height="16" rx="2"/><path d="M22 7l-10 6L2 7"/></svg>
          </div>
          <p class="empty-title">暂无通知邮箱</p>
          <p class="empty-sub">点击右上角"新增通知邮箱"添加，用于接收后台状态通知</p>
        </div>
      </Transition>

      <!-- 通知邮箱列表 -->
      <div v-else class="notify-list">
        <TransitionGroup name="notify-card">
          <div
            v-for="(item, idx) in list"
            :key="item.id"
            class="notify-card"
            :class="{ 'is-disabled': item.status == 0 }"
            :style="{ animationDelay: `${idx * 60}ms` }"
          >
            <div class="notify-avatar" :class="item.status == 1 ? 'avatar-active' : 'avatar-disabled'">
              <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="4" width="20" height="16" rx="2"/><path d="M22 7l-10 6L2 7"/></svg>
            </div>
            <div class="notify-main">
              <div class="notify-info">
                <div class="notify-row">
                  <span class="notify-email">{{ item.email }}</span>
                  <span class="notify-status" :class="item.status == 1 ? 'st-active' : 'st-disabled'">
                    <span class="status-dot"></span>{{ item.status == 1 ? '启用中' : '已停用' }}
                  </span>
                </div>
                <p class="notify-remark">{{ item.remark || '暂无备注' }}</p>
                <p class="notify-time">创建时间：{{ item.created_at || '-' }}</p>
              </div>
              <div class="notify-actions">
                <button class="btn-action" @click="sendTest(item)">发送测试</button>
                <button class="btn-action" @click="toggle(item)">{{ item.status == 1 ? '停用' : '启用' }}</button>
                <button class="btn-action btn-danger" @click="remove(item)">删除</button>
              </div>
            </div>
          </div>
        </TransitionGroup>
      </div>
    </template>

    <!-- 新增通知邮箱弹窗 -->
    <Transition name="modal">
      <div v-if="addModalVisible" class="modal-backdrop" @click.self="closeAddModal">
        <div class="modal-dialog">
          <div class="modal-head">
            <h3>新增通知邮箱</h3>
            <button class="modal-close" @click="closeAddModal">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
          <div class="modal-body">
            <label class="modal-field">
              <span class="required">邮箱地址</span>
              <input v-model="form.email" type="email" placeholder="notify@example.com" @keydown.enter="doAdd" />
            </label>
            <label class="modal-field">
              <span>备注</span>
              <input v-model="form.remark" type="text" placeholder="例如：站长通知邮箱" @keydown.enter="doAdd" />
            </label>
            <p class="modal-tip">添加后默认启用，用于接收后台状态通知邮件。</p>
          </div>
          <div class="modal-foot">
            <button class="btn-cancel" @click="closeAddModal">取消</button>
            <button class="btn-save" :disabled="saving" @click="doAdd">
              <span v-if="saving" class="btn-spinner"></span>
              {{ saving ? '添加中...' : '确认添加' }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { adminApi, showToast } from '@/api/client'
import { webConfirm } from '@/utils/webDialog'

interface NotifyEmail {
  id: number
  email: string
  remark: string
  status: number
  created_at: string
  [key: string]: any
}

// ===== 状态 =====
const loading = ref(true)
const saving = ref(false)
const list = ref<NotifyEmail[]>([])

const stats = computed(() => {
  const total = list.value.length
  const active = list.value.filter(i => i.status == 1).length
  return { total, active, disabled: total - active }
})

// ===== 加载数据 =====
async function loadList() {
  loading.value = true
  const res = await adminApi<NotifyEmail[]>('list_notification_emails')
  if (res.code === 200 && Array.isArray(res.data)) {
    list.value = res.data
  } else {
    list.value = []
  }
  loading.value = false
}

// ===== 新增 =====
const addModalVisible = ref(false)
const form = ref<{ email: string; remark: string }>({ email: '', remark: '' })

function openAddModal() {
  form.value = { email: '', remark: '' }
  addModalVisible.value = true
}

function closeAddModal() {
  if (saving.value) return
  addModalVisible.value = false
}

function isValidEmail(email: string): boolean {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)
}

async function doAdd() {
  const email = form.value.email.trim()
  if (!email) {
    showToast('请输入邮箱地址')
    return
  }
  if (!isValidEmail(email)) {
    showToast('邮箱格式不正确')
    return
  }
  saving.value = true
  const res = await adminApi('add_notification_email', {
    email,
    remark: form.value.remark.trim(),
  })
  saving.value = false
  if (res.code === 200) {
    showToast('添加成功', 'success')
    addModalVisible.value = false
    loadList()
  } else {
    showToast(res.msg || '添加失败')
  }
}

// ===== 启用/停用 =====
async function toggle(item: NotifyEmail) {
  const res = await adminApi('toggle_notification_email', { id: item.id })
  if (res.code === 200) {
    item.status = item.status == 1 ? 0 : 1
    showToast('已更新', 'success')
  } else {
    showToast(res.msg || '操作失败')
  }
}

// ===== 删除 =====
async function remove(item: NotifyEmail) {
  const ok = await webConfirm(`确认删除通知邮箱 "${item.email}"？`, { title: '删除通知邮箱', confirmText: '确认删除' })
  if (!ok) return
  const res = await adminApi('delete_notification_email', { id: item.id })
  if (res.code === 200) {
    showToast('删除成功', 'success')
    list.value = list.value.filter(i => i.id !== item.id)
  } else {
    showToast(res.msg || '删除失败')
  }
}

// ===== 发送测试通知 =====
const testingId = ref<number | null>(null)
async function sendTest(item: NotifyEmail) {
  testingId.value = item.id
  const res = await adminApi('test_notification_email', { email: item.email })
  testingId.value = null
  if (res.code === 200) {
    showToast(res.msg || '测试通知已发送', 'success')
  } else {
    showToast(res.msg || '发送失败')
  }
}

onMounted(loadList)
</script>

<style scoped>
.notify-page {
  max-width: 860px;
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
  max-width: 560px;
}
.btn-add {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 10px 18px;
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
.btn-add:hover { opacity: 0.85; transform: translateY(-1px); }
.btn-add:active { transform: scale(0.96); }

/* ===== 统计 ===== */
.stats-row {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
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
.stat-icon-total { background: #eff6ff; color: #3b82f6; }
.stat-icon-active { background: #f0fdf4; color: #16a34a; }
.stat-icon-disabled { background: #fef2f2; color: #ef4444; }
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
.stat-num {
  font-size: 20px;
  font-weight: 800;
  line-height: 1.2;
  color: var(--text);
}
@keyframes cardIn {
  from { opacity: 0; transform: translateY(16px); }
  to { opacity: 1; transform: translateY(0); }
}

/* ===== 列表 ===== */
.notify-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.notify-card {
  display: flex;
  align-items: flex-start;
  gap: 14px;
  background: var(--white);
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 16px;
  transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
  animation: cardIn 0.45s cubic-bezier(0.16, 1, 0.3, 1) backwards;
}
.notify-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.06);
}
.notify-card.is-disabled { opacity: 0.62; }
.notify-avatar {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.avatar-active { background: #eff6ff; color: #3b82f6; }
.avatar-disabled { background: #f3f4f6; color: #9ca3af; }
.notify-main {
  flex: 1;
  min-width: 0;
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
}
.notify-info { min-width: 0; }
.notify-row {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}
.notify-email {
  font-size: 15px;
  font-weight: 700;
  color: var(--text);
  word-break: break-all;
}
.notify-status {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 11px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 20px;
}
.notify-status .status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
}
.st-active { background: #f0fdf4; color: #16a34a; }
.st-active .status-dot { background: #16a34a; }
.st-disabled { background: #f3f4f6; color: #9ca3af; }
.st-disabled .status-dot { background: #9ca3af; }
.notify-remark {
  margin: 6px 0 2px;
  font-size: 13px;
  color: var(--text-light);
}
.notify-time {
  margin: 0;
  font-size: 11px;
  color: var(--text-muted);
}
.notify-actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}
.btn-action {
  border: 1px solid var(--border);
  border-radius: 8px;
  background: transparent;
  color: var(--text-light);
  padding: 6px 12px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}
.btn-action:hover { border-color: var(--accent); color: var(--accent); background: var(--accent-soft); }
.btn-action.btn-danger:hover { border-color: #ef4444; color: #ef4444; background: #fef2f2; }

/* ===== 弹窗 ===== */
.modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 1000;
  background: rgba(0, 0, 0, 0.4);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
}
.modal-dialog {
  width: 100%;
  max-width: 420px;
  background: var(--card-solid, var(--white));
  border-radius: 16px;
  box-shadow: 0 24px 60px rgba(0, 0, 0, 0.2);
  overflow: hidden;
}
.modal-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 18px 20px 0;
}
.modal-head h3 { margin: 0; font-size: 16px; font-weight: 800; color: var(--text); }
.modal-close {
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  padding: 4px;
  border-radius: 8px;
  display: flex;
}
.modal-close:hover { background: var(--control-bg); color: var(--text); }
.modal-body {
  padding: 16px 20px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.modal-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.modal-field span {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-light);
}
.modal-field span.required::after { content: '*'; color: #ef4444; margin-left: 2px; }
.modal-field input {
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 10px 12px;
  font-size: 14px;
  font-family: inherit;
  outline: none;
  background: var(--control-bg);
  color: var(--text);
  transition: border-color 0.2s;
  width: 100%;
  box-sizing: border-box;
}
.modal-field input:focus { border-color: var(--accent); }
.modal-tip {
  margin: 0;
  font-size: 12px;
  color: var(--text-muted);
  line-height: 1.5;
}
.modal-foot {
  display: flex;
  gap: 10px;
  padding: 14px 20px 18px;
}
.btn-cancel, .btn-save {
  flex: 1;
  padding: 10px;
  border-radius: 10px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  transition: all 0.2s;
}
.btn-cancel {
  border: 1px solid var(--border);
  background: transparent;
  color: var(--text-muted);
}
.btn-cancel:hover { background: var(--control-bg); }
.btn-save {
  border: none;
  background: var(--accent);
  color: #fff;
}
.btn-save:hover:not(:disabled) { opacity: 0.85; }
.btn-save:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: #fff;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

/* ===== 加载/空状态 ===== */
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
.state-empty { padding: 80px 20px; }
.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid #e5e5e5;
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}
.empty-icon { color: var(--text-light); opacity: 0.5; }
.empty-title { font-size: 16px; font-weight: 700; color: var(--text-light); margin: 0; }
.empty-sub { font-size: 13px; color: var(--text-muted); margin: 0; }
@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }

/* ===== 过渡动画 ===== */
.fade-down-enter-active, .fade-down-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-down-enter-from { opacity: 0; transform: translateY(-12px); }
.fade-up-enter-active, .fade-up-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-up-enter-from { opacity: 0; transform: translateY(12px); }
.notify-card-enter-active, .notify-card-leave-active { transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1); }
.notify-card-enter-from { opacity: 0; transform: translateY(12px); }
.notify-card-leave-to { opacity: 0; transform: translateY(-8px); }
.modal-enter-active, .modal-leave-active { transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1); }
.modal-enter-from, .modal-leave-to { opacity: 0; transform: scale(0.96) translateY(8px); }

/* ===== 响应式 ===== */
@media (max-width: 640px) {
  .stats-row { grid-template-columns: 1fr 1fr; gap: 8px; }
  .notify-main { flex-direction: column; }
  .page-header { flex-direction: column; }
  .btn-add { width: 100%; justify-content: center; }
}
</style>