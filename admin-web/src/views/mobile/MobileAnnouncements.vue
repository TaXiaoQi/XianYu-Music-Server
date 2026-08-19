<template>
  <div class="mobile-page">
    <!-- 页头 -->
    <div class="ann-header">
      <div class="ann-header-info">
        <h2 class="ann-title">公告管理</h2>
        <p class="ann-desc">应用启动时自动拉取最新一条已启用的公告。新增或编辑后如需让所有用户重新看到，请确保启用状态为开启。</p>
      </div>
      <button class="mobile-btn primary ann-add" @click="openAddModal">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
        新增公告
      </button>
    </div>

    <!-- 统计 -->
    <div class="mobile-grid" style="grid-template-columns: repeat(3, minmax(0, 1fr));">
      <div class="mobile-stat">
        <div class="stat-icon-row">
          <span class="stat-icon stat-icon-total">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg>
          </span>
          <strong>{{ list.length }}</strong>
        </div>
        <span class="stat-label">全部</span>
      </div>
      <div class="mobile-stat">
        <div class="stat-icon-row">
          <span class="stat-icon stat-icon-on">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
          </span>
          <strong>{{ enabledCount }}</strong>
        </div>
        <span class="stat-label">已启用</span>
      </div>
      <div class="mobile-stat">
        <div class="stat-icon-row">
          <span class="stat-icon stat-icon-off">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>
          </span>
          <strong>{{ disabledCount }}</strong>
        </div>
        <span class="stat-label">已禁用</span>
      </div>
    </div>

    <!-- 加载中 -->
    <div v-if="loading" class="mobile-empty">加载中...</div>

    <!-- 空状态 -->
    <div v-else-if="list.length === 0" class="mobile-empty">
      <div class="empty-title">暂无公告</div>
      <div class="empty-sub">点击上方「新增公告」创建第一条公告</div>
    </div>

    <!-- 公告列表 -->
    <div v-else class="ann-list">
      <div
        v-for="(item, idx) in list"
        :key="item.id"
        class="ann-card"
        :class="['type-' + (item.type || 'info'), { 'is-disabled': !item.enabled }]"
        :style="{ animationDelay: (idx * 0.05) + 's' }"
      >
        <!-- 类型指示条 -->
        <div class="type-bar"></div>
        <div class="ann-body">
          <div class="ann-top">
            <span class="type-badge" :class="'badge-' + (item.type || 'info')">{{ typeLabel(item.type) }}</span>
            <label class="switch" :title="item.enabled ? '点击禁用' : '点击启用'">
              <input :checked="item.enabled" type="checkbox" @change="toggle(item, ($event.target as HTMLInputElement).checked)" />
              <span class="track"><span class="thumb"></span></span>
            </label>
          </div>
          <h3 class="ann-title-text">{{ item.title }}</h3>
          <p class="ann-content">{{ item.content }}</p>
          <div v-if="item.actionUrl" class="ann-link">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>
            <a :href="item.actionUrl" target="_blank" rel="noopener">{{ item.actionUrl }}</a>
          </div>
          <div class="ann-footer">
            <span class="ann-date">
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="4" width="18" height="18" rx="2" ry="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/></svg>
              {{ fmtDateTime(item.created_at || item.date) || '-' }}
            </span>
            <div class="ann-actions">
              <button class="mobile-btn" @click="openEditModal(item)">编辑</button>
              <button class="mobile-btn danger" @click="remove(item)">删除</button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 新增/编辑弹窗 -->
    <Transition name="modal" @before-leave="removeBackdropBlur">
      <div v-if="modalVisible" class="modal-backdrop">
        <div class="modal-dialog">
          <div class="modal-head">
            <h3>{{ editingId ? '编辑公告' : '新增公告' }}</h3>
            <button class="modal-close" @click="closeModal">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
          <div class="modal-body">
            <label class="modal-field">
              <span class="required">标题</span>
              <input v-model="form.title" type="text" placeholder="请输入公告标题" />
            </label>
            <label class="modal-field">
              <span class="required">正文</span>
              <textarea v-model="form.content" rows="4" placeholder="请输入公告正文"></textarea>
            </label>
            <div class="modal-field">
              <span class="required">类型</span>
              <div class="type-picker">
                <button
                  v-for="t in typeOptions"
                  :key="t.value"
                  class="type-option"
                  :class="['pick-' + t.value, { active: form.type === t.value }]"
                  @click="form.type = t.value"
                >
                  <span class="pick-dot"></span>{{ t.label }}
                </button>
              </div>
            </div>
            <label class="modal-field">
              <span>按钮链接 <em class="optional">（可空）</em></span>
              <input v-model="form.actionUrl" type="text" placeholder="https://..." />
            </label>
            <div v-if="!editingId" class="modal-field">
              <span class="required">启用状态</span>
              <div class="type-picker">
                <button class="type-option pick-enable" :class="{ active: form.enabled }" @click="form.enabled = true"><span class="pick-dot"></span>启用</button>
                <button class="type-option pick-disable" :class="{ active: !form.enabled }" @click="form.enabled = false"><span class="pick-dot"></span>禁用</button>
              </div>
            </div>
          </div>
          <div class="modal-foot">
            <button class="modal-btn cancel" @click="closeModal">取消</button>
            <button class="modal-btn save" :disabled="saving" @click="save">{{ saving ? '保存中...' : '保存' }}</button>
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
import { mobileConfirm, removeBackdropBlur } from '@/utils/mobileDialog'
import { fmtDateTime } from '@/utils/time'

interface Announcement {
  id: string
  title: string
  content: string
  type: string
  date: string
  actionUrl: string
  actionText: string
  enabled: boolean
  created_at: string
  updated_at: string
  [key: string]: any
}

const typeOptions = [
  { value: 'info', label: '通知' },
  { value: 'warning', label: '警告' },
  { value: 'update', label: '更新' },
]

function typeLabel(t: string | undefined): string {
  return typeOptions.find(o => o.value === t)?.label || '通知'
}

const loading = ref(true)
const saving = ref(false)
const list = ref<Announcement[]>([])
const loadError = ref('')

const enabledCount = computed(() => list.value.filter(a => a.enabled).length)
const disabledCount = computed(() => list.value.filter(a => !a.enabled).length)

async function loadList() {
  loading.value = true
  loadError.value = ''
  const res = await adminApi<Announcement[]>('list_announcements')
  if (res.code === 200 && res.data) {
    list.value = Array.isArray(res.data) ? res.data : []
  } else {
    loadError.value = res.msg || '加载失败'
    list.value = []
  }
  loading.value = false
}

async function toggle(item: Announcement, enabled: boolean) {
  const res = await adminApi('toggle_announcement', { id: item.id, enabled: enabled ? 1 : 0 })
  if (res.code === 200) {
    item.enabled = enabled
    showToast(enabled ? '已启用' : '已禁用', 'success')
  } else {
    showToast(res.msg || '操作失败')
  }
}

// ===== 弹窗 =====
const modalVisible = ref(false)
const editingId = ref('')
const form = ref({
  title: '',
  content: '',
  type: 'info',
  actionUrl: '',
  enabled: true,
})

function openAddModal() {
  editingId.value = ''
  form.value = { title: '', content: '', type: 'info', actionUrl: '', enabled: true }
  modalVisible.value = true
}

function openEditModal(item: Announcement) {
  editingId.value = item.id
  form.value = {
    title: item.title || '',
    content: item.content || '',
    type: item.type || 'info',
    actionUrl: item.actionUrl || '',
    enabled: item.enabled,
  }
  modalVisible.value = true
}

function closeModal() {
  if (saving.value) return
  modalVisible.value = false
}

async function save() {
  if (!form.value.title.trim() || !form.value.content.trim()) {
    showToast('请填写标题和正文')
    return
  }
  saving.value = true
  const payload: Record<string, any> = {
    title: form.value.title.trim(),
    content: form.value.content.trim(),
    type: form.value.type,
    action_url: form.value.actionUrl.trim(),
  }

  let res
  if (editingId.value) {
    payload.id = editingId.value
    res = await adminApi('update_announcement', payload)
  } else {
    payload.enabled = form.value.enabled ? 1 : 0
    res = await adminApi('add_announcement', payload)
  }
  saving.value = false

  if (res.code === 200) {
    showToast(editingId.value ? '修改成功' : '添加成功', 'success')
    closeModal()
    loadList()
  } else {
    showToast(res.msg || '保存失败')
  }
}

async function remove(item: Announcement) {
  const ok = await mobileConfirm('确认删除该公告？此操作不可恢复。', { title: '删除公告', confirmText: '确认删除', danger: true })
  if (!ok) return
  const res = await adminApi('delete_announcement', { id: item.id })
  if (res.code === 200) {
    showToast('删除成功', 'success')
    loadList()
  } else {
    showToast(res.msg || '删除失败')
  }
}

onMounted(loadList)
</script>

<style scoped>
.ann-header {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.ann-header-info { min-width: 0; }
.ann-title { font-size: 18px; font-weight: 850; margin: 0 0 4px; color: var(--text); }
.ann-desc { font-size: 12px; color: var(--text-light); line-height: 1.6; margin: 0; }
.ann-add {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  align-self: flex-start;
  padding: 9px 16px;
}

/* 统计（复用共享 .mobile-grid/.mobile-stat，内联3列） */

/* 空状态 */
.empty-title { font-size: 14px; font-weight: 700; color: var(--text-light); margin-bottom: 4px; }
.empty-sub { font-size: 12px; color: var(--text-muted); }

/* 公告列表 */
.ann-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.ann-card {
  display: flex;
  border: 1px solid var(--border);
  border-radius: 18px;
  background: var(--card);
  box-shadow: var(--shadow-soft);
  overflow: hidden;
  animation: annCardIn 0.4s var(--motion, cubic-bezier(0.16, 1, 0.3, 1)) both;
}
.ann-card.is-disabled { opacity: 0.6; }
@keyframes annCardIn {
  from { opacity: 0; transform: translateY(12px); }
  to { opacity: 1; transform: translateY(0); }
}
.type-bar { width: 5px; flex-shrink: 0; }
.type-info .type-bar { background: #3b82f6; }
.type-warning .type-bar { background: #f59e0b; }
.type-update .type-bar { background: #10b981; }

.ann-body {
  flex: 1;
  min-width: 0;
  padding: 14px 15px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.ann-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 10px;
}
.type-badge {
  display: inline-block;
  padding: 3px 10px;
  border-radius: 20px;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.02em;
}
.badge-info { background: #eff6ff; color: #3b82f6; }
.badge-warning { background: rgba(245, 158, 11, 0.14); color: #f59e0b; }
.badge-update { background: #ecfdf5; color: #10b981; }

/* 开关 */
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
  background: var(--card-solid);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
  transition: transform 0.25s;
}
.switch input:checked + .track { background: #EC4141; }
.switch input:checked + .track .thumb { transform: translateX(18px); }

.ann-title-text {
  font-size: 15px;
  font-weight: 750;
  margin: 0;
  color: var(--text);
  line-height: 1.4;
  word-break: break-word;
}
.ann-content {
  font-size: 13px;
  color: var(--text-light);
  line-height: 1.6;
  margin: 0;
  word-break: break-word;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
.ann-link {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--text-muted);
}
.ann-link a {
  color: #6366f1;
  text-decoration: none;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 220px;
}
.ann-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 10px;
  margin-top: auto;
  padding-top: 10px;
  border-top: 1px solid var(--border);
}
.ann-date {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: var(--text-muted);
  min-width: 0;
}
.ann-actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

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
  max-height: 88vh;
  border-radius: 22px;
  background: var(--card-solid, var(--card));
  box-shadow: 0 24px 60px rgba(15, 23, 42, 0.22);
  overflow: hidden;
  display: flex;
  flex-direction: column;
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
  overflow-y: auto;
}
.modal-field { display: flex; flex-direction: column; gap: 6px; }
.modal-field > span { font-size: 12px; font-weight: 700; color: var(--text-light); }
.modal-field > span.required::after { content: '*'; color: #ef4444; margin-left: 2px; }
.modal-field .optional { font-style: normal; font-weight: 400; color: var(--text-muted); font-size: 11px; }
.modal-field input,
.modal-field textarea {
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 11px 12px;
  font-size: 14px;
  font-family: inherit;
  outline: none;
  background: var(--control-bg);
  color: var(--text);
  resize: vertical;
  transition: border-color 0.18s, box-shadow 0.18s;
  width: 100%;
  box-sizing: border-box;
}
.modal-field input:focus,
.modal-field textarea:focus { border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-soft); }

/* 类型选择器 */
.type-picker {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}
.type-option {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 13px;
  border-radius: 10px;
  border: 1.5px solid var(--border);
  background: transparent;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-light);
  cursor: pointer;
  transition: all 0.18s;
}
.type-option.active { font-weight: 700; }
.pick-dot { width: 8px; height: 8px; border-radius: 50%; background: var(--border); transition: background 0.18s; }
.pick-info.active { background: #eff6ff; color: #3b82f6; border-color: transparent; }
.pick-info.active .pick-dot { background: #3b82f6; }
.pick-warning.active { background: rgba(245, 158, 11, 0.14); color: #f59e0b; border-color: transparent; }
.pick-warning.active .pick-dot { background: #f59e0b; }
.pick-update.active { background: #ecfdf5; color: #10b981; border-color: transparent; }
.pick-update.active .pick-dot { background: #10b981; }
.pick-enable.active { background: #ecfdf5; color: #10b981; border-color: transparent; }
.pick-enable.active .pick-dot { background: #10b981; }
.pick-disable.active { background: var(--control-bg); color: var(--text-muted); border-color: transparent; }
.pick-disable.active .pick-dot { background: #6b7280; }

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
.modal-enter-active, .modal-leave-active { transition: opacity 0.24s var(--motion, cubic-bezier(0.16, 1, 0.3, 1)); }
.modal-enter-from, .modal-leave-to { opacity: 0; }
.modal-enter-active .modal-dialog { animation: modalIn 0.24s cubic-bezier(0.16, 1, 0.3, 1) forwards; }
.modal-leave-active .modal-dialog { animation: modalOut 0.2s ease forwards; }
@keyframes modalIn {
  from { opacity: 0; transform: scale(0.94); }
  to { opacity: 1; transform: scale(1); }
}
@keyframes modalOut {
  from { opacity: 1; transform: scale(1); }
  to { opacity: 0; transform: scale(0.96); }
}

@media (prefers-reduced-motion: reduce) {
  .ann-card { animation: none !important; }
}
</style>