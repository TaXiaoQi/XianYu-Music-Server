<template>
  <div class="admins-page">
    <!-- 页面头部 -->
    <Transition name="fade-down" appear>
      <div class="page-header">
        <div class="header-info">
          <h2 class="page-title">管理员管理</h2>
          <p class="page-desc">管理系统后台管理员账户，新增、禁用或删除管理员。</p>
        </div>
        <button class="btn-add" @click="openAddModal">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
          </svg>
          新增管理员
        </button>
      </div>
    </Transition>

    <!-- 统计卡片 -->
    <Transition name="fade-up" appear>
      <div class="stats-row">
        <div class="stat-chip">
          <div class="stat-icon stat-icon-total">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg>
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
          <div class="stat-icon stat-icon-super">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2l3 7h7l-5.5 4.5L18 21l-6-4-6 4 1.5-7.5L2 9h7z"/></svg>
          </div>
          <div class="stat-body"><span class="stat-num">{{ stats.super_admin }}</span><span class="stat-label">超级管理员</span></div>
        </div>
        <div class="stat-chip">
          <div class="stat-icon stat-icon-disabled">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="4.93" y1="4.93" x2="19.07" y2="19.07"/></svg>
          </div>
          <div class="stat-body"><span class="stat-num">{{ stats.disabled }}</span><span class="stat-label">已禁用</span></div>
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
      <Transition name="fade-up" appear v-if="adminList.length === 0">
        <div class="state-box state-empty">
          <div class="empty-icon">
            <svg width="56" height="56" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg>
          </div>
          <p class="empty-title">暂无管理员</p>
          <p class="empty-sub">点击右上角"新增管理员"添加</p>
        </div>
      </Transition>

      <!-- 管理员卡片列表 -->
      <div v-else class="admin-grid">
        <TransitionGroup name="admin-card">
          <div
            v-for="(item, idx) in adminList"
            :key="item.id"
            class="admin-card"
            :class="{ 'is-disabled': item.status == 0, 'is-self': item.id === currentAdminId }"
            :style="{ animationDelay: `${idx * 70}ms` }"
          >
            <!-- 卡片头部 -->
            <div class="card-top">
              <div class="admin-avatar" :class="item.role === 'super_admin' ? 'avatar-super' : 'avatar-normal'">
                {{ initialOf(item.username) }}
              </div>
              <div class="admin-info">
                <div class="info-row">
                  <span class="admin-name">{{ item.username }}</span>
                  <span v-if="item.id === currentAdminId" class="self-tag">你</span>
                </div>
                <span class="admin-email">{{ item.email || '未绑定邮箱' }}</span>
              </div>
              <span class="role-badge" :class="item.role === 'super_admin' ? 'badge-super' : 'badge-admin'">
                <svg v-if="item.role === 'super_admin'" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2l3 7h7l-5.5 4.5L18 21l-6-4-6 4 1.5-7.5L2 9h7z"/></svg>
                {{ item.role === 'super_admin' ? '超级管理员' : '管理员' }}
              </span>
            </div>

            <!-- 卡片底部 -->
            <div class="card-foot">
              <div class="foot-meta">
                <span class="meta-status" :class="item.status == 1 ? 'status-on' : 'status-off'">
                  <span class="status-dot"></span>
                  {{ item.status == 1 ? '正常' : '已禁用' }}
                </span>
                <span class="meta-time">
                  <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
                  {{ item.created_at || '-' }}
                </span>
              </div>
              <div class="foot-actions">
                <button
                  v-if="item.id !== currentAdminId"
                  class="act-btn"
                  :class="item.status == 1 ? 'act-disable' : 'act-enable'"
                  @click="toggleStatus(item)"
                >
                  <svg v-if="item.status == 1" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="4.93" y1="4.93" x2="19.07" y2="19.07"/></svg>
                  <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
                  {{ item.status == 1 ? '禁用' : '启用' }}
                </button>
                <button
                  v-if="item.id !== currentAdminId"
                  class="act-btn act-delete"
                  @click="deleteAdmin(item)"
                >
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/><path d="M10 11v6M14 11v6"/></svg>
                  删除
                </button>
              </div>
            </div>
          </div>
        </TransitionGroup>
      </div>
    </template>

    <!-- 新增管理员弹窗 -->
    <Transition name="modal">
      <div v-if="addModalVisible" class="modal-backdrop" @click.self="closeAddModal">
        <div class="modal-dialog">
          <div class="modal-head">
            <h3>新增管理员</h3>
            <button class="modal-close" @click="closeAddModal">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
          <div class="modal-body">
            <div class="field">
              <label class="required">用户名</label>
              <input v-model="form.username" type="text" placeholder="请输入用户名" autocomplete="off" />
            </div>
            <div class="field">
              <label class="required">密码</label>
              <input v-model="form.password" type="password" placeholder="请输入密码" autocomplete="new-password" />
            </div>
            <div class="field">
              <label class="required">角色</label>
              <div class="role-select">
                <div
                  class="role-option"
                  :class="{ active: form.role === 'admin' }"
                  @click="form.role = 'admin'"
                >
                  <div class="role-opt-icon role-opt-admin">
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
                  </div>
                  <div class="role-opt-text">
                    <span class="role-opt-name">管理员</span>
                    <span class="role-opt-desc">常规后台管理权限</span>
                  </div>
                </div>
                <div
                  class="role-option"
                  :class="{ active: form.role === 'super_admin' }"
                  @click="form.role = 'super_admin'"
                >
                  <div class="role-opt-icon role-opt-super">
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2l3 7h7l-5.5 4.5L18 21l-6-4-6 4 1.5-7.5L2 9h7z"/></svg>
                  </div>
                  <div class="role-opt-text">
                    <span class="role-opt-name">超级管理员</span>
                    <span class="role-opt-desc">全部权限，包括管理员管理</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div class="modal-foot">
            <button class="btn-cancel" @click="closeAddModal">取消</button>
            <button class="btn-save" :disabled="saving" @click="doAdd">
              <span v-if="saving" class="btn-spinner"></span>
              {{ saving ? '提交中...' : '确认添加' }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { adminApi, showToast, getAdminUser } from '@/api/client'

interface Admin {
  id: number
  username: string
  email: string
  role: string
  status: number
  created_at: string
  updated_at: string
  [key: string]: any
}

interface AdminStats {
  total: number
  active: number
  disabled: number
  super_admin: number
  admin: number
}

// ===== 状态 =====
const loading = ref(true)
const adminList = ref<Admin[]>([])
const stats = ref<AdminStats>({ total: 0, active: 0, disabled: 0, super_admin: 0, admin: 0 })
const currentAdminId = computed(() => getAdminUser()?.id ?? 0)

// ===== 工具 =====
function initialOf(name: string): string {
  if (!name) return '?'
  return name.charAt(0).toUpperCase()
}

// ===== 加载数据 =====
async function loadList() {
  loading.value = true
  const res = await adminApi<{ list: Admin[]; stats: AdminStats }>('list_admins')
  if (res.code === 200 && res.data) {
    adminList.value = res.data.list || []
    if (res.data.stats) {
      stats.value = res.data.stats
    }
  } else {
    adminList.value = []
  }
  loading.value = false
}

// ===== 切换状态 =====
async function toggleStatus(item: Admin) {
  const action = item.status == 1 ? '禁用' : '启用'
  if (!confirm(`确认${action}管理员 "${item.username}"？`)) return
  const res = await adminApi('toggle_admin_status', { id: item.id })
  if (res.code === 200) {
    showToast(`已${action}`, 'success')
    item.status = item.status == 1 ? 0 : 1
    // 更新统计
    if (item.status == 1) {
      stats.value.active++
      stats.value.disabled--
    } else {
      stats.value.active--
      stats.value.disabled++
    }
  } else {
    showToast(res.msg || '操作失败')
  }
}

// ===== 删除 =====
async function deleteAdmin(item: Admin) {
  if (!confirm(`确认删除管理员 "${item.username}"？此操作不可恢复！`)) return
  const res = await adminApi('delete_admin', { id: item.id })
  if (res.code === 200) {
    showToast('删除成功', 'success')
    // 更新统计
    stats.value.total--
    if (item.status == 1) stats.value.active--
    else stats.value.disabled--
    if (item.role === 'super_admin') stats.value.super_admin--
    else stats.value.admin--
    // 从列表移除
    adminList.value = adminList.value.filter(a => a.id !== item.id)
  } else {
    showToast(res.msg || '删除失败')
  }
}

// ===== 新增弹窗 =====
const addModalVisible = ref(false)
const saving = ref(false)
const form = ref({ username: '', password: '', role: 'admin' })

function openAddModal() {
  form.value = { username: '', password: '', role: 'admin' }
  addModalVisible.value = true
}

function closeAddModal() {
  if (saving.value) return
  addModalVisible.value = false
}

async function doAdd() {
  if (!form.value.username.trim()) {
    showToast('请输入用户名')
    return
  }
  if (!form.value.password.trim()) {
    showToast('请输入密码')
    return
  }
  saving.value = true
  const res = await adminApi('add_admin', {
    username: form.value.username.trim(),
    password: form.value.password,
    role: form.value.role,
  })
  saving.value = false
  if (res.code === 200) {
    showToast('添加成功', 'success')
    addModalVisible.value = false
    await loadList()
  } else {
    showToast(res.msg || '添加失败')
  }
}

onMounted(() => {
  loadList()
})
</script>

<style scoped>
.admins-page {
  max-width: 960px;
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
  transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}
.btn-add:hover { transform: translateY(-1px); box-shadow: 0 6px 20px rgba(0, 0, 0, 0.15); }
.btn-add:active { transform: scale(0.96); }

/* ===== 统计卡片 ===== */
.stats-row {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
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
}
.stat-chip:hover { transform: translateY(-2px); box-shadow: 0 6px 20px rgba(0, 0, 0, 0.06); }
.stat-icon {
  width: 38px; height: 38px;
  border-radius: 10px;
  display: flex; align-items: center; justify-content: center;
  flex-shrink: 0;
}
.stat-icon-total { background: #f0f0f0; color: #1a1a1a; }
.stat-icon-active { background: #f0fdf4; color: #16a34a; }
.stat-icon-super { background: #fffbeb; color: #f59e0b; }
.stat-icon-disabled { background: #fef2f2; color: #dc2626; }
.stat-body { display: flex; flex-direction: column; }
.stat-num { font-size: 22px; font-weight: 800; line-height: 1.1; color: var(--text); }
.stat-label { font-size: 11px; color: var(--text-muted); margin-top: 2px; }

/* ===== 管理员卡片网格 ===== */
.admin-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(380px, 1fr));
  gap: 14px;
}
.admin-card {
  background: var(--white);
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 18px 20px;
  transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
  animation: cardIn 0.5s cubic-bezier(0.16, 1, 0.3, 1) both;
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.admin-card:hover { box-shadow: 0 8px 24px rgba(0, 0, 0, 0.06); transform: translateY(-2px); }
.admin-card.is-disabled { opacity: 0.65; }
.admin-card.is-self { border-color: var(--accent); box-shadow: 0 0 0 2px rgba(26, 26, 26, 0.06); }
@keyframes cardIn {
  from { opacity: 0; transform: translateY(16px); }
  to { opacity: 1; transform: translateY(0); }
}

/* 卡片头部 */
.card-top {
  display: flex;
  align-items: center;
  gap: 12px;
}
.admin-avatar {
  width: 44px; height: 44px;
  border-radius: 50%;
  display: flex; align-items: center; justify-content: center;
  font-size: 18px;
  font-weight: 800;
  flex-shrink: 0;
  color: #fff;
}
.avatar-super {
  background: linear-gradient(135deg, #f59e0b, #f97316);
}
.avatar-normal {
  background: linear-gradient(135deg, #6366f1, #818cf8);
}
.admin-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}
.info-row {
  display: flex;
  align-items: center;
  gap: 6px;
}
.admin-name {
  font-size: 15px;
  font-weight: 700;
  color: var(--text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.self-tag {
  font-size: 10px;
  font-weight: 700;
  padding: 1px 6px;
  border-radius: 4px;
  background: var(--accent);
  color: #fff;
  flex-shrink: 0;
}
.admin-email {
  font-size: 12px;
  color: var(--text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.role-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  border-radius: 20px;
  font-size: 11px;
  font-weight: 600;
  white-space: nowrap;
  flex-shrink: 0;
}
.badge-super { background: #fffbeb; color: #f59e0b; }
.badge-admin { background: #eff6ff; color: #3b82f6; }

/* 卡片底部 */
.card-foot {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
  padding-top: 12px;
  border-top: 1px solid #f0f0f5;
}
.foot-meta { display: flex; align-items: center; gap: 12px; }
.meta-status {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 12px;
  font-weight: 600;
}
.status-dot {
  width: 7px; height: 7px;
  border-radius: 50%;
}
.status-on { color: #16a34a; }
.status-on .status-dot { background: #16a34a; box-shadow: 0 0 0 3px rgba(22, 163, 74, 0.15); }
.status-off { color: #dc2626; }
.status-off .status-dot { background: #dc2626; box-shadow: 0 0 0 3px rgba(220, 38, 38, 0.15); }
.meta-time {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: var(--text-muted);
}

.foot-actions { display: flex; gap: 8px; }
.act-btn {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 7px 14px;
  border-radius: 8px;
  border: none;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}
.act-btn:active { transform: scale(0.95); }
.act-enable { background: #f0fdf4; color: #16a34a; }
.act-enable:hover { background: #dcfce7; }
.act-disable { background: #fffbeb; color: #d97706; }
.act-disable:hover { background: #fef3c7; }
.act-delete { background: #fef2f2; color: #dc2626; }
.act-delete:hover { background: #fee2e2; }

/* ===== 弹窗 ===== */
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
}
.modal-dialog {
  background: var(--white);
  border-radius: 16px;
  width: 100%;
  max-width: 460px;
  overflow: hidden;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.15);
}
.modal-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 18px 20px;
  border-bottom: 1px solid var(--border);
}
.modal-head h3 { font-size: 16px; font-weight: 700; margin: 0; }
.modal-close {
  width: 32px; height: 32px;
  border: none;
  background: #f5f5f5;
  border-radius: 8px;
  cursor: pointer;
  display: flex; align-items: center; justify-content: center;
  color: var(--text-muted);
  transition: all 0.2s;
}
.modal-close:hover { background: #e5e5e5; color: var(--text); }
.modal-body { padding: 20px; }
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
}
.field input:focus { border-color: var(--accent); }

/* 角色选择 */
.role-select {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.role-option {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 14px;
  border: 2px solid var(--border);
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.2s;
}
.role-option:hover { border-color: #d0d0d8; }
.role-option.active { border-color: var(--accent); background: #f8f9fc; }
.role-opt-icon {
  width: 36px; height: 36px;
  border-radius: 10px;
  display: flex; align-items: center; justify-content: center;
  flex-shrink: 0;
}
.role-opt-admin { background: #eff6ff; color: #3b82f6; }
.role-opt-super { background: #fffbeb; color: #f59e0b; }
.role-opt-text { display: flex; flex-direction: column; gap: 2px; }
.role-opt-name { font-size: 14px; font-weight: 600; color: var(--text); }
.role-opt-desc { font-size: 11px; color: var(--text-muted); }

.modal-foot {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  padding: 16px 20px;
  border-top: 1px solid var(--border);
}
.btn-cancel {
  padding: 9px 18px;
  border-radius: 10px;
  border: 1px solid var(--border);
  background: var(--white);
  color: var(--text-light);
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}
.btn-cancel:hover { background: #f5f5f5; }
.btn-save {
  padding: 9px 20px;
  border-radius: 10px;
  border: none;
  background: var(--accent);
  color: #fff;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  transition: all 0.2s;
}
.btn-save:hover { opacity: 0.85; }
.btn-save:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-spinner {
  width: 14px; height: 14px;
  border: 2px solid rgba(255,255,255,0.3);
  border-top-color: #fff;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

/* ===== 空状态 / 加载 ===== */
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
.state-empty { padding: 48px 20px; }
.empty-icon { color: #d0d0d0; margin-bottom: 4px; }
.empty-title { font-size: 15px; font-weight: 600; color: var(--text-light); margin: 0; }
.empty-sub { font-size: 13px; color: var(--text-muted); margin: 0; }

/* ===== Spinner ===== */
.spinner {
  width: 32px; height: 32px;
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

.admin-card-enter-active { transition: all 0.5s cubic-bezier(0.16, 1, 0.3, 1); }
.admin-card-enter-from { opacity: 0; transform: translateY(16px); }
.admin-card-leave-active { transition: all 0.3s ease; }
.admin-card-leave-to { opacity: 0; transform: scale(0.95); }
.admin-card-move { transition: transform 0.4s cubic-bezier(0.16, 1, 0.3, 1); }

.modal-enter-active, .modal-leave-active { transition: opacity 0.3s ease; }
.modal-enter-from, .modal-leave-to { opacity: 0; }
.modal-enter-active .modal-dialog { animation: modalIn 0.3s cubic-bezier(0.16, 1, 0.3, 1); }
@keyframes modalIn {
  from { opacity: 0; transform: scale(0.92) translateY(20px); }
  to { opacity: 1; transform: scale(1) translateY(0); }
}

/* ===== 响应式 ===== */
@media (max-width: 768px) {
  .stats-row { grid-template-columns: 1fr 1fr; gap: 8px; }
  .stat-chip { padding: 10px 12px; flex-direction: column; align-items: flex-start; gap: 6px; }
  .stat-num { font-size: 18px; }
  .stat-label { font-size: 10px; }
  .admin-grid { grid-template-columns: 1fr; }
  .card-foot { flex-direction: column; align-items: stretch; }
  .foot-actions { justify-content: flex-end; }
}
</style>
