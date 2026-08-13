<template>
  <div class="mobile-page">
    <!-- 页头 -->
    <div class="notify-header">
      <div class="notify-header-info">
        <h2 class="notify-title">外部通知</h2>
        <p class="notify-desc">管理用于接收后台状态通知的绑定邮箱，并可分别设置壁纸审核、头像审核、昵称审核、反馈更新等板块的通知开关。</p>
      </div>
      <div class="notify-header-actions">
        <button class="mobile-btn" :disabled="importing" @click="doImportAdmin">{{ importing ? '导入中...' : '导入管理员邮箱' }}</button>
        <button class="mobile-btn primary" @click="openAddModal">+ 新增</button>
      </div>
    </div>

    <!-- 通知板块设置 -->
    <section class="mobile-card">
      <div class="module-head">
        <div class="module-icon">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
        </div>
        <div class="module-text">
          <h3 class="module-name">通知板块</h3>
          <p class="module-desc">全局开关，控制哪些板块默认发送通知。关闭后该板块所有邮箱将不再收到通知；开启后下方邮箱可单独关闭。</p>
        </div>
      </div>
      <div class="module-list">
        <div v-for="mod in moduleList" :key="mod.key" class="module-item">
          <div class="module-item-icon" :class="'mi-' + mod.key">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M3 9h18"/><path d="M9 21V9"/></svg>
          </div>
          <div class="module-item-text">
            <span class="module-item-name">{{ mod.label }}</span>
            <span class="module-item-desc">{{ mod.desc }}</span>
          </div>
          <label class="switch">
            <input :checked="moduleSettings[mod.key] === true" type="checkbox" @change="toggleModule(mod.key)" />
            <span class="track"><span class="thumb"></span></span>
          </label>
        </div>
      </div>
    </section>

    <!-- 统计卡片 -->
    <div class="stats-grid">
      <div class="stat-chip">
        <div class="stat-icon stat-icon-total">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="4" width="20" height="16" rx="2"/><path d="M22 7l-10 6L2 7"/></svg>
        </div>
        <div class="stat-body"><span class="stat-num">{{ stats.total }}</span><span class="stat-label">总数</span></div>
      </div>
      <div class="stat-chip">
        <div class="stat-icon stat-icon-active">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
        </div>
        <div class="stat-body"><span class="stat-num">{{ stats.active }}</span><span class="stat-label">启用中</span></div>
      </div>
      <div class="stat-chip">
        <div class="stat-icon stat-icon-disabled">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="4.93" y1="4.93" x2="19.07" y2="19.07"/></svg>
        </div>
        <div class="stat-body"><span class="stat-num">{{ stats.disabled }}</span><span class="stat-label">已停用</span></div>
      </div>
    </div>

    <!-- 加载中 -->
    <div v-if="loading" class="mobile-empty">加载中...</div>

    <!-- 空状态 -->
    <div v-else-if="list.length === 0" class="mobile-empty">
      <div class="empty-title">暂无通知邮箱</div>
      <div class="empty-sub">点击上方"新增"添加，用于接收后台状态通知</div>
    </div>

    <!-- 通知邮箱列表 -->
    <div v-else class="mobile-list">
      <div v-for="item in list" :key="item.id" class="mobile-item" :class="{ 'is-disabled': item.status == 0 }">
        <div class="mobile-item-head">
          <div class="mobile-admin-left">
            <div class="notify-avatar" :class="item.status == 1 ? 'avatar-active' : 'avatar-disabled'">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="4" width="20" height="16" rx="2"/><path d="M22 7l-10 6L2 7"/></svg>
            </div>
            <div>
              <div class="mobile-item-title">{{ item.email }}</div>
              <div class="mobile-item-sub">{{ item.remark || '暂无备注' }}</div>
              <div class="mobile-item-sub muted-time">创建时间：{{ item.created_at || '-' }}</div>
            </div>
          </div>
          <span class="mobile-badge" :class="item.status == 1 ? 'green' : 'red'">{{ item.status == 1 ? '启用中' : '已停用' }}</span>
        </div>
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
        <div class="mobile-actions">
          <button class="mobile-btn" :disabled="testingId === item.id" @click="sendTest(item)">{{ testingId === item.id ? '发送中...' : '发送测试' }}</button>
          <button class="mobile-btn" @click="toggle(item)">{{ item.status == 1 ? '停用' : '启用' }}</button>
          <button class="mobile-btn danger" @click="remove(item)">删除</button>
        </div>
      </div>
    </div>

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
                  <input type="checkbox" v-model="form[mod.field]" :true-value="1" :false-value="0" />
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
            <button class="modal-btn cancel" @click="closeAddModal">取消</button>
            <button class="modal-btn save" :disabled="saving" @click="doAdd">{{ saving ? '添加中...' : '确认添加' }}</button>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'
import './MobilePage.css'
import { mobileConfirm } from '@/utils/mobileDialog'

interface ModuleItem {
  key: string
  field: string
  label: string
  desc: string
}

const moduleList: ModuleItem[] = [
  { key: 'wallpaper', field: 'notify_wallpaper', label: '壁纸审核', desc: '新壁纸提交待审核' },
  { key: 'avatar', field: 'notify_avatar', label: '头像审核', desc: '新头像提交待审核' },
  { key: 'nickname', field: 'notify_nickname', label: '昵称审核', desc: '新昵称提交待审核' },
  { key: 'feedback', field: 'notify_feedback', label: '反馈更新', desc: '用户提交新反馈' },
]

const loading = ref(true)
const saving = ref(false)
const importing = ref(false)
const testingId = ref<number | null>(null)
const list = ref<any[]>([])
const moduleSettings = ref<Record<string, boolean>>({ wallpaper: true, avatar: true, nickname: true, feedback: true })

const stats = computed(() => {
  const total = list.value.length
  const active = list.value.filter(i => i.status == 1).length
  return { total, active, disabled: total - active }
})

async function loadList() {
  loading.value = true
  const [listRes, moduleRes] = await Promise.all([
    adminApi<any[]>('list_notification_emails'),
    adminApi<Record<string, boolean>>('get_notification_modules'),
  ])
  list.value = listRes.code === 200 && Array.isArray(listRes.data) ? listRes.data : []
  if (moduleRes.code === 200 && moduleRes.data) {
    moduleSettings.value = { ...moduleSettings.value, ...moduleRes.data }
  }
  loading.value = false
}

async function toggleModule(key: string) {
  moduleSettings.value[key] = !moduleSettings.value[key]
  const payload: Record<string, boolean> = {}
  moduleList.forEach(m => { payload[m.key] = moduleSettings.value[m.key] })
  const res = await adminApi('update_notification_modules', payload)
  if (res.code !== 200) {
    moduleSettings.value[key] = !moduleSettings.value[key]
    showToast(res.msg || '保存失败')
  }
}

async function toggleModuleOnEmail(item: any, key: string) {
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

async function doImportAdmin() {
  const ok = await mobileConfirm('导入后，后台所有已填写邮箱的管理员账号将自动加入通知邮箱列表（已存在的会跳过）。继续吗？', {
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

const addModalVisible = ref(false)
const form = ref<any>({ email: '', remark: '', notify_wallpaper: 1, notify_avatar: 1, notify_nickname: 1, notify_feedback: 1 })

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
  if (!email) { showToast('请输入邮箱地址'); return }
  if (!isValidEmail(email)) { showToast('邮箱格式不正确'); return }
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

async function toggle(item: any) {
  const res = await adminApi('toggle_notification_email', { id: item.id })
  if (res.code === 200) {
    item.status = item.status == 1 ? 0 : 1
    showToast('已更新', 'success')
  } else {
    showToast(res.msg || '操作失败')
  }
}

async function remove(item: any) {
  const ok = await mobileConfirm(`确认删除通知邮箱 "${item.email}"？`, {
    title: '删除通知邮箱',
    confirmText: '确认删除',
    danger: true,
  })
  if (!ok) return
  const res = await adminApi('delete_notification_email', { id: item.id })
  if (res.code === 200) {
    showToast('删除成功', 'success')
    list.value = list.value.filter(i => i.id !== item.id)
  } else {
    showToast(res.msg || '删除失败')
  }
}

async function sendTest(item: any) {
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
.notify-header {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.notify-header-info { min-width: 0; }
.notify-title { font-size: 18px; font-weight: 850; margin: 0 0 4px; color: var(--text); }
.notify-desc { font-size: 12px; color: var(--text-light); line-height: 1.6; margin: 0; }
.notify-header-actions {
  display: grid;
  grid-template-columns: 1.4fr 1fr;
  gap: 8px;
}

/* 通知板块 */
.module-head {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  margin-bottom: 12px;
}
.module-icon {
  width: 36px;
  height: 36px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  background: #eff6ff;
  color: #3b82f6;
}
.module-text { min-width: 0; }
.module-name { font-size: 15px; font-weight: 750; margin: 0; color: var(--text); }
.module-desc { font-size: 11px; color: var(--text-muted); line-height: 1.5; margin: 2px 0 0; }
.module-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.module-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  background: var(--control-bg);
  border-radius: 12px;
}
.module-item-icon {
  width: 30px;
  height: 30px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.mi-wallpaper { background: #f0fdf4; color: #16a34a; }
.mi-avatar { background: #eff6ff; color: #3b82f6; }
.mi-nickname { background: #fffbeb; color: #f59e0b; }
.mi-feedback { background: #fef2f2; color: #ef4444; }
.module-item-text {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
  flex: 1;
}
.module-item-name { font-size: 13px; font-weight: 650; color: var(--text); }
.module-item-desc { font-size: 11px; color: var(--text-muted); }

.switch { position: relative; display: inline-flex; flex-shrink: 0; cursor: pointer; }
.switch input { display: none; }
.switch .track {
  width: 40px;
  height: 22px;
  border-radius: 999px;
  background: var(--border);
  position: relative;
  transition: background 0.25s;
}
.switch .thumb {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: var(--white);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
  transition: transform 0.25s;
}
.switch input:checked + .track { background: #EC4141; }
.switch input:checked + .track .thumb { transform: translateX(18px); }

/* 统计 */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 10px;
}
.stat-chip {
  border: 1px solid var(--border);
  border-radius: 16px;
  background: var(--card);
  padding: 12px;
  display: flex;
  align-items: center;
  gap: 10px;
  box-shadow: var(--shadow-soft);
}
.stat-icon {
  width: 34px;
  height: 34px;
  border-radius: 9px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.stat-icon-total { background: #eff6ff; color: #3b82f6; }
.stat-icon-active { background: #f0fdf4; color: #16a34a; }
.stat-icon-disabled { background: #fef2f2; color: #ef4444; }
.stat-body { display: flex; flex-direction: column; min-width: 0; }
.stat-label { font-size: 10px; color: var(--text-muted); }
.stat-num { font-size: 18px; font-weight: 850; line-height: 1.2; color: var(--text); }

/* 列表 */
.mobile-item.is-disabled { opacity: 0.6; }
.mobile-admin-left {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}
.notify-avatar {
  width: 40px;
  height: 40px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.avatar-active { background: #eff6ff; color: #3b82f6; }
.avatar-disabled { background: #f3f4f6; color: #9ca3af; }

.notify-modules {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 10px;
}
.notify-module-tag {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 4px 10px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
  user-select: none;
  transition: transform 0.15s;
}
.notify-module-tag:active { transform: scale(0.94); }
.notify-module-tag .tag-dot { width: 6px; height: 6px; border-radius: 50%; }
.notify-module-tag.tag-on { background: rgba(34, 197, 94, 0.12); color: #16a34a; }
.notify-module-tag.tag-on .tag-dot { background: #16a34a; }
.notify-module-tag.tag-off { background: var(--control-bg); color: var(--text-muted); }
.notify-module-tag.tag-off .tag-dot { background: var(--text-muted); }

.empty-title { font-size: 14px; font-weight: 700; color: var(--text-light); margin-bottom: 4px; }
.empty-sub { font-size: 12px; color: var(--text-muted); }

/* 弹窗 */
.modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 32px 24px;
  background: rgba(15, 23, 42, 0.38);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
}
.modal-dialog {
  width: 100%;
  max-width: 340px;
  border-radius: 22px;
  background: var(--card-solid, var(--card));
  box-shadow: 0 24px 60px rgba(15, 23, 42, 0.22);
  overflow: hidden;
}
.modal-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 18px 20px 0;
}
.modal-head h3 { margin: 0; font-size: 16px; font-weight: 850; color: var(--text); }
.modal-close {
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  padding: 4px;
  border-radius: 8px;
  display: flex;
}
.modal-close:active { background: var(--control-bg); color: var(--text); }
.modal-body {
  padding: 14px 20px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.modal-field { display: flex; flex-direction: column; gap: 6px; }
.modal-field span { font-size: 12px; font-weight: 700; color: var(--text-light); }
.modal-field span.required::after { content: '*'; color: #ef4444; margin-left: 2px; }
.modal-field input {
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 11px 12px;
  font-size: 14px;
  font-family: inherit;
  outline: none;
  background: var(--control-bg);
  color: var(--text);
  transition: border-color 0.18s, box-shadow 0.18s;
  width: 100%;
  box-sizing: border-box;
}
.modal-field input:focus { border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-soft); }
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
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.18s;
  background: transparent;
}
.modal-module-item input { display: none; }
.modal-module-check {
  width: 18px;
  height: 18px;
  border-radius: 5px;
  border: 1.5px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: center;
  color: transparent;
  flex-shrink: 0;
  transition: all 0.18s;
}
.modal-module-item.on .modal-module-check {
  background: #EC4141;
  border-color: #EC4141;
  color: #fff;
}
.modal-module-name { font-size: 12px; font-weight: 650; color: var(--text); }
.modal-tip { margin: 0; font-size: 11px; color: var(--text-muted); line-height: 1.5; }
.modal-foot {
  display: flex;
  gap: 10px;
  padding: 14px 20px 18px;
}
.modal-btn {
  flex: 1;
  padding: 11px;
  border-radius: 12px;
  font-size: 14px;
  font-weight: 750;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  transition: all 0.18s;
}
.modal-btn.cancel { border: 1px solid var(--border); background: transparent; color: var(--text-muted); }
.modal-btn.cancel:active { background: var(--control-bg); }
.modal-btn.save { border: none; background: #EC4141; color: #fff; }
.modal-btn.save:disabled { opacity: 0.55; }

/* 过渡动画 */
.modal-enter-active, .modal-leave-active { transition: opacity 0.26s var(--motion, cubic-bezier(0.16, 1, 0.3, 1)); }
.modal-enter-from, .modal-leave-to { opacity: 0; }
.modal-enter-active .modal-dialog { animation: modalIn 0.28s cubic-bezier(0.16, 1, 0.3, 1); }
.modal-leave-active .modal-dialog { animation: modalOut 0.2s ease; }
@keyframes modalIn {
  from { opacity: 0; transform: scale(0.92) translateY(20px); }
  to { opacity: 1; transform: scale(1) translateY(0); }
}
@keyframes modalOut {
  from { opacity: 1; transform: scale(1) translateY(0); }
  to { opacity: 0; transform: scale(0.95) translateY(10px); }
}
</style>