<template>
  <div class="users-wrap">
    <!-- 顶部工具栏：搜索 + 操作合并为一行，批量管理在最右侧弹出 -->
    <Transition name="fade-down" appear>
    <div class="toolbar-row">
      <div class="search-box">
        <input
          v-model="keyword"
          type="text"
          placeholder="搜索昵称、弦予号或邮箱"
          @keyup.enter="handleSearch"
        />
        <button class="btn btn-primary" @click="handleSearch">搜索</button>
        <button v-if="keyword" class="btn" @click="clearSearch">清除</button>
      </div>

      <div class="toolbar-actions">
        <Transition name="batch-swap" mode="out-in">
          <div v-if="!isBatchMode" key="normal" class="toolbar-actions-row">
            <button class="btn btn-primary" @click="showAddModal = true">+ 添加用户</button>
            <button class="btn btn-dark" @click="enterBatchMode">批量管理</button>
          </div>

          <div v-else key="batch" class="batch-mode-bar">
            <button class="btn btn-sm" @click="toggleSelectAll">
              <span class="checkbox-badge" :class="{ checked: isAllSelected }">
                <svg v-if="isAllSelected" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
              </span>
              {{ isAllSelected ? '取消全选' : '全选' }}
            </button>
            <button class="btn btn-sm" @click="batchToggleSelected(0)" :disabled="selectedCount === 0 || batchLoading">封禁</button>
            <button class="btn btn-sm" @click="batchToggleSelected(1)" :disabled="selectedCount === 0 || batchLoading">启用</button>
            <button class="btn btn-sm" @click="batchBanDevice" :disabled="selectedCount === 0 || batchLoading">封禁ID</button>
            <button class="btn btn-sm btn-danger" @click="batchDeleteSelected" :disabled="selectedCount === 0 || batchLoading">删除</button>
            <button class="btn btn-sm" @click="deleteEmptyPlaylists" :disabled="batchLoading">清空歌单</button>
            <span class="batch-count">已选 {{ selectedCount }} 项</span>
            <button class="btn btn-sm btn-primary" @click="exitBatchMode">完成</button>
          </div>
        </Transition>
      </div>
    </div>
    </Transition>

    <!-- 用户表格 -->
    <Transition name="fade-up" appear>
    <div class="card">
      <div v-if="loading" class="empty">加载中...</div>
      <div v-else-if="loadError" class="empty">{{ loadError }}</div>
      <div v-else-if="users.length === 0" class="empty">暂无用户数据</div>
      <div v-else class="table-wrapper">
        <table>
          <thead>
            <tr>
              <th v-if="isBatchMode" class="col-check">
                <span class="checkbox-badge" :class="{ checked: isAllSelected }" @click="toggleSelectAll">
                  <svg v-if="isAllSelected" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
                </span>
              </th>
              <th>弦予号</th>
              <th>头像</th>
              <th>昵称</th>
              <th>邮箱</th>
              <th>邮箱验证</th>
              <th>状态</th>
              <th>听歌时长</th>
              <th>注册时间</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(u, idx) in users" :key="u.id" class="table-row" :class="{ 'row-selected': isBatchMode && selectedIds.has(u.id) }" :style="{ animationDelay: `${idx * 40}ms` }">
              <td v-if="isBatchMode" class="col-check">
                <span class="checkbox-badge" :class="{ checked: selectedIds.has(u.id) }" @click="toggleSelect(u.id)">
                  <svg v-if="selectedIds.has(u.id)" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
                </span>
              </td>
              <td>{{ u.ciyuanxi_id || '-' }}</td>
              <td>
                <img
                  v-if="u.avatar_url"
                  :src="u.avatar_url"
                  alt="头像"
                  class="user-avatar"
                  @click="openAvatarView(u)"
                />
                <div v-else class="avatar-placeholder">{{ (u.nickname || u.username || '?').charAt(0) }}</div>
              </td>
              <td>{{ u.nickname || u.username }}</td>
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
                <div v-if="u.status == 0 && u.ban_reason" style="margin-top:4px;font-size:11px;color:#e74c3c;max-width:160px;word-break:break-all;">
                  原因：{{ u.ban_reason }}
                </div>
              </td>
              <td>{{ formatDuration(u.listen_duration) }}</td>
              <td>{{ u.created_at }}</td>
              <td>
                <div class="row-actions">
                  <button class="btn btn-sm btn-primary" @click="openRowMenu(u)">
                    操作
                  </button>
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
    </Transition>

    <!-- 添加用户弹窗 -->
    <Transition name="modal">
    <div v-if="showAddModal" class="modal-overlay">
      <div class="modal">
        <h3>添加用户</h3>
        <div class="form-group">
          <label class="required">弦予号</label>
          <input v-model="addForm.username" type="text" placeholder="6-20 位，仅含字母或数字" />
          <div class="hint">用于登录，6-20 位，仅含字母或数字（支持大小写字母）</div>
        </div>
        <div class="form-group">
          <label class="required">密码</label>
          <input v-model="addForm.password" type="password" placeholder="至少 6 位" />
        </div>
        <div class="form-group">
          <label>昵称（选填）</label>
          <input v-model="addForm.nickname" type="text" placeholder='留空默认"弦予+弦予号"' />
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
    </Transition>

    <!-- 修改邮箱弹窗 -->
    <Transition name="modal">
    <div v-if="showEmailModal" class="modal-overlay">
      <div class="modal">
        <h3>修改用户邮箱</h3>
        <div class="form-group">
          <label>昵称</label>
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
    </Transition>

    <!-- 重置听歌时长确认弹窗 -->
    <Transition name="modal">
    <div v-if="showResetModal" class="modal-overlay">
      <div class="modal">
        <h3>重置听歌时长</h3>
        <div class="form-group">
          <label>昵称</label>
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
    </Transition>

    <!-- 查看插件弹窗 -->
    <Transition name="modal">
    <div v-if="showPluginsModal" class="modal-overlay">
      <div class="modal" style="max-width:900px;">
        <div class="modal-head-bar">
          <h3>用户插件 - {{ pluginsData.nickname || pluginsData.username }}</h3>
          <button class="modal-close-btn" @click="showPluginsModal = false">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
          </button>
        </div>
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
      </div>
    </div>
    </Transition>

    <!-- 头像大图查看弹窗 -->
    <Transition name="modal">
    <div v-if="showAvatarModal" class="modal-overlay">
      <div class="modal" style="width:300px;text-align:center;">
        <h3>{{ avatarViewUser?.nickname || avatarViewUser?.username }} 的头像</h3>
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
    </Transition>

    <!-- 设备信息弹窗 -->
    <Transition name="modal">
    <div v-if="showDeviceModal" class="modal-overlay">
      <div class="modal" style="max-width:900px;">
        <div class="modal-head-bar">
          <h3>设备信息 - {{ deviceData.nickname || deviceData.username }}</h3>
          <button class="modal-close-btn" @click="showDeviceModal = false">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
          </button>
        </div>
        <div v-if="deviceLoading" class="empty">加载中...</div>
        <div v-else>
          <div style="display:flex;gap:16px;margin-bottom:16px;font-size:13px;color:#666;flex-wrap:wrap;">
            <span>弦予号: {{ deviceData.ciyuanxi_id || '-' }}</span>
            <span v-if="deviceData.last_device_id">设备ID: {{ deviceData.last_device_id }}</span>
            <span v-if="deviceData.is_banned" style="color:#e74c3c;font-weight:600;">设备已封禁</span>
          </div>

          <div v-if="deviceData.last_device_id" style="margin-bottom:16px;">
            <button
              v-if="!deviceData.is_banned"
              class="btn btn-danger btn-sm"
              @click="banUserDevice(deviceData.last_device_id, deviceData.nickname || deviceData.username)"
            >封禁此设备</button>
            <button
              v-else
              class="btn btn-success btn-sm"
              @click="unbanUserDevice(deviceData.last_device_id)"
            >解封此设备</button>
          </div>

          <div v-if="deviceData.login_logs && deviceData.login_logs.length > 0" style="margin-bottom:16px;">
            <h4 style="font-size:13px;margin-bottom:8px;">登录记录</h4>
            <div class="table-wrapper">
              <table>
                <thead><tr><th>设备ID</th><th>IP</th><th>时间</th></tr></thead>
                <tbody>
                  <tr v-for="(log, i) in deviceData.login_logs" :key="'l'+i">
                    <td style="font-size:11px;">{{ log.device_id }}</td>
                    <td>{{ log.ip }}</td>
                    <td>{{ log.created_at }}</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>

          <div v-if="deviceData.open_logs && deviceData.open_logs.length > 0">
            <h4 style="font-size:13px;margin-bottom:8px;">启动记录</h4>
            <div class="table-wrapper">
              <table>
                <thead><tr><th>设备ID</th><th>IP</th><th>版本</th><th>时间</th></tr></thead>
                <tbody>
                  <tr v-for="(log, i) in deviceData.open_logs" :key="'o'+i">
                    <td style="font-size:11px;">{{ log.device_id }}</td>
                    <td>{{ log.ip }}</td>
                    <td>{{ log.app_version }}</td>
                    <td>{{ log.created_at }}</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>

          <div v-if="!deviceData.last_device_id" class="empty">该用户暂无设备记录</div>
        </div>
      </div>
    </div>
    </Transition>

    <!-- 设备管理弹窗 -->
    <Transition name="modal">
    <div v-if="showBannedModal" class="modal-overlay">
      <div class="modal" style="max-width:700px;">
        <h3>设备管理</h3>

        <!-- 手动添加封禁 -->
        <div class="ban-form">
          <div class="form-group ban-device-input">
            <input v-model="banDeviceInput" type="text" placeholder="输入设备ID" />
          </div>
          <div class="form-group ban-reason-input">
            <input v-model="banReasonInput" type="text" placeholder="封禁原因（必填）" />
          </div>
          <button class="btn btn-danger ban-submit" @click="manualBanDevice">封禁</button>
        </div>

        <div v-if="bannedLoading" class="empty">加载中...</div>
        <div v-else-if="bannedDevices.length === 0" class="empty">暂无封禁设备</div>
        <div v-else class="table-wrapper">
          <table>
            <thead>
              <tr><th>ID</th><th>设备ID</th><th>原因</th><th>操作人</th><th>封禁时间</th><th>操作</th></tr>
            </thead>
            <tbody>
              <tr v-for="d in bannedDevices" :key="d.id">
                <td>{{ d.id }}</td>
                <td style="font-size:11px;word-break:break-all;">{{ d.device_id }}</td>
                <td>{{ d.reason || '-' }}</td>
                <td>{{ d.banned_by }}</td>
                <td>{{ d.created_at }}</td>
                <td>
                  <button class="btn btn-sm btn-success" @click="unbanDeviceById(d.id, d.device_id)">解封</button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>

        <div class="modal-actions">
          <button class="btn" @click="showBannedModal = false">关闭</button>
        </div>
      </div>
    </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { adminApi, showToast } from '@/api/client'
import { webConfirm, webPrompt, webActionMenu } from '@/utils/webDialog'

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
async function openRowMenu(u: User) {
  const action = await webActionMenu(`用户操作 · ${u.nickname || u.username}`, [
    { key: 'toggle', label: u.status != 0 ? '禁用用户' : '启用用户', danger: u.status != 0, success: u.status == 0 },
    { key: 'ciyuanxi', label: '修改弦予号' },
    { key: 'email', label: '修改邮箱' },
    { key: 'reset', label: '重置听歌时长' },
    { key: 'plugins', label: '查看插件' },
    { key: 'device', label: '设备信息' },
    { key: 'avatar', label: '删除头像', danger: true, show: !!u.avatar_url },
    { key: 'delete', label: '删除用户', danger: true },
  ])
  if (!action) return
  switch (action) {
    case 'toggle': await toggleStatus(u); break
    case 'ciyuanxi': await changeCiyuanxi(u); break
    case 'email': openEmailModal(u); break
    case 'reset': openResetModal(u); break
    case 'plugins': await viewPlugins(u); break
    case 'device': await openDeviceModal(u); break
    case 'avatar': await deleteAvatar(u); break
    case 'delete': await deleteUser(u); break
  }
}

async function toggleStatus(u: User) {
  const newStatus = u.status != 0 ? 0 : 1
  let reason = ''
  if (newStatus === 0) {
    const input = await webPrompt(`请输入封禁用户 "${u.nickname || u.username}" 的原因：`, '', { title: '封禁用户', placeholder: '封禁原因（必填）' })
    if (input === null) return // 用户取消，静默退出
    reason = input.trim()
    if (!reason) {
      showToast('封禁原因不能为空')
      return
    }
  }
  const res = await adminApi('toggle_user_status', { id: u.id, status: newStatus, reason })
  if (res.code === 200) {
    showToast(newStatus ? '已启用' : '已禁用', 'success')
    u.status = newStatus
    u.ban_reason = newStatus ? '' : reason
  } else {
    showToast(res.msg || '操作失败')
  }
}

async function changeCiyuanxi(u: User) {
  const input = await webPrompt(`请输入 "${u.nickname || u.username}" 的新弦予号：`, u.ciyuanxi_id || '', { title: '修改弦予号', placeholder: '6-20 位，仅含字母或数字' })
  if (input === null) return // 用户取消，静默退出
  const newId = input.trim()
  if (!newId) {
    showToast('请输入弦予号', 'error')
    return
  }
  if (!/^[a-zA-Z0-9]{6,20}$/.test(newId)) {
    showToast('弦予号需 6-20 位，仅含字母或数字', 'error')
    return
  }
  const ok = await webConfirm(`确定将 "${u.nickname || u.username}" 的弦予号由 "${u.ciyuanxi_id || '-'}" 修改为 "${newId}" 吗？`, { title: '修改弦予号', confirmText: '确认修改' })
  if (!ok) return
  const res = await adminApi('change_ciyuanxi_id', { user_id: u.id, new_ciyuanxi_id: newId })
  if (res.code === 200) {
    showToast('弦予号已修改', 'success')
    u.ciyuanxi_id = newId
  } else {
    showToast(res.msg || '修改失败', 'error')
  }
}

async function deleteUser(u: User) {
  const ok = await webConfirm(`确定删除用户 "${u.nickname || u.username}" 吗？此操作不可恢复。`, { title: '删除用户', confirmText: '确认删除' })
  if (!ok) return
  const res = await adminApi('delete_user', { id: u.id })
  if (res.code === 200) {
    showToast('删除成功', 'success')
    loadUsers()
  } else {
    showToast(res.msg || '删除失败')
  }
}

async function deleteAvatar(u: User) {
  const ok = await webConfirm(`确定删除用户 "${u.nickname || u.username}" 的头像吗？`, { title: '删除头像', confirmText: '确认删除' })
  if (!ok) return
  const res = await adminApi('delete_user_avatar', { user_id: u.id })
  if (res.code === 200) {
    showToast('头像已删除', 'success')
    u.avatar_url = ''
  } else {
    showToast(res.msg || '删除失败')
  }
}

// ===== 批量选择模式（参考客户端批量选择页） =====
const isBatchMode = ref(false)
const selectedIds = ref<Set<number>>(new Set())
const selectedCount = computed(() => selectedIds.value.size)
const isAllSelected = computed(() => users.value.length > 0 && selectedIds.value.size === users.value.length)

function enterBatchMode() {
  selectedIds.value = new Set()
  isBatchMode.value = true
}

function exitBatchMode() {
  isBatchMode.value = false
  selectedIds.value = new Set()
}

function toggleSelect(id: number) {
  const next = new Set(selectedIds.value)
  if (next.has(id)) next.delete(id)
  else next.add(id)
  selectedIds.value = next
}

function toggleSelectAll() {
  if (isAllSelected.value) {
    selectedIds.value = new Set()
  } else {
    selectedIds.value = new Set(users.value.map(u => u.id))
  }
}

async function batchToggleSelected(status: number) {
  const ids = [...selectedIds.value]
  if (ids.length === 0) return
  const label = status ? '启用' : '禁用'
  let reason = ''
  if (!status) {
    const input = await webPrompt(`请输入对选中的 ${ids.length} 个用户执行封禁的原因：`, '', { title: '批量禁用', placeholder: '封禁原因（必填）' })
    if (input === null) return // 用户取消，静默退出
    reason = input.trim()
    if (!reason) {
      showToast('封禁原因不能为空')
      return
    }
  }
  const ok = await webConfirm(`确定${label}选中的 ${ids.length} 个用户吗？`, { title: `批量${label}`, confirmText: `确认${label}` })
  if (!ok) return
  batchLoading.value = true
  let fail = 0
  for (const id of ids) {
    const res = await adminApi('toggle_user_status', { id, status, reason })
    if (res.code !== 200) fail++
  }
  batchLoading.value = false
  if (fail === 0) {
    showToast(`已${label} ${ids.length} 个用户`, 'success')
  } else {
    showToast(`${ids.length - fail} 个成功，${fail} 个失败`, 'error')
  }
  selectedIds.value = new Set()
  loadUsers()
}

async function batchBanDevice() {
  const ids = [...selectedIds.value]
  if (ids.length === 0) return
  const targets: { id: number; name: string; deviceId: string }[] = []
  for (const u of users.value) {
    if (!ids.includes(u.id)) continue
    const deviceId = (u.last_device_id || '').trim()
    if (!deviceId) continue
    targets.push({ id: u.id, name: u.nickname || u.username, deviceId })
  }
  if (targets.length === 0) {
    showToast('选中的用户均无设备ID，无需封禁', 'error')
    return
  }
  const input = await webPrompt(`将封禁选中的 ${targets.length} 个用户最近登录的设备，请输入封禁原因：`, '', { title: '批量封禁设备ID', placeholder: '封禁原因（必填）' })
  if (input === null) return // 用户取消，静默退出
  const reason = input.trim()
  if (!reason) {
    showToast('封禁原因不能为空')
    return
  }
  const ok = await webConfirm(`确定封禁 ${targets.length} 个用户的设备ID吗？封禁后这些设备将无法登录。`, { title: '批量封禁设备ID', confirmText: '确认封禁' })
  if (!ok) return
  batchLoading.value = true
  let success = 0
  let fail = 0
  for (const t of targets) {
    const res = await adminApi('ban_device', { device_id: t.deviceId, reason })
    if (res.code === 200) success++
    else fail++
  }
  batchLoading.value = false
  if (fail === 0) {
    showToast(`已封禁 ${success} 个设备ID`, 'success')
  } else {
    showToast(`${success} 个成功，${fail} 个失败`, 'error')
  }
  selectedIds.value = new Set()
}

async function batchDeleteSelected() {
  const ids = [...selectedIds.value]
  if (ids.length === 0) return
  const ok = await webConfirm(`确定删除选中的 ${ids.length} 个用户吗？此操作不可恢复。`, { title: '批量删除', confirmText: '确认删除' })
  if (!ok) return
  batchLoading.value = true
  let fail = 0
  for (const id of ids) {
    const res = await adminApi('delete_user', { id })
    if (res.code !== 200) fail++
  }
  batchLoading.value = false
  if (fail === 0) {
    showToast(`已删除 ${ids.length} 个用户`, 'success')
  } else {
    showToast(`${ids.length - fail} 个成功，${fail} 个失败`, 'error')
  }
  selectedIds.value = new Set()
  loadUsers()
}

async function deleteEmptyPlaylists() {
  const ok = await webConfirm('确定删除所有空的"我喜欢的音乐"歌单吗？', { title: '清理空歌单', confirmText: '确认删除' })
  if (!ok) return
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
const addForm = ref({ username: '', nickname: '', password: '', email: '' })

async function submitAddUser() {
  const ciyuanxi = addForm.value.username.trim()
  if (ciyuanxi.length < 6) {
    showToast('弦予号至少 6 位')
    return
  }
  if (!/^[a-zA-Z0-9]{6,20}$/.test(ciyuanxi)) {
    showToast('弦予号需 6-20 位，仅含字母或数字')
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
    username: ciyuanxi,
    nickname: addForm.value.nickname.trim(),
    password: addForm.value.password,
    email: addForm.value.email.trim(),
  })
  addLoading.value = false
  if (res.code === 200) {
    showToast(`添加成功，弦予号: ${res.data ? (res.data as any).ciyuanxi_id : ciyuanxi}`, 'success')
    showAddModal.value = false
    addForm.value = { username: '', nickname: '', password: '', email: '' }
    loadUsers()
  } else {
    showToast(res.msg || '添加失败')
  }
}

// ===== 修改邮箱弹窗 =====
const showEmailModal = ref(false)
const emailLoading = ref(false)
const emailForm = ref({ userId: 0, username: '', nickname: '', currentEmail: '', newEmail: '' })

function openEmailModal(u: User) {
  emailForm.value = { userId: u.id, username: u.nickname || u.username, nickname: u.nickname || u.username, currentEmail: u.email || '', newEmail: '' }
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
const resetForm = ref({ userId: 0, username: '', nickname: '', duration: '', ciyuanxiId: '' })

function openResetModal(u: User) {
  resetForm.value = {
    userId: u.id,
    username: u.nickname || u.username,
    nickname: u.nickname || u.username,
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

// ===== 设备管理 =====
const showDeviceModal = ref(false)
const deviceLoading = ref(false)
const deviceData = ref<any>({})
const showBannedModal = ref(false)
const bannedLoading = ref(false)
const bannedDevices = ref<any[]>([])
const banDeviceInput = ref('')
const banReasonInput = ref('')

async function openDeviceModal(u: User) {
  showDeviceModal.value = true
  deviceLoading.value = true
  deviceData.value = { username: u.nickname || u.username, nickname: u.nickname || u.username }
  const res = await adminApi('get_user_devices', { user_id: u.id })
  if (res.code === 200 && res.data) {
    deviceData.value = res.data
  } else {
    showToast(res.msg || '加载设备信息失败')
  }
  deviceLoading.value = false
}

async function banUserDevice(deviceId: string, username: string) {
  const reason = await webPrompt(`请输入封禁用户 "${username}" 的设备 (${deviceId.substring(0, 16)}...) 的原因：`, '', { title: '封禁设备', placeholder: '封禁原因（必填）' })
  if (reason === null) return // 用户取消，静默退出
  const reasonText = reason.trim()
  if (!reasonText) {
    showToast('封禁原因不能为空')
    return
  }
  const ok = await webConfirm(`确定封禁用户 "${username}" 的设备 (${deviceId.substring(0, 16)}...) 吗？封禁后该设备将无法登录。`, { title: '封禁设备', confirmText: '确认封禁' })
  if (!ok) return
  const res = await adminApi('ban_device', { device_id: deviceId, reason: reasonText })
  if (res.code === 200) {
    showToast('设备已封禁', 'success')
    deviceData.value.is_banned = true
  } else {
    showToast(res.msg || '操作失败')
  }
}

async function unbanUserDevice(deviceId: string) {
  const res = await adminApi('unban_device', { device_id: deviceId })
  if (res.code === 200) {
    showToast('设备已解封', 'success')
    deviceData.value.is_banned = false
  } else {
    showToast(res.msg || '操作失败')
  }
}

async function openBannedDevicesModal() {
  showBannedModal.value = true
  await loadBannedDevices()
}

async function loadBannedDevices() {
  bannedLoading.value = true
  const res = await adminApi('list_banned_devices', { page: 1, page_size: 100 })
  if (res.code === 200 && res.data) {
    bannedDevices.value = res.data.list || []
  } else {
    bannedDevices.value = []
  }
  bannedLoading.value = false
}

async function manualBanDevice() {
  const deviceId = banDeviceInput.value.trim()
  if (!deviceId) {
    showToast('请输入设备ID')
    return
  }
  const reason = banReasonInput.value.trim()
  if (!reason) {
    showToast('封禁原因不能为空')
    return
  }
  const res = await adminApi('ban_device', { device_id: deviceId, reason })
  if (res.code === 200) {
    showToast('设备已封禁', 'success')
    banDeviceInput.value = ''
    banReasonInput.value = ''
    await loadBannedDevices()
  } else {
    showToast(res.msg || '操作失败')
  }
}

async function unbanDeviceById(id: number, deviceId: string) {
  const res = await adminApi('unban_device', { id, device_id: deviceId })
  if (res.code === 200) {
    showToast('设备已解封', 'success')
    await loadBannedDevices()
  } else {
    showToast(res.msg || '操作失败')
  }
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

/* 顶部工具栏：搜索 + 操作合并为一行 */
.toolbar-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
  margin-bottom: 16px;
}
.search-box {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 260px;
}
.search-box input {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid var(--border);
  border-radius: 6px;
  font-size: 13px;
  outline: none;
  min-width: 180px;
  background: var(--white);
}
.search-box input:focus { border-color: var(--accent); }

/* 右侧操作区 */
.toolbar-actions {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}
.toolbar-actions-row {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}
/* 批量菜单打开/关闭切换动效 */
.batch-swap-enter-active,
.batch-swap-leave-active {
  transition: all 0.28s cubic-bezier(0.16, 1, 0.3, 1);
}
.batch-swap-enter-from {
  opacity: 0;
  transform: translateY(-6px) scale(0.97);
}
.batch-swap-leave-to {
  opacity: 0;
  transform: translateY(4px) scale(0.97);
}
.batch-mode-bar {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
  padding: 8px 12px;
  border: 1px solid var(--border);
  border-radius: 12px;
  background: var(--accent-soft);
}
.batch-count {
  font-size: 12px;
  color: var(--accent);
  font-weight: 600;
  margin-left: 2px;
}
.checkbox-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  border: 1.5px solid var(--border);
  border-radius: 5px;
  background: var(--white);
  cursor: pointer;
  transition: all 0.15s;
  vertical-align: -2px;
  margin-right: 6px;
}
.checkbox-badge.checked {
  background: var(--accent);
  border-color: var(--accent);
  color: #fff;
}
.checkbox-badge:hover { border-color: var(--accent); }
.col-check { width: 40px; text-align: center; }
.col-check .checkbox-badge { margin-right: 0; }
.row-selected { background: var(--accent-soft); }
.btn-dark {
  background: var(--accent);
  color: var(--white);
  border-color: var(--accent);
}
.btn-dark:hover { background: #000; border-color: #000; }
.btn-dark:disabled { opacity: 0.6; cursor: not-allowed; }
.btn-warning {
  background: #f39c12;
  color: #fff;
  border-color: #f39c12;
}
.btn-warning:hover { background: #e67e22; border-color: #e67e22; }

/* 设备ID单元格 */
.device-id-cell {
  font-size: 11px;
  font-family: monospace;
  cursor: pointer;
  color: var(--primary);
  transition: opacity 0.15s;
}
.device-id-cell:hover { opacity: 0.7; }

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

/* 设备封禁表单 */
.ban-form {
  display: flex;
  gap: 10px;
  align-items: flex-start;
  margin-bottom: 16px;
  flex-wrap: wrap;
}
.ban-device-input { flex: 1.2; min-width: 180px; margin-bottom: 0; }
.ban-reason-input { flex: 1; min-width: 160px; margin-bottom: 0; }
.ban-submit {
  flex-shrink: 0;
  height: 40px;
  padding: 0 20px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
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

/* 弹窗顶部标题栏 + 关闭按钮 */
.modal-head-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 18px;
  padding-bottom: 14px;
  border-bottom: 1px solid var(--border, #eee);
}
.modal-head-bar h3 {
  margin: 0;
  font-size: 17px;
  font-weight: 700;
}
.modal-close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: 8px;
  border: none;
  background: transparent;
  color: var(--text-muted, #999);
  cursor: pointer;
  transition: all 0.2s;
  flex-shrink: 0;
}
.modal-close-btn:hover {
  background: rgba(0, 0, 0, 0.06);
  color: #e74c3c;
}

/* 弹窗淡进淡出 */
.modal-enter-active, .modal-leave-active { transition: opacity 0.3s ease; }
.modal-enter-from, .modal-leave-to { opacity: 0; }
.modal-enter-active .modal, .modal-leave-active .modal {
  transition: transform 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}
.modal-enter-from .modal, .modal-leave-to .modal {
  transform: scale(0.92) translateY(20px);
}

/* ===== 页面进入动效 ===== */
.fade-down-enter-active, .fade-down-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-down-enter-from { opacity: 0; transform: translateY(-12px); }

.fade-up-enter-active, .fade-up-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-up-enter-from { opacity: 0; transform: translateY(12px); }

/* 表格行逐条加载 */
tbody tr.table-row { animation: rowIn 0.4s cubic-bezier(0.16, 1, 0.3, 1) both; }
@keyframes rowIn {
  from { opacity: 0; transform: translateY(8px); }
  to { opacity: 1; transform: translateY(0); }
}
</style>
