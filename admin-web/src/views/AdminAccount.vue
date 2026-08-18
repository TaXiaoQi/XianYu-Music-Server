<template>
  <div class="admin-account-page">
    <!-- 页面头部 -->
    <Transition name="fade-down" appear>
      <div class="page-header">
        <div class="header-info">
          <h2 class="page-title">后台账号</h2>
          <p class="page-desc">管理系统后台管理员账户，支持新增、禁用、删除管理员，上传头像，以及修改登录用户名与密码。</p>
        </div>
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

    <!-- 管理员列表区块 -->
    <section class="section-block">
      <div class="section-head">
        <div class="section-title">
          <span class="section-icon section-icon-admin">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
          </span>
          <div>
            <h3 class="section-name">管理员账户</h3>
            <p class="section-desc">{{ isSuper ? '管理所有后台管理员账户，超管全局仅可有一个' : '查看后台管理员账户信息' }}</p>
          </div>
        </div>
        <button v-if="isSuper" class="btn-add" @click="openAddModal">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
          </svg>
          新增管理员
        </button>
      </div>

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
                <div class="avatar-wrap">
                  <img v-if="item.avatar_url" :src="item.avatar_url" alt="" class="admin-avatar-img" />
                  <div v-else class="admin-avatar" :class="item.role === 'super_admin' ? 'avatar-super' : 'avatar-normal'">
                    {{ initialOf(item.username) }}
                  </div>
                  <button
                    v-if="canUploadAvatar(item)"
                    class="avatar-upload-btn"
                    title="上传头像"
                    @click="openAvatarModal(item)"
                  >
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z"/><circle cx="12" cy="13" r="4"/></svg>
                  </button>
                </div>
                <div class="admin-info">
                  <div class="info-row">
                    <span class="admin-name">{{ item.username }}</span>
                    <span v-if="item.id === currentAdminId" class="self-tag">你</span>
                  </div>
                  <span class="admin-sub">{{ item.role === 'super_admin' ? '超级管理员账号' : '管理员账号' }}</span>
                  <span v-if="item.email" class="admin-email">
                    <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="4" width="20" height="16" rx="2"/><path d="M22 7l-10 6L2 7"/></svg>
                    {{ item.email }}
                  </span>
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
                    {{ fmtDateTime(item.created_at) || '-' }}
                  </span>
                </div>
                <div v-if="isSuper || item.id === currentAdminId" class="foot-actions">
                  <button
                    v-if="item.id === currentAdminId || isSuper"
                    class="act-btn act-login"
                    @click="openLoginModal(item)"
                  >
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4"/><polyline points="10 17 15 12 10 7"/><line x1="15" y1="12" x2="3" y2="12"/></svg>
                    修改登录
                  </button>
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
                <span v-else class="foot-none">当前账户信息</span>
              </div>
            </div>
          </TransitionGroup>
        </div>
      </template>
    </section>

    <!-- 新增管理员弹窗 -->
    <Transition name="modal">
      <div v-if="addModalVisible" class="modal-backdrop">
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
              <label>邮箱 <span class="opt-label">（可选，用于接收通知）</span></label>
              <input v-model="form.email" type="email" placeholder="请输入管理员的接收通知邮箱" autocomplete="off" />
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
                  @click="selectSuperRole"
                >
                  <div class="role-opt-icon role-opt-super">
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2l3 7h7l-5.5 4.5L18 21l-6-4-6 4 1.5-7.5L2 9h7z"/></svg>
                  </div>
                  <div class="role-opt-text">
                    <span class="role-opt-name">超级管理员</span>
                    <span class="role-opt-desc">全部权限，全局仅可有一个</span>
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

    <!-- 上传头像弹窗 -->
    <Transition name="modal">
      <div v-if="avatarModalVisible" class="modal-backdrop">
        <div class="modal-dialog">
          <div class="modal-head">
            <h3>上传头像 - {{ avatarTarget?.username || '' }}</h3>
            <button class="modal-close" @click="closeAvatarModal">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
          <div class="modal-body">
            <div class="avatar-preview">
              <img v-if="avatarPreview" :src="avatarPreview" alt="预览" />
              <div v-else class="avatar-preview-empty">请选择图片</div>
            </div>
            <label class="avatar-file-label">
              <input
                type="file"
                accept="image/jpeg,image/png,image/webp,image/gif"
                class="avatar-file-input"
                @change="onAvatarFileChange"
              />
              <span class="avatar-file-btn">选择图片</span>
            </label>
            <p class="avatar-tip">支持 JPG / PNG / WEBP / GIF，建议正方形图片，将自动缩放为 256×256。</p>
          </div>
          <div class="modal-foot">
            <button class="btn-cancel" @click="closeAvatarModal">取消</button>
            <button class="btn-save" :disabled="avatarSaving || !avatarPreview" @click="doUploadAvatar">
              <span v-if="avatarSaving" class="btn-spinner"></span>
              {{ avatarSaving ? '上传中...' : '确认上传' }}
            </button>
          </div>
        </div>
      </div>
    </Transition>

    <!-- 修改登录弹窗 -->
    <Transition name="modal">
      <div v-if="loginModalVisible" class="modal-backdrop">
        <div class="modal-dialog">
          <div class="modal-head">
            <h3>修改登录 - {{ loginTarget?.username || '' }}</h3>
            <button class="modal-close" @click="closeLoginModal">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
          <div class="modal-body">
            <div class="field">
              <label class="required">新用户名</label>
              <input v-model="loginForm.new_username" type="text" placeholder="请输入新用户名" autocomplete="off" @keydown.enter="submitLogin" />
            </div>
            <div class="field">
              <label>邮箱 <span class="opt-label">（留空则保持原邮箱）</span></label>
              <input v-model="loginForm.new_email" type="email" placeholder="请输入管理员的接收通知邮箱" autocomplete="off" @keydown.enter="submitLogin" />
            </div>
            <div v-if="needOldPassword" class="field">
              <label class="required">当前密码</label>
              <input v-model="loginForm.old_password" type="password" placeholder="请输入当前密码" autocomplete="current-password" />
              <p class="field-hint">修改自己的账号需验证当前密码。</p>
            </div>
            <div class="field">
              <label>新密码 <span class="opt-label">（留空则不修改）</span></label>
              <input v-model="loginForm.new_password" type="password" placeholder="至少 6 个字符" autocomplete="new-password" />
              <Transition name="fade-up">
                <div v-if="loginForm.new_password" class="strength">
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
              <label>确认新密码 <span class="opt-label">（留空则不修改）</span></label>
              <input v-model="loginForm.confirm_password" type="password" placeholder="再次输入新密码" autocomplete="new-password" />
              <span v-if="loginForm.confirm_password && loginForm.new_password && loginForm.confirm_password !== loginForm.new_password" class="hint hint-error">两次输入的密码不一致</span>
              <span v-else-if="loginForm.confirm_password && loginForm.confirm_password === loginForm.new_password" class="hint hint-ok">两次密码一致</span>
            </div>
          </div>
          <div class="modal-foot">
            <button class="btn-cancel" @click="closeLoginModal">取消</button>
            <button class="btn-save" :disabled="loginSaving" @click="submitLogin">
              <span v-if="loginSaving" class="btn-spinner"></span>
              {{ loginSaving ? '提交中...' : '保存' }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { adminApi, showToast, getAdminUser, setAdminUser } from '@/api/client'
import { webConfirm } from '@/utils/webDialog'
import { useAuthStore } from '@/stores/auth'
import { fmtDateTime } from '@/utils/time'

interface Admin {
  id: number
  username: string
  avatar_url: string
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

const auth = useAuthStore()
const isSuper = computed(() => auth.user?.role === 'super_admin')
const currentAdminId = computed(() => auth.user?.id ?? 0)

// ===== 管理员列表 =====
const loading = ref(true)
const adminList = ref<Admin[]>([])
const stats = ref<AdminStats>({ total: 0, active: 0, disabled: 0, super_admin: 0, admin: 0 })

function initialOf(name: string): string {
  if (!name) return '?'
  return name.charAt(0).toUpperCase()
}

// 头像上传权限：超管可传任意管理员，普通管理员只能传自己
function canUploadAvatar(item: Admin): boolean {
  if (isSuper.value) return true
  return item.id === currentAdminId.value
}

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
  const ok = await webConfirm(`确认${action}管理员 "${item.username}"？`, { title: `${action}管理员`, confirmText: `确认${action}` })
  if (!ok) return
  const res = await adminApi('toggle_admin_status', { id: item.id })
  if (res.code === 200) {
    showToast(`已${action}`, 'success')
    item.status = item.status == 1 ? 0 : 1
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
  const ok = await webConfirm(`确认删除管理员 "${item.username}"？此操作不可恢复！`, { title: '删除管理员', confirmText: '确认删除' })
  if (!ok) return
  const res = await adminApi('delete_admin', { id: item.id })
  if (res.code === 200) {
    showToast('删除成功', 'success')
    stats.value.total--
    if (item.status == 1) stats.value.active--
    else stats.value.disabled--
    if (item.role === 'super_admin') stats.value.super_admin--
    else stats.value.admin--
    adminList.value = adminList.value.filter(a => a.id !== item.id)
  } else {
    showToast(res.msg || '删除失败')
  }
}

// ===== 新增弹窗 =====
const addModalVisible = ref(false)
const saving = ref(false)
const form = ref({ username: '', password: '', email: '', role: 'admin' })

function openAddModal() {
  form.value = { username: '', password: '', email: '', role: 'admin' }
  addModalVisible.value = true
}

function closeAddModal() {
  if (saving.value) return
  addModalVisible.value = false
}

function selectSuperRole() {
  // 仅当当前无超管时允许选择，否则提示
  if (stats.value.super_admin > 0) {
    showToast('超级管理员已存在，全局最多只能有一个')
    return
  }
  form.value.role = 'super_admin'
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
  const email = form.value.email.trim()
  if (email && !isValidEmail(email)) {
    showToast('邮箱格式不正确')
    return
  }
  saving.value = true
  const res = await adminApi('add_admin', {
    username: form.value.username.trim(),
    password: form.value.password,
    email,
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

// ===== 头像上传 =====
const avatarModalVisible = ref(false)
const avatarSaving = ref(false)
const avatarTarget = ref<Admin | null>(null)
const avatarPreview = ref('')

function openAvatarModal(item: Admin) {
  avatarTarget.value = item
  avatarPreview.value = ''
  avatarModalVisible.value = true
}

function closeAvatarModal() {
  if (avatarSaving.value) return
  avatarModalVisible.value = false
  avatarTarget.value = null
  avatarPreview.value = ''
}

function onAvatarFileChange(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  if (!/^image\/(jpeg|png|webp|gif)$/i.test(file.type)) {
    showToast('只支持 JPG / PNG / WEBP / GIF 图片')
    return
  }
  if (file.size > 4 * 1024 * 1024) {
    showToast('图片不能超过 4MB')
    return
  }
  const reader = new FileReader()
  reader.onload = () => {
    avatarPreview.value = reader.result as string
  }
  reader.readAsDataURL(file)
}

async function doUploadAvatar() {
  if (!avatarTarget.value || !avatarPreview.value) return
  avatarSaving.value = true
  const res = await adminApi('upload_admin_avatar', {
    admin_id: avatarTarget.value.id,
    image: avatarPreview.value,
  })
  avatarSaving.value = false
  if (res.code === 200) {
    showToast('头像已更新', 'success')
    avatarTarget.value.avatar_url = res.data?.avatar_url || avatarPreview.value
    avatarModalVisible.value = false
    avatarTarget.value = null
    avatarPreview.value = ''
  } else {
    showToast(res.msg || '上传失败')
  }
}

// ===== 修改登录（用户名 + 密码）弹窗 =====
const loginModalVisible = ref(false)
const loginSaving = ref(false)
const loginTarget = ref<Admin | null>(null)
const loginForm = ref<{ new_username: string; new_email: string; old_password: string; new_password: string; confirm_password: string }>({
  new_username: '',
  new_email: '',
  old_password: '',
  new_password: '',
  confirm_password: '',
})

function openLoginModal(item: Admin) {
  loginTarget.value = item
  loginForm.value = { new_username: item.username, new_email: item.email || '', old_password: '', new_password: '', confirm_password: '' }
  loginModalVisible.value = true
}

function closeLoginModal() {
  if (loginSaving.value) return
  loginModalVisible.value = false
  loginTarget.value = null
}

// 修改自己时需要填当前密码；超管修改他人时无需
const needOldPassword = computed(() => !!loginTarget.value && loginTarget.value.id === currentAdminId.value)

function strengthLevel(): number {
  const pw = loginForm.value.new_password
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

async function submitLogin() {
  const target = loginTarget.value
  if (!target) return
  const { new_username, new_email, old_password, new_password, confirm_password } = loginForm.value
  const uname = new_username.trim()
  const email = new_email.trim()
  const isSelf = target.id === currentAdminId.value
  if (!uname) {
    showToast('请输入新用户名')
    return
  }
  if (email && !isValidEmail(email)) {
    showToast('邮箱格式不正确')
    return
  }
  if (isSelf && !old_password) {
    showToast('请输入当前密码')
    return
  }
  if (new_password && new_password.length < 6) {
    showToast('新密码至少需要 6 个字符')
    return
  }
  if (new_password && new_password !== confirm_password) {
    showToast('两次输入的新密码不一致')
    return
  }
  loginSaving.value = true
  const payload: Record<string, string | number> = { new_username: uname, admin_id: target.id }
  if (email) payload.new_email = email
  if (isSelf) payload.old_password = old_password
  if (new_password) {
    payload.new_password = new_password
    payload.confirm_password = confirm_password
  }
  const res = await adminApi('change_login', payload)
  loginSaving.value = false
  if (res.code === 200) {
    showToast(res.msg || '修改成功', 'success')
    loginModalVisible.value = false
    loginTarget.value = null
    await reloadAfterLoginChange()
  } else {
    showToast(res.msg || '修改失败')
  }
}

function isValidEmail(email: string): boolean {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)
}

async function reloadAfterLoginChange() {
  // 若修改的是自己，更新本地缓存的登录用户信息
  if (loginTarget.value && loginTarget.value.id === currentAdminId.value) {
    const u = getAdminUser()
    if (u) {
      u.username = loginForm.value.new_username.trim()
      setAdminUser(u)
    }
  }
  await loadList()
}

onMounted(() => {
  loadList()
})
</script>

<style scoped>
.admin-account-page {
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
  max-width: 560px;
}
.btn-add {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 9px 16px;
  border-radius: 10px;
  border: none;
  background: var(--accent);
  color: #fff;
  font-size: 13px;
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

/* ===== 区块 ===== */
.section-block {
  background: var(--card, var(--white));
  border: 1px solid var(--border);
  border-radius: 16px;
  padding: 18px;
  margin-bottom: 20px;
}
.section-sec { padding: 20px; }
.section-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
}
.section-title {
  display: flex;
  align-items: center;
  gap: 12px;
}
.section-icon {
  width: 38px; height: 38px;
  border-radius: 10px;
  display: flex; align-items: center; justify-content: center;
  flex-shrink: 0;
}
.section-icon-admin { background: #eff6ff; color: #3b82f6; }
.section-icon-sec { background: #fffbeb; color: #f59e0b; }
.section-name { font-size: 15px; font-weight: 700; margin: 0; color: var(--text); }
.section-desc { font-size: 12px; color: var(--text-muted); margin: 2px 0 0; }

/* ===== 管理员卡片网格 ===== */
.admin-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
  gap: 12px;
}
.admin-card {
  background: var(--white);
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 16px 18px;
  transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
  animation: cardIn 0.5s cubic-bezier(0.16, 1, 0.3, 1) both;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.admin-card:hover { box-shadow: 0 8px 24px rgba(0, 0, 0, 0.06); transform: translateY(-2px); }
.admin-card.is-disabled { opacity: 0.65; }
.admin-card.is-self { border-color: var(--accent); box-shadow: 0 0 0 2px rgba(26, 26, 26, 0.06); }
@keyframes cardIn {
  from { opacity: 0; transform: translateY(16px); }
  to { opacity: 1; transform: translateY(0); }
}

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
.avatar-wrap {
  position: relative;
  width: 44px;
  height: 44px;
  flex-shrink: 0;
}
.avatar-wrap .admin-avatar,
.avatar-wrap .admin-avatar-img {
  width: 44px;
  height: 44px;
}
.admin-avatar-img {
  border-radius: 50%;
  object-fit: cover;
  display: block;
}
.avatar-upload-btn {
  position: absolute;
  right: -6px;
  bottom: -6px;
  width: 22px;
  height: 22px;
  border-radius: 50%;
  border: 2px solid var(--white);
  background: var(--accent);
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  padding: 0;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.2);
  transition: transform 0.2s;
}
.avatar-upload-btn:hover { transform: scale(1.1); }
.avatar-upload-btn:active { transform: scale(0.95); }
.avatar-preview {
  width: 120px;
  height: 120px;
  border-radius: 50%;
  margin: 0 auto 16px;
  overflow: hidden;
  border: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: center;
  background: #f5f5f5;
}
.avatar-preview img { width: 100%; height: 100%; object-fit: cover; display: block; }
.avatar-preview-empty { font-size: 12px; color: var(--text-muted); }
.avatar-file-label { display: block; text-align: center; cursor: pointer; }
.avatar-file-input { display: none; }
.avatar-file-btn {
  display: inline-block;
  padding: 8px 24px;
  border-radius: 10px;
  border: 1px solid var(--border);
  background: var(--white);
  color: var(--text);
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}
.avatar-file-btn:hover { border-color: var(--accent); color: var(--accent); }
.avatar-tip { margin: 12px 0 0; text-align: center; font-size: 12px; color: var(--text-muted); }

.admin-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}
.info-row { display: flex; align-items: center; gap: 6px; }
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
.admin-sub {
  font-size: 12px;
  color: var(--text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.admin-email {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  margin-top: 4px;
  font-size: 11px;
  color: var(--text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 100%;
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
.foot-none { font-size: 12px; color: var(--text-muted); }
.meta-status {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 12px;
  font-weight: 600;
}
.status-dot { width: 7px; height: 7px; border-radius: 50%; }
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
.act-login { background: #eff6ff; color: #3b82f6; }
.act-login:hover { background: #dbeafe; }
.act-delete { background: #fef2f2; color: #dc2626; }
.act-delete:hover { background: #fee2e2; }

/* ===== 账户安全卡片 ===== */
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
.card:hover { box-shadow: 0 8px 24px rgba(0, 0, 0, 0.06); transform: translateY(-2px); }
.card-head { display: flex; align-items: center; gap: 12px; }
.card-icon {
  width: 40px; height: 40px;
  border-radius: 10px;
  display: flex; align-items: center; justify-content: center;
  flex-shrink: 0;
}
.icon-user { background: #eff6ff; color: #3b82f6; }
.icon-lock { background: #fffbeb; color: #f59e0b; }
.card-head-text { display: flex; flex-direction: column; gap: 2px; min-width: 0; }
.card-title { font-size: 16px; font-weight: 700; margin: 0; color: var(--text); }
.card-desc { font-size: 12px; color: var(--text-muted); margin: 0; line-height: 1.5; }
.card-body { display: flex; flex-direction: column; }

.field { margin-bottom: 16px; }
.field:last-child { margin-bottom: 0; }
.field label {
  display: block;
  font-size: 13px;
  font-weight: 600;
  color: var(--text);
  margin-bottom: 8px;
}
.field label.required::after { content: '*'; color: #ef4444; margin-left: 2px; }
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

.select-wrap { position: relative; }
.admin-select {
  width: 100%;
  appearance: none;
  -webkit-appearance: none;
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 10px 38px 10px 14px;
  font-size: 14px;
  font-family: inherit;
  color: var(--text);
  background: var(--white);
  outline: none;
  cursor: pointer;
  transition: border-color 0.2s;
  box-sizing: border-box;
}
.admin-select:focus { border-color: var(--accent); }
.admin-select:disabled { opacity: 0.6; cursor: not-allowed; }
.select-wrap svg {
  position: absolute;
  right: 12px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--text-light);
  pointer-events: none;
}
.field-hint { font-size: 11px; color: var(--text-muted); margin: 6px 0 0; line-height: 1.5; }
.opt-label { font-size: 11px; font-weight: 500; color: var(--text-light); }
.hint { display: block; font-size: 11px; margin-top: 6px; font-weight: 500; }
.hint-error { color: #ef4444; }
.hint-ok { color: #16a34a; }

.strength { display: flex; align-items: center; gap: 8px; margin-top: 8px; }
.strength-bars { display: flex; gap: 4px; flex: 1; }
.bar { flex: 1; height: 4px; border-radius: 2px; background: #ececf0; transition: background 0.25s ease; }
.bar.lv-1 { background: #ef4444; }
.bar.lv-2 { background: #f59e0b; }
.bar.lv-3 { background: #22c55e; }
.strength-label { font-size: 11px; font-weight: 600; white-space: nowrap; color: var(--text-light); }
.strength-label.lv-1 { color: #ef4444; }
.strength-label.lv-2 { color: #f59e0b; }
.strength-label.lv-3 { color: #22c55e; }

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
  width: 14px; height: 14px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: #fff;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

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

.role-select { display: flex; flex-direction: column; gap: 8px; }
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

/* ===== 空状态 / 加载 ===== */
.state-box {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  color: var(--text-muted);
  gap: 12px;
  font-size: 14px;
}
.state-empty { padding: 48px 20px; }
.empty-icon { color: #d0d0d0; margin-bottom: 4px; }
.empty-title { font-size: 15px; font-weight: 600; color: var(--text-light); margin: 0; }
.empty-sub { font-size: 13px; color: var(--text-muted); margin: 0; }
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
.card-delay.fade-up-enter-active { transition-delay: 0.1s; }
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
  .card-grid { grid-template-columns: 1fr; }
  .section-head { flex-direction: column; align-items: flex-start; }
}
</style>