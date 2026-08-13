<template>
  <div class="notify-page">
    <!-- 页面头部 -->
    <Transition name="fade-down" appear>
      <div class="page-header">
        <div class="header-info">
          <h2 class="page-title">外部通知</h2>
          <p class="page-desc">管理用于接收后台状态通知的绑定邮箱，并可分别设置壁纸审核、头像审核、昵称审核、反馈更新等板块的通知开关。</p>
        </div>
        <div class="header-actions">
          <button class="btn-ghost" @click="doImportAdmin" :disabled="importing">
            <span v-if="importing" class="btn-spinner btn-spinner-dark"></span>
            <svg v-else width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
            导入管理员邮箱
          </button>
          <button class="btn-add" @click="openAddModal">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
            </svg>
            新增通知邮箱
          </button>
        </div>
      </div>
    </Transition>

    <!-- 通知板块设置 -->
    <Transition name="fade-up" appear>
      <div class="module-block">
        <div class="module-head">
          <div class="module-title">
            <span class="module-icon">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
            </span>
            <div>
              <h3 class="module-name">通知板块</h3>
              <p class="module-desc">全局开关，控制哪些板块默认发送通知。关闭后该板块所有邮箱将不再收到通知；开启后下方邮箱可单独关闭。</p>
            </div>
          </div>
        </div>
        <div class="module-grid">
          <div
            v-for="mod in moduleList"
            :key="mod.key"
            class="module-item"
          >
            <div class="module-item-info">
              <span class="module-item-icon" :class="'mi-' + mod.key">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M3 9h18"/><path d="M9 21V9"/></svg>
              </span>
              <div class="module-item-text">
                <span class="module-item-name">{{ mod.label }}</span>
                <span class="module-item-desc">{{ mod.desc }}</span>
              </div>
            </div>
            <button
              class="switch"
              :class="{ on: moduleSettings[mod.key] }"
              @click="toggleModule(mod.key)"
            >
              <span class="switch-knob"></span>
            </button>
          </div>
        </div>
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
                <div class="notify-modules">
                  <span
                    v-for="mod in moduleList"
                    :key="mod.key"
                    class="notify-module-tag"
                    :class="{ 'tag-on': item[mod.field] == 1, 'tag-off': item[mod.field] == 0 }"
                    @click="toggleModuleOnEmail(item, mod.key)"
                  >
                    <span class="tag-dot"></span>
                    {{ mod.label }}
                  </span>
                </div>
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
      <div v-if="addModalVisible" class="modal-backdrop">
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
            <div class="modal-field">
              <span>通知板块</span>
              <div class="modal-modules">
                <label
                  v-for="mod in moduleList"
                  :key="mod.key"
                  class="modal-module-item"
                  :class="{ on: form[mod.field] === 1 }"
                >
                  <input
                    type="checkbox"
                    v-model="form[mod.field]"
                    :true-value="1"
                    :false-value="0"
                  />
                  <span class="modal-module-check">
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><polyline points="20 6 9 17 4 12"/></svg>
                  </span>
                  <span class="modal-module-name">{{ mod.label }}</span>
                </label>
              </div>
            </div>
            <p class="modal-tip">添加后默认启用，用于接收所选板块的状态通知；也可在列表中单独调整。</p>
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

interface ModuleItem {
  key: string
  field: string
  label: string
  desc: string
}

// 通知板块定义
const moduleList: ModuleItem[] = [
  { key: 'wallpaper', field: 'notify_wallpaper', label: '壁纸审核', desc: '新壁纸提交待审核' },
  { key: 'avatar', field: 'notify_avatar', label: '头像审核', desc: '新头像提交待审核' },
  { key: 'nickname', field: 'notify_nickname', label: '昵称审核', desc: '新昵称提交待审核' },
  { key: 'feedback', field: 'notify_feedback', label: '反馈更新', desc: '用户提交新反馈' },
]

// ===== 状态 =====
const loading = ref(true)
const saving = ref(false)
const list = ref<NotifyEmail[]>([])
const moduleSettings = ref<Record<string, boolean>>({
  wallpaper: true,
  avatar: true,
  nickname: true,
  feedback: true,
})
const importing = ref(false)

const stats = computed(() => {
  const total = list.value.length
  const active = list.value.filter(i => i.status == 1).length
  return { total, active, disabled: total - active }
})

// ===== 加载数据 =====
async function loadList() {
  loading.value = true
  const [listRes, moduleRes] = await Promise.all([
    adminApi<NotifyEmail[]>('list_notification_emails'),
    adminApi<Record<string, boolean>>('get_notification_modules'),
  ])
  if (listRes.code === 200 && Array.isArray(listRes.data)) {
    list.value = listRes.data
  } else {
    list.value = []
  }
  if (moduleRes.code === 200 && moduleRes.data) {
    moduleSettings.value = { ...moduleSettings.value, ...moduleRes.data }
  }
  loading.value = false
}

// ===== 全局板块开关 =====
async function toggleModule(key: string) {
  moduleSettings.value[key] = !moduleSettings.value[key]
  const payload: Record<string, boolean> = {}
  moduleList.forEach(m => { payload[m.key] = moduleSettings.value[m.key] })
  const res = await adminApi('update_notification_modules', payload)
  if (res.code === 200) {
    showToast('已保存', 'success')
  } else {
    moduleSettings.value[key] = !moduleSettings.value[key]
    showToast(res.msg || '保存失败')
  }
}

// ===== 单邮箱板块开关 =====
async function toggleModuleOnEmail(item: NotifyEmail, key: string) {
  const field = moduleList.find(m => m.key === key)?.field || ''
  if (!field) return
  item[field] = item[field] == 1 ? 0 : 1
  const res = await adminApi('update_notification_email', {
    id: item.id,
    remark: item.remark || '',
    notify_wallpaper: item.notify_wallpaper == 1 ? 1 : 0,
    notify_avatar: item.notify_avatar == 1 ? 1 : 0,
    notify_nickname: item.notify_nickname == 1 ? 1 : 0,
    notify_feedback: item.notify_feedback == 1 ? 1 : 0,
  })
  if (res.code !== 200) {
    item[field] = item[field] == 1 ? 0 : 1
    showToast(res.msg || '操作失败')
  }
}

// ===== 导入管理员邮箱 =====
async function doImportAdmin() {
  const ok = await webConfirm('导入后，后台所有已填写邮箱的管理员账号将自动加入通知邮箱列表（已存在的会跳过）。继续吗？', {
    title: '导入管理员邮箱',
    confirmText: '确认导入',
  })
  if (!ok) return
  importing.value = true
  const res = await adminApi<{ imported: string[]; skipped: string[] }>('import_admin_emails')
  importing.value = false
  if (res.code === 200) {
    showToast(res.msg || '导入成功', 'success')
    loadList()
  } else {
    showToast(res.msg || '导入失败')
  }
}

// ===== 新增 =====
const addModalVisible = ref(false)
interface AddForm {
  email: string
  remark: string
  notify_wallpaper: number
  notify_avatar: number
  notify_nickname: number
  notify_feedback: number
  [key: string]: any
}
const form = ref<AddForm>({
  email: '',
  remark: '',
  notify_wallpaper: 1,
  notify_avatar: 1,
  notify_nickname: 1,
  notify_feedback: 1,
})

function openAddModal() {
  form.value = { email: '', remark: '', notify_wallpaper: 1, notify_avatar: 1, notify_nickname: 1, notify_feedback: 1 }
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
    notify_wallpaper: form.value.notify_wallpaper,
    notify_avatar: form.value.notify_avatar,
    notify_nickname: form.value.notify_nickname,
    notify_feedback: form.value.notify_feedback,
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

/* ===== 头部按钮组 ===== */
.header-actions {
  display: flex;
  gap: 10px;
  flex-shrink: 0;
}
.btn-ghost {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 10px 18px;
  border-radius: 10px;
  border: 1px solid var(--border);
  background: transparent;
  color: var(--text-light);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}
.btn-ghost:hover:not(:disabled) { border-color: var(--accent); color: var(--accent); background: var(--accent-soft); }
.btn-ghost:disabled { opacity: 0.6; cursor: not-allowed; }
.btn-spinner-dark {
  border-color: rgba(0, 0, 0, 0.2);
  border-top-color: var(--accent);
}

/* ===== 通知板块设置 ===== */
.module-block {
  background: var(--card, var(--white));
  border: 1px solid var(--border);
  border-radius: 16px;
  padding: 18px;
  margin-bottom: 20px;
}
.module-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
}
.module-title { display: flex; align-items: center; gap: 12px; }
.module-icon {
  width: 38px; height: 38px;
  border-radius: 10px;
  display: flex; align-items: center; justify-content: center;
  flex-shrink: 0;
  background: #eff6ff; color: #3b82f6;
}
.module-name { font-size: 15px; font-weight: 700; margin: 0; color: var(--text); }
.module-desc { font-size: 12px; color: var(--text-muted); margin: 2px 0 0; max-width: 560px; line-height: 1.5; }
.module-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: 10px;
}
.module-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 12px 14px;
  background: var(--control-bg, #f7f7f8);
  border-radius: 12px;
  transition: all 0.2s;
}
.module-item-info { display: flex; align-items: center; gap: 10px; min-width: 0; }
.module-item-icon {
  width: 32px; height: 32px;
  border-radius: 8px;
  display: flex; align-items: center; justify-content: center;
  flex-shrink: 0;
}
.mi-wallpaper { background: #f0fdf4; color: #16a34a; }
.mi-avatar { background: #eff6ff; color: #3b82f6; }
.mi-nickname { background: #fffbeb; color: #f59e0b; }
.mi-feedback { background: #fef2f2; color: #ef4444; }
.module-item-text { display: flex; flex-direction: column; gap: 1px; min-width: 0; }
.module-item-name { font-size: 13px; font-weight: 600; color: var(--text); }
.module-item-desc { font-size: 11px; color: var(--text-muted); }

/* ===== 开关 ===== */
.switch {
  width: 40px; height: 22px;
  border-radius: 12px;
  border: none;
  background: #d1d5db;
  cursor: pointer;
  position: relative;
  flex-shrink: 0;
  transition: background 0.25s cubic-bezier(0.16, 1, 0.3, 1);
  padding: 0;
}
.switch.on { background: var(--accent); }
.switch-knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 18px; height: 18px;
  border-radius: 50%;
  background: #fff;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
  transition: transform 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}
.switch.on .switch-knob { transform: translateX(18px); }

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
.notify-modules {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 8px;
}
.notify-module-tag {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 3px 9px;
  border-radius: 20px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  user-select: none;
  transition: all 0.2s;
}
.notify-module-tag .tag-dot { width: 6px; height: 6px; border-radius: 50%; }
.notify-module-tag.tag-on { background: #f0fdf4; color: #16a34a; }
.notify-module-tag.tag-on .tag-dot { background: #16a34a; }
.notify-module-tag.tag-off { background: #f3f4f6; color: #9ca3af; }
.notify-module-tag.tag-off .tag-dot { background: #9ca3af; }
.notify-module-tag:hover { transform: translateY(-1px); }
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
.modal-modules {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}
.modal-module-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 9px 11px;
  border: 1px solid var(--border);
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.2s;
  background: transparent;
}
.modal-module-item input { display: none; }
.modal-module-check {
  width: 18px; height: 18px;
  border-radius: 5px;
  border: 1.5px solid #d1d5db;
  display: flex; align-items: center; justify-content: center;
  color: transparent;
  flex-shrink: 0;
  transition: all 0.2s;
}
.modal-module-item.on .modal-module-check {
  background: var(--accent);
  border-color: var(--accent);
  color: #fff;
}
.modal-module-name { font-size: 12px; font-weight: 600; color: var(--text); }
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
.modal-enter-active, .modal-leave-active { transition: opacity 0.3s ease; }
.modal-enter-from, .modal-leave-to { opacity: 0; }
.modal-enter-active .modal-dialog { animation: modalIn 0.3s cubic-bezier(0.16, 1, 0.3, 1); }
@keyframes modalIn {
  from { opacity: 0; transform: scale(0.92) translateY(20px); }
  to { opacity: 1; transform: scale(1) translateY(0); }
}

/* ===== 响应式 ===== */
@media (max-width: 640px) {
  .stats-row { grid-template-columns: 1fr 1fr; gap: 8px; }
  .notify-main { flex-direction: column; }
  .page-header { flex-direction: column; }
  .header-actions { width: 100%; }
  .btn-add, .btn-ghost { flex: 1; justify-content: center; }
  .module-grid { grid-template-columns: 1fr; }
  .modal-modules { grid-template-columns: 1fr; }
}
</style>