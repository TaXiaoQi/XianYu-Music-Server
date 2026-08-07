<template>
  <div class="users-wrap">
    <!-- 搜索栏 -->
    <div class="search-bar">
      <input
        v-model="keyword"
        type="text"
        placeholder="搜索用户名或邮箱"
        @keyup.enter="handleSearch"
      />
      <button class="btn btn-primary" @click="handleSearch">搜索</button>
      <button v-if="keyword" class="btn" @click="clearSearch">清除</button>
    </div>

    <!-- 批量操作 -->
    <div class="batch-actions">
      <button class="btn btn-success" @click="batchToggle(1)" :disabled="batchLoading">一键全开</button>
      <button class="btn btn-danger" @click="batchToggle(0)" :disabled="batchLoading">一键全禁</button>
      <button class="btn btn-primary" @click="showAddModal = true">+ 添加用户</button>
      <button class="btn btn-dark" @click="deleteEmptyPlaylists" :disabled="batchLoading">一键删除空的我喜欢的音乐歌单</button>
    </div>

    <!-- 用户表格 -->
    <div class="card">
      <div v-if="loading" class="empty">加载中...</div>
      <div v-else-if="loadError" class="empty">{{ loadError }}</div>
      <div v-else-if="users.length === 0" class="empty">暂无用户数据</div>
      <div v-else class="table-wrapper">
        <table>
          <thead>
            <tr>
              <th>ID</th>
              <th>头像</th>
              <th>用户名</th>
              <th>邮箱</th>
              <th>邮箱验证</th>
              <th>状态</th>
              <th>听歌时长</th>
              <th>注册时间</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="u in users" :key="u.id">
              <td>{{ u.id }}</td>
              <td>
                <img
                  v-if="u.avatar_url"
                  :src="u.avatar_url"
                  alt="头像"
                  class="user-avatar"
                  @click="openAvatarView(u)"
                />
                <div v-else class="avatar-placeholder">{{ (u.username || '?').charAt(0) }}</div>
              </td>
              <td>{{ u.username }}</td>
              <td>{{ u.email || '-' }}</td>
              <td>
                <span :class="['badge', u.email_verified == 1 ? 'badge-success' : 'badge-warning']">
                  {{ u.email_verified == 1 ? '已验证' : '未验证' }}
                </span>
              </td>
              <td>
                <span :class="['badge', u.status != 0 ? 'badge-success' : 'badge-error']">
                  {{ u.status != 0 ? '正常' : '禁用' }}
                </span>
              </td>
              <td>{{ formatDuration(u.listen_duration) }}</td>
              <td>{{ u.created_at }}</td>
              <td>
                <div class="row-actions">
                  <button class="btn btn-sm" :class="u.status != 0 ? 'btn-danger' : 'btn-success'" @click="toggleStatus(u)">
                    {{ u.status != 0 ? '禁用' : '启用' }}
                  </button>
                  <button class="btn btn-sm" @click="openEmailModal(u)">改邮箱</button>
                  <button class="btn btn-sm" @click="openResetModal(u)">重置时长</button>
                  <button class="btn btn-sm" @click="viewPlugins(u)">插件</button>
                  <button v-if="u.avatar_url" class="btn btn-sm btn-danger" @click="deleteAvatar(u)">删头像</button>
                  <button class="btn btn-sm btn-danger" @click="deleteUser(u)">删除</button>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- 分页 -->
      <div v-if="!loading && total > 0" class="pagination">
        <button :disabled="page <= 1" @click="goPage(page - 1)">上一页</button>
        <button
          v-for="p in pageNumbers"
          :key="p"
          :class="{ active: p === page }"
          @click="goPage(p)"
        >{{ p }}</button>
        <button :disabled="page >= totalPages" @click="goPage(page + 1)">下一页</button>
        <span>共 {{ total }} 条</span>
      </div>
    </div>

    <!-- 添加用户弹窗 -->
    <div v-if="showAddModal" class="modal-overlay" @click.self="showAddModal = false">
      <div class="modal">
        <h3>添加用户</h3>
        <div class="form-group">
          <label>用户名 <span style="color:#c00">*</span></label>
          <input v-model="addForm.username" type="text" placeholder="2-32 个字符" />
        </div>
        <div class="form-group">
          <label>密码 <span style="color:#c00">*</span></label>
          <input v-model="addForm.password" type="password" placeholder="至少 6 位" />
        </div>
        <div class="form-group">
          <label>邮箱（选填）</label>
          <input v-model="addForm.email" type="email" placeholder="留空则不绑定" />
        </div>
        <div class="modal-actions">
          <button class="btn" @click="showAddModal = false">取消</button>
          <button class="btn btn-primary" @click="submitAddUser" :disabled="addLoading">
            {{ addLoading ? '添加中...' : '确定' }}
          </button>
        </div>
      </div>
    </div>

    <!-- 修改邮箱弹窗 -->
    <div v-if="showEmailModal" class="modal-overlay" @click.self="showEmailModal = false">
      <div class="modal">
        <h3>修改用户邮箱</h3>
        <div class="form-group">
          <label>用户名</label>
          <input :value="emailForm.username" type="text" disabled />
        </div>
        <div class="form-group">
          <label>当前邮箱</label>
          <input :value="emailForm.currentEmail || '(无)'" type="text" disabled />
        </div>
        <div class="form-group">
          <label>新邮箱</label>
          <input v-model="emailForm.newEmail" type="email" placeholder="留空则清除邮箱" />
          <div class="hint">清除邮箱后该用户将视为普通成员</div>
        </div>
        <div class="modal-actions">
          <button class="btn" @click="showEmailModal = false">取消</button>
          <button class="btn btn-primary" @click="submitEmailChange" :disabled="emailLoading">
            {{ emailLoading ? '提交中...' : '确定' }}
          </button>
        </div>
      </div>
    </div>

    <!-- 重置听歌时长确认弹窗 -->
    <div v-if="showResetModal" class="modal-overlay" @click.self="showResetModal = false">
      <div class="modal">
        <h3>重置听歌时长</h3>
        <div class="form-group">
          <label>用户名</label>
          <input :value="resetForm.username" type="text" disabled />
        </div>
        <div class="form-group">
          <label>当前听歌时长</label>
          <input :value="resetForm.duration" type="text" disabled />
        </div>
        <div style="background:#fffdf0;border:1px solid #e0d090;color:#b8860b;padding:10px 14px;border-radius:6px;font-size:12px;">
          重置后听歌时长与新歌数将清零，此操作不可恢复。
        </div>
        <div class="modal-actions">
          <button class="btn" @click="showResetModal = false">取消</button>
          <button class="btn btn-danger" @click="submitReset" :disabled="resetLoading">
            {{ resetLoading ? '重置中...' : '确定重置' }}
          </button>
        </div>
      </div>
    </div>

    <!-- 查看插件弹窗 -->
    <div v-if="showPluginsModal" class="modal-overlay" @click.self="showPluginsModal = false">
      <div class="modal" style="max-width:700px;">
        <h3>用户插件 - {{ pluginsData.username }}</h3>
        <div v-if="pluginsLoading" class="empty">加载中...</div>
        <div v-else>
          <div style="display:flex;gap:16px;margin-bottom:16px;font-size:13px;color:#666;">
            <span>插件数量: {{ pluginsData.plugin_count || 0 }}</span>
            <span v-if="pluginsData.uploaded_at">上传时间: {{ pluginsData.uploaded_at }}</span>
            <span v-if="pluginsData.ciyuanxi_id">弦予号: {{ pluginsData.ciyuanxi_id }}</span>
          </div>
          <div v-if="pluginsData.plugins && pluginsData.plugins.length > 0" class="table-wrapper">
            <table>
              <thead>
                <tr><th>名称</th><th>格式</th><th>版本</th><th>作者</th><th>状态</th><th>大小</th></tr>
              </thead>
              <tbody>
                <tr v-for="(p, i) in pluginsData.plugins" :key="i">
                  <td>
                    {{ p.name }}
                    <div v-if="p.description" style="font-size:11px;color:#999;">{{ p.description }}</div>
                  </td>
                  <td><span class="badge badge-info">{{ p.format }}</span></td>
                  <td>{{ p.version || '-' }}</td>
                  <td>{{ p.author || '-' }}</td>
                  <td>
                    <span :class="['badge', p.enabled ? 'badge-success' : 'badge-error']">
                      {{ p.enabled ? '启用' : '禁用' }}
                    </span>
                  </td>
                  <td>{{ formatScriptSize(p.scriptSize) }}</td>
                </tr>
              </tbody>
            </table>
          </div>
          <div v-else class="empty">该用户暂无插件数据</div>
        </div>
        <div class="modal-actions">
          <button class="btn" @click="showPluginsModal = false">关闭</button>
        </div>
      </div>
    </div>

    <!-- 头像大图查看弹窗 -->
    <div v-if="showAvatarModal" class="modal-overlay" @click.self="showAvatarModal = false">
      <div class="modal" style="width:300px;text-align:center;">
        <h3>{{ avatarViewUser?.username }} 的头像</h3>
        <img
          v-if="avatarViewUser?.avatar_url"
          :src="avatarViewUser.avatar_url"
          alt="头像"
          style="width:200px;height:200px;border-radius:50%;object-fit:cover;margin:16px auto;"
        />
        <div class="modal-actions">
          <button class="btn" @click="showAvatarModal = false">关闭</button>
          <button class="btn btn-danger" @click="confirmDeleteAvatar" :disabled="avatarDeleting">
            {{ avatarDeleting ? '删除中...' : '删除头像' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { adminApi, showToast } from '@/api/client'

// ===== 类型定义 =====
interface User {
  id: number
  username: string
  email: string
  email_verified: number
  status: number
  listen_duration: number
  created_at: string
  avatar_url: string
  ciyuanxi_id: string
  master_quota: number
  [key: string]: any
}

interface Plugin {
  name: string
  format: string
  version: string
  author: string
  description: string
  enabled: boolean
  scriptSize: number
}

// ===== 列表数据 =====
const users = ref<User[]>([])
const loading = ref(true)
const loadError = ref('')
const keyword = ref('')
const page = ref(1)
const pageSize = 20
const total = ref(0)
const totalPages = ref(0)
const batchLoading = ref(false)

// 分页页码计算
const pageNumbers = computed(() => {
  const max = 7
  const pages: number[] = []
  if (totalPages.value <= max) {
    for (let i = 1; i <= totalPages.value; i++) pages.push(i)
  } else {
    let start = Math.max(1, page.value - 3)
    let end = Math.min(totalPages.value, start + max - 1)
    if (end - start < max - 1) start = Math.max(1, end - max + 1)
    for (let i = start; i <= end; i++) pages.push(i)
  }
  return pages
})

// ===== 加载用户列表 =====
async function loadUsers() {
  loading.value = true
  loadError.value = ''
  const res = await adminApi<{ total: number; page: number; total_pages: number; list: User[] }>('get_users', {
    page: page.value,
    page_size: pageSize,
    keyword: keyword.value,
  })
  if (res.code === 200 && res.data) {
    users.value = res.data.list || []
    total.value = res.data.total
    totalPages.value = res.data.total_pages
  } else {
    loadError.value = res.msg || '加载失败'
    users.value = []
  }
  loading.value = false
}

function handleSearch() {
  page.value = 1
  loadUsers()
}

function clearSearch() {
  keyword.value = ''
  page.value = 1
  loadUsers()
}

function goPage(p: number) {
  if (p < 1 || p > totalPages.value || p === page.value) return
  page.value = p
  loadUsers()
}

// ===== 工具函数 =====
function formatDuration(seconds: number | undefined): string {
  const dur = Number(seconds) || 0
  const h = Math.floor(dur / 3600)
  const m = Math.floor((dur % 3600) / 60)
  const s = dur % 60
  return `${h}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`
}

function formatScriptSize(bytes: number | undefined): string {
  const b = Number(bytes) || 0
  if (b < 1024) return `${b} B`
  return `${(b / 1024).toFixed(1)} KB`
}

// ===== 行操作 =====
async function toggleStatus(u: User) {
  const newStatus = u.status != 0 ? 0 : 1
  const res = await adminApi('toggle_user_status', { id: u.id, status: newStatus })
  if (res.code === 200) {
    showToast(newStatus ? '已启用' : '已禁用', 'success')
    u.status = newStatus
  } else {
    showToast(res.msg || '操作失败')
  }
}

async function deleteUser(u: User) {
  if (!confirm(`确定删除用户 "${u.username}" 吗？此操作不可恢复。`)) return
  const res = await adminApi('delete_user', { id: u.id })
  if (res.code === 200) {
    showToast('删除成功', 'success')
    loadUsers()
  } else {
    showToast(res.msg || '删除失败')
  }
}

async function deleteAvatar(u: User) {
  if (!confirm(`确定删除用户 "${u.username}" 的头像吗？`)) return
  const res = await adminApi('delete_user_avatar', { user_id: u.id })
  if (res.code === 200) {
    showToast('头像已删除', 'success')
    u.avatar_url = ''
  } else {
    showToast(res.msg || '删除失败')
  }
}

// ===== 批量操作 =====
async function batchToggle(status: number) {
  const label = status ? '全开' : '全禁'
  if (!confirm(`确定${label}所有用户状态吗？`)) return
  batchLoading.value = true
  const res = await adminApi('batch_toggle_user_status', { status })
  batchLoading.value = false
  if (res.code === 200) {
    showToast(res.msg || `已${label}`, 'success')
    loadUsers()
  } else {
    showToast(res.msg || '操作失败')
  }
}

async function deleteEmptyPlaylists() {
  if (!confirm('确定删除所有空的"我喜欢的音乐"歌单吗？')) return
  batchLoading.value = true
  const res = await adminApi('delete_empty_favorite_playlists')
  batchLoading.value = false
  if (res.code === 200 && res.data) {
    const d = res.data as any
    showToast(`已删除 ${d.deleted_count || 0} 个空歌单（扫描 ${d.total_scanned || 0} 个）`, 'success')
  } else {
    showToast(res.msg || '操作失败')
  }
}

// ===== 添加用户弹窗 =====
const showAddModal = ref(false)
const addLoading = ref(false)
const addForm = ref({ username: '', password: '', email: '' })

async function submitAddUser() {
  const uname = addForm.value.username.trim()
  if (uname.length < 2 || uname.length > 32) {
    showToast('用户名需 2-32 个字符')
    return
  }
  if (addForm.value.password.length < 6) {
    showToast('密码至少 6 位')
    return
  }
  if (addForm.value.email && !isValidEmail(addForm.value.email)) {
    showToast('邮箱格式不正确')
    return
  }
  addLoading.value = true
  const res = await adminApi('add_user', {
    username: uname,
    password: addForm.value.password,
    email: addForm.value.email.trim(),
  })
  addLoading.value = false
  if (res.code === 200) {
    showToast(`添加成功${res.data ? '，弦予号: ' + (res.data as any).ciyuanxi_id : ''}`, 'success')
    showAddModal.value = false
    addForm.value = { username: '', password: '', email: '' }
    loadUsers()
  } else {
    showToast(res.msg || '添加失败')
  }
}

// ===== 修改邮箱弹窗 =====
const showEmailModal = ref(false)
const emailLoading = ref(false)
const emailForm = ref({ userId: 0, username: '', currentEmail: '', newEmail: '' })

function openEmailModal(u: User) {
  emailForm.value = { userId: u.id, username: u.username, currentEmail: u.email || '', newEmail: '' }
  showEmailModal.value = true
}

async function submitEmailChange() {
  if (emailForm.value.newEmail && !isValidEmail(emailForm.value.newEmail)) {
    showToast('邮箱格式不正确')
    return
  }
  emailLoading.value = true
  const res = await adminApi('change_user_email', {
    user_id: emailForm.value.userId,
    new_email: emailForm.value.newEmail.trim(),
  })
  emailLoading.value = false
  if (res.code === 200) {
    showToast('邮箱已更新', 'success')
    showEmailModal.value = false
    loadUsers()
  } else {
    showToast(res.msg || '修改失败')
  }
}

// ===== 重置听歌时长弹窗 =====
const showResetModal = ref(false)
const resetLoading = ref(false)
const resetForm = ref({ userId: 0, username: '', duration: '', ciyuanxiId: '' })

function openResetModal(u: User) {
  resetForm.value = {
    userId: u.id,
    username: u.username,
    duration: formatDuration(u.listen_duration),
    ciyuanxiId: u.ciyuanxi_id || '',
  }
  showResetModal.value = true
}

async function submitReset() {
  resetLoading.value = true
  const res = await adminApi('reset_listen_duration', {
    user_id: resetForm.value.userId,
    ciyuanxi_id: resetForm.value.ciyuanxiId,
  })
  resetLoading.value = false
  if (res.code === 200) {
    showToast('重置成功', 'success')
    showResetModal.value = false
    loadUsers()
  } else {
    showToast(res.msg || '重置失败')
  }
}

// ===== 查看插件弹窗 =====
const showPluginsModal = ref(false)
const pluginsLoading = ref(false)
const pluginsData = ref<any>({})

async function viewPlugins(u: User) {
  showPluginsModal.value = true
  pluginsLoading.value = true
  pluginsData.value = {}
  const res = await adminApi('get_user_plugins', { user_id: u.id })
  pluginsLoading.value = false
  if (res.code === 200 && res.data) {
    pluginsData.value = res.data
  } else {
    showToast(res.msg || '加载失败')
  }
}

// ===== 头像大图弹窗 =====
const showAvatarModal = ref(false)
const avatarViewUser = ref<User | null>(null)
const avatarDeleting = ref(false)

function openAvatarView(u: User) {
  avatarViewUser.value = u
  showAvatarModal.value = true
}

async function confirmDeleteAvatar() {
  if (!avatarViewUser.value) return
  avatarDeleting.value = true
  const res = await adminApi('delete_user_avatar', { user_id: avatarViewUser.value.id })
  avatarDeleting.value = false
  if (res.code === 200) {
    showToast('头像已删除', 'success')
    avatarViewUser.value.avatar_url = ''
    showAvatarModal.value = false
    loadUsers()
  } else {
    showToast(res.msg || '删除失败')
  }
}

// ===== 工具函数 =====
function isValidEmail(email: string): boolean {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)
}

// ===== 初始化 =====
onMounted(() => {
  loadUsers()
})
</script>

<style scoped>
.users-wrap {
  max-width: 1320px;
  margin: 0 auto;
}

/* 搜索栏 */
.search-bar {
  display: flex;
  gap: 12px;
  margin-bottom: 16px;
  align-items: center;
}
.search-bar input {
  padding: 8px 12px;
  border: 1px solid var(--border);
  border-radius: 6px;
  font-size: 13px;
  outline: none;
  min-width: 240px;
  background: var(--white);
}
.search-bar input:focus { border-color: var(--accent); }

/* 批量操作 */
.batch-actions {
  display: flex;
  gap: 12px;
  margin-bottom: 16px;
  flex-wrap: wrap;
}
.btn-dark {
  background: var(--accent);
  color: var(--white);
  border-color: var(--accent);
}
.btn-dark:hover { background: #000; border-color: #000; }
.btn-dark:disabled { opacity: 0.6; cursor: not-allowed; }

/* 表格 */
.table-wrapper {
  overflow-x: auto;
  -webkit-overflow-scrolling: touch;
}

/* 头像 */
.user-avatar {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  object-fit: cover;
  cursor: pointer;
  transition: opacity 0.15s;
}
.user-avatar:hover { opacity: 0.8; }
.avatar-placeholder {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: #e0e0e0;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #999;
  font-size: 14px;
}

/* 行操作按钮 */
.row-actions {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
}

/* 分页 */
.pagination {
  display: flex;
  justify-content: center;
  gap: 6px;
  margin-top: 16px;
  align-items: center;
}
.pagination button {
  padding: 6px 12px;
  border: 1px solid var(--border);
  background: var(--white);
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  transition: all 0.15s;
}
.pagination button:hover:not(.active):not(:disabled) {
  border-color: var(--accent);
  color: var(--accent);
}
.pagination button.active {
  background: var(--accent);
  color: var(--white);
  border-color: var(--accent);
}
.pagination button:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
.pagination span {
  font-size: 12px;
  color: var(--text-muted);
  margin-left: 8px;
}

/* 弹窗 */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.4);
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
}
.modal {
  background: var(--white);
  border-radius: 8px;
  padding: 28px;
  width: 500px;
  max-width: 100%;
  max-height: 85vh;
  overflow-y: auto;
  border: 1px solid var(--border);
}
.modal h3 {
  font-size: 17px;
  margin-bottom: 20px;
  font-weight: 700;
}
.form-group {
  margin-bottom: 16px;
}
.form-group label {
  display: block;
  font-size: 13px;
  color: var(--text-light);
  margin-bottom: 6px;
  font-weight: 500;
}
.form-group input {
  width: 100%;
  padding: 10px 14px;
  border: 1px solid var(--border);
  border-radius: 6px;
  font-size: 14px;
  outline: none;
  transition: border-color 0.15s;
  background: var(--white);
}
.form-group input:focus { border-color: var(--accent); }
.form-group input:disabled { background: #fafafa; color: var(--text-muted); }
.form-group .hint {
  font-size: 12px;
  color: var(--text-muted);
  margin-top: 4px;
}
.modal-actions {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
  margin-top: 24px;
}
</style>
