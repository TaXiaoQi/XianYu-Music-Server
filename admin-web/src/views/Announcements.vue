<template>
  <div class="ann-page">
    <!-- 页面头部 -->
    <Transition name="fade-down" appear>
      <div class="page-header">
        <div class="header-info">
          <h2 class="page-title">公告管理</h2>
          <p class="page-desc">
            桌面端应用启动时自动拉取最新一条已启用的公告。新增或编辑后如需让所有用户重新看到，请确保启用状态为开启。
          </p>
        </div>
        <button class="btn-add" @click="openAddModal">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
          </svg>
          新增公告
        </button>
      </div>
    </Transition>

    <!-- 统计栏 -->
    <Transition name="fade-up" appear>
      <div class="stats-row">
        <div class="stat-chip stat-total">
          <span class="stat-num">{{ announcements.length }}</span>
          <span class="stat-label">全部</span>
        </div>
        <div class="stat-chip stat-on">
          <span class="stat-num">{{ enabledCount }}</span>
          <span class="stat-label">已启用</span>
        </div>
        <div class="stat-chip stat-off">
          <span class="stat-num">{{ disabledCount }}</span>
          <span class="stat-label">已禁用</span>
        </div>
      </div>
    </Transition>

    <!-- 加载中 -->
    <div v-if="loading" class="state-box">
      <div class="spinner"></div>
      <span>加载中...</span>
    </div>

    <!-- 加载失败 -->
    <div v-else-if="loadError" class="state-box state-error">
      <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
      <span>{{ loadError }}</span>
    </div>

    <!-- 空状态 -->
    <Transition name="fade-up" appear v-else-if="announcements.length === 0">
      <div class="state-box state-empty">
        <div class="empty-icon">
          <svg width="56" height="56" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/>
            <path d="M13.73 21a2 2 0 0 1-3.46 0"/>
          </svg>
        </div>
        <p class="empty-title">暂无公告</p>
        <p class="empty-sub">点击右上角「新增公告」创建第一条公告</p>
      </div>
    </Transition>

    <!-- 公告卡片列表 -->
    <div v-else class="card-grid">
      <TransitionGroup name="card">
        <div
          v-for="(item, idx) in announcements"
          :key="item.id"
          class="ann-card"
          :class="[`type-${item.type || 'info'}`, { disabled: !item.enabled }]"
          :style="{ animationDelay: `${idx * 60}ms` }"
        >
          <!-- 类型指示条 -->
          <div class="type-bar"></div>

          <!-- 卡片内容 -->
          <div class="card-body">
            <div class="card-top">
              <span class="type-badge" :class="`badge-${item.type || 'info'}`">
                {{ typeLabel(item.type) }}
              </span>
              <label class="toggle-switch" :title="item.enabled ? '点击禁用' : '点击启用'">
                <input type="checkbox" :checked="item.enabled" @change="toggleAnn(item.id, ($event.target as HTMLInputElement).checked)" />
                <span class="toggle-slider"></span>
              </label>
            </div>

            <h3 class="card-title">{{ item.title }}</h3>
            <p class="card-content">{{ item.content }}</p>

            <div v-if="item.actionUrl" class="card-link">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>
              <a :href="item.actionUrl" target="_blank" rel="noopener">{{ item.actionUrl }}</a>
            </div>

            <div class="card-footer">
              <span class="card-date">
                <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="4" width="18" height="18" rx="2" ry="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/></svg>
                {{ item.created_at || item.date || '-' }}
              </span>
              <div class="card-actions">
                <button class="icon-btn" title="编辑" @click="openEditModal(item)">
                  <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
                </button>
                <button class="icon-btn icon-btn-danger" title="删除" @click="deleteAnn(item.id)">
                  <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
                </button>
              </div>
            </div>
          </div>
        </div>
      </TransitionGroup>
    </div>

    <!-- 新增/编辑弹窗 -->
    <Transition name="modal">
      <div v-if="modalVisible" class="modal-backdrop" @click.self="closeModal">
        <div class="modal-dialog">
          <div class="modal-head">
            <h3>{{ editingId ? '编辑公告' : '新增公告' }}</h3>
            <button class="modal-close" @click="closeModal">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
          <div class="modal-form">
            <div class="field">
              <label class="required">标题</label>
              <input v-model="form.title" type="text" placeholder="请输入公告标题" />
            </div>
            <div class="field">
              <label class="required">正文</label>
              <textarea v-model="form.content" rows="4" placeholder="请输入公告正文"></textarea>
            </div>
            <div class="field-row">
              <div class="field">
                <label class="required">类型</label>
                <div class="type-picker">
                  <button
                    v-for="t in typeOptions"
                    :key="t.value"
                    class="type-option"
                    :class="[`pick-${t.value}`, { active: form.type === t.value }]"
                    @click="form.type = t.value"
                  >
                    <span class="pick-dot"></span>{{ t.label }}
                  </button>
                </div>
              </div>
            </div>
            <div class="field">
              <label>按钮链接 <span class="field-optional">（可空）</span></label>
              <input v-model="form.actionUrl" type="text" placeholder="https://..." />
            </div>
            <div v-if="!editingId" class="field">
              <label class="required">启用状态</label>
              <div class="type-picker">
                <button class="type-option pick-enable" :class="{ active: form.enabled }" @click="form.enabled = true">
                  <span class="pick-dot"></span>启用
                </button>
                <button class="type-option pick-disable" :class="{ active: !form.enabled }" @click="form.enabled = false">
                  <span class="pick-dot"></span>禁用
                </button>
              </div>
            </div>
          </div>
          <div class="modal-foot">
            <button class="btn-cancel" @click="closeModal">取消</button>
            <button class="btn-save" :disabled="saving" @click="save">
              <span v-if="saving" class="btn-spinner"></span>
              {{ saving ? '保存中...' : '保存' }}
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

// ===== 列表 =====
const announcements = ref<Announcement[]>([])
const loading = ref(true)
const loadError = ref('')

const enabledCount = computed(() => announcements.value.filter(a => a.enabled).length)
const disabledCount = computed(() => announcements.value.filter(a => !a.enabled).length)

async function loadList() {
  loading.value = true
  loadError.value = ''
  const res = await adminApi<Announcement[]>('list_announcements')
  if (res.code === 200 && res.data) {
    announcements.value = Array.isArray(res.data) ? res.data : []
  } else {
    loadError.value = res.msg || '加载失败'
    announcements.value = []
  }
  loading.value = false
}

// ===== 切换状态 =====
async function toggleAnn(id: string, enabled: boolean) {
  const res = await adminApi('toggle_announcement', { id, enabled: enabled ? 1 : 0 })
  if (res.code === 200) {
    showToast(enabled ? '已启用' : '已禁用', 'success')
    loadList()
  } else {
    showToast(res.msg || '操作失败')
    loadList()
  }
}

// ===== 删除 =====
async function deleteAnn(id: string) {
  const ok = await webConfirm('确认删除该公告？此操作不可恢复。', { title: '删除公告', confirmText: '确认删除' })
  if (!ok) return
  const res = await adminApi('delete_announcement', { id })
  if (res.code === 200) {
    showToast('删除成功', 'success')
    loadList()
  } else {
    showToast(res.msg || '删除失败')
  }
}

// ===== 弹窗 =====
const modalVisible = ref(false)
const editingId = ref('')
const saving = ref(false)
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

onMounted(() => {
  loadList()
})
</script>

<style scoped>
.ann-page {
  max-width: 1200px;
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
  max-width: 580px;
}

/* 新增按钮 */
.btn-add {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 10px 20px;
  border-radius: 10px;
  border: none;
  background: var(--accent);
  color: var(--white);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.12);
}
.btn-add:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(0, 0, 0, 0.2);
}
.btn-add:active { transform: scale(0.96); }

/* ===== 统计栏 ===== */
.stats-row {
  display: flex;
  gap: 12px;
  margin-bottom: 24px;
}
.stat-chip {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 10px 18px;
  border-radius: 10px;
  background: var(--white);
  border: 1px solid var(--border);
  transition: all 0.2s;
}
.stat-chip:hover { transform: translateY(-1px); box-shadow: 0 2px 8px rgba(0,0,0,0.06); }
.stat-num { font-size: 18px; font-weight: 800; }
.stat-label { font-size: 12px; color: var(--text-muted); }
.stat-total .stat-num { color: var(--text); }
.stat-on .stat-num { color: #10b981; }
.stat-off .stat-num { color: #9ca3af; }

/* ===== 加载/错误/空状态 ===== */
.state-box {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 60px 20px;
  color: var(--text-muted);
  font-size: 14px;
}
.state-error { color: #ef4444; }
.state-empty { padding: 80px 20px; }
.empty-icon {
  color: #d1d5db;
  margin-bottom: 4px;
}
.empty-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-light);
  margin: 0;
}
.empty-sub {
  font-size: 13px;
  color: var(--text-muted);
  margin: 0;
}
.spinner {
  width: 28px; height: 28px;
  border: 3px solid var(--border);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }

/* ===== 公告卡片网格 ===== */
.card-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
  gap: 16px;
}

.ann-card {
  display: flex;
  background: var(--white);
  border-radius: 14px;
  border: 1px solid var(--border);
  overflow: hidden;
  transition: transform 0.3s cubic-bezier(0.16, 1, 0.3, 1), box-shadow 0.3s ease, border-color 0.2s;
  animation: cardEnter 0.5s cubic-bezier(0.16, 1, 0.3, 1) backwards;
}
.ann-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.08);
  border-color: transparent;
}
.ann-card.disabled {
  opacity: 0.6;
}
.ann-card.disabled:hover { opacity: 0.85; }

@keyframes cardEnter {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* 类型指示条 */
.type-bar {
  width: 4px;
  flex-shrink: 0;
}
.type-info .type-bar { background: #3b82f6; }
.type-warning .type-bar { background: #f59e0b; }
.type-update .type-bar { background: #10b981; }

/* 卡片内容 */
.card-body {
  flex: 1;
  padding: 16px 18px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 0;
}
.card-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

/* 类型徽章 */
.type-badge {
  display: inline-block;
  padding: 3px 10px;
  border-radius: 20px;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.02em;
}
.badge-info { background: #eff6ff; color: #3b82f6; }
.badge-warning { background: #fffbeb; color: #f59e0b; }
.badge-update { background: #ecfdf5; color: #10b981; }

/* Toggle 开关 */
.toggle-switch {
  position: relative;
  display: inline-block;
  width: 38px;
  height: 22px;
  cursor: pointer;
  flex-shrink: 0;
}
.toggle-switch input { opacity: 0; width: 0; height: 0; }
.toggle-slider {
  position: absolute;
  inset: 0;
  background: #d1d5db;
  border-radius: 22px;
  transition: background 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}
.toggle-slider::before {
  content: '';
  position: absolute;
  width: 16px;
  height: 16px;
  left: 3px;
  top: 3px;
  background: var(--white);
  border-radius: 50%;
  box-shadow: 0 1px 3px rgba(0,0,0,0.2);
  transition: transform 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}
.toggle-switch input:checked + .toggle-slider { background: #10b981; }
.toggle-switch input:checked + .toggle-slider::before { transform: translateX(16px); }

/* 标题和正文 */
.card-title {
  font-size: 15px;
  font-weight: 700;
  margin: 0;
  color: var(--text);
  line-height: 1.4;
}
.card-content {
  font-size: 13px;
  color: var(--text-light);
  line-height: 1.6;
  margin: 0;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

/* 链接 */
.card-link {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--text-muted);
}
.card-link a {
  color: #6366f1;
  text-decoration: none;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 240px;
  transition: color 0.15s;
}
.card-link a:hover { color: #4f46e5; text-decoration: underline; }

/* 底部 */
.card-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: auto;
  padding-top: 8px;
  border-top: 1px solid #f5f5f5;
}
.card-date {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: var(--text-muted);
}
.card-actions {
  display: flex;
  gap: 4px;
}
.icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border-radius: 8px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.2s;
}
.icon-btn:hover {
  background: #f5f5f5;
  color: var(--text);
}
.icon-btn-danger:hover {
  background: #fef2f2;
  color: #ef4444;
}

/* ===== 弹窗 ===== */
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.35);
  backdrop-filter: blur(6px);
  -webkit-backdrop-filter: blur(6px);
  z-index: 9999;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
}
.modal-dialog {
  background: var(--white);
  border-radius: 18px;
  width: 100%;
  max-width: 520px;
  max-height: 90vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.16);
}
.modal-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px 16px;
}
.modal-head h3 {
  font-size: 17px;
  font-weight: 700;
  margin: 0;
}
.modal-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: 8px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.2s;
}
.modal-close:hover { background: #f5f5f5; color: var(--text); }

.modal-form {
  padding: 0 24px;
  overflow-y: auto;
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 16px;
}
.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.field label {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-light);
}
.field-optional {
  font-weight: 400;
  color: var(--text-muted);
  font-size: 12px;
}
.field input,
.field textarea {
  padding: 10px 14px;
  border: 1.5px solid var(--border);
  border-radius: 10px;
  font-size: 14px;
  outline: none;
  background: #fafafa;
  font-family: inherit;
  resize: vertical;
  transition: border-color 0.2s, background 0.2s;
}
.field input:focus,
.field textarea:focus {
  border-color: var(--accent);
  background: var(--white);
}
.field-row {
  display: flex;
  gap: 16px;
}
.field-row .field { flex: 1; }

/* 类型选择器 */
.type-picker {
  display: flex;
  gap: 8px;
}
.type-option {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border-radius: 10px;
  border: 1.5px solid var(--border);
  background: #fafafa;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-light);
  cursor: pointer;
  transition: all 0.2s;
}
.type-option:hover { border-color: var(--text-muted); }
.type-option.active {
  border-color: transparent;
  font-weight: 600;
}
.pick-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #d1d5db;
  transition: background 0.2s;
}
.pick-info.active { background: #eff6ff; color: #3b82f6; }
.pick-info.active .pick-dot { background: #3b82f6; }
.pick-warning.active { background: #fffbeb; color: #f59e0b; }
.pick-warning.active .pick-dot { background: #f59e0b; }
.pick-update.active { background: #ecfdf5; color: #10b981; }
.pick-update.active .pick-dot { background: #10b981; }
.pick-enable.active { background: #ecfdf5; color: #10b981; }
.pick-enable.active .pick-dot { background: #10b981; }
.pick-disable.active { background: #f9fafb; color: #6b7280; }
.pick-disable.active .pick-dot { background: #6b7280; }

/* 弹窗底部 */
.modal-foot {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  padding: 16px 24px 20px;
  margin-top: 8px;
}
.btn-cancel {
  padding: 10px 20px;
  border-radius: 10px;
  border: 1.5px solid var(--border);
  background: var(--white);
  font-size: 14px;
  font-weight: 500;
  color: var(--text-light);
  cursor: pointer;
  transition: all 0.2s;
}
.btn-cancel:hover { background: #f5f5f5; }
.btn-save {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 10px 24px;
  border-radius: 10px;
  border: none;
  background: var(--accent);
  color: var(--white);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}
.btn-save:hover:not(:disabled) { transform: translateY(-1px); box-shadow: 0 4px 12px rgba(0,0,0,0.15); }
.btn-save:disabled { opacity: 0.6; cursor: not-allowed; }
.btn-spinner {
  width: 14px; height: 14px;
  border: 2px solid rgba(255,255,255,0.3);
  border-top-color: var(--white);
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

/* ===== Transition 动画 ===== */
.fade-down-enter-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-down-enter-from { opacity: 0; transform: translateY(-12px); }

.fade-up-enter-active { transition: all 0.5s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-up-enter-from { opacity: 0; transform: translateY(16px); }

/* 弹窗动画 */
.modal-enter-active { transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1); }
.modal-leave-active { transition: all 0.2s ease; }
.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
.modal-enter-from .modal-dialog,
.modal-leave-to .modal-dialog {
  transform: scale(0.92) translateY(20px);
}
.modal-enter-active .modal-dialog,
.modal-leave-active .modal-dialog {
  transition: transform 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

/* 卡片列表过渡 */
.card-enter-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.card-leave-active { transition: all 0.3s ease; }
.card-enter-from { opacity: 0; transform: translateY(20px); }
.card-leave-to { opacity: 0; transform: scale(0.9); }

/* ===== 响应式 ===== */
@media (max-width: 768px) {
  .page-header { flex-direction: column; }
  .page-desc { max-width: 100%; }
  .card-grid { grid-template-columns: 1fr; }
  .stats-row { flex-wrap: wrap; }
  .field-row { flex-direction: column; }
}
</style>
