<template>
  <div class="device-page">
    <!-- 顶部工具栏 -->
    <Transition name="fade-down" appear>
      <div class="toolbar-row">
        <div class="search-box">
          <input
            v-model="keyword"
            type="text"
            placeholder="搜索设备ID、型号、弦予号或昵称"
            @keyup.enter="handleSearch"
          />
          <button class="btn btn-primary" @click="handleSearch">搜索</button>
          <button v-if="keyword" class="btn" @click="clearSearch">清除</button>
        </div>

        <div class="toolbar-actions">
          <Transition name="batch-swap" mode="out-in">
            <div v-if="!isBatchMode" key="normal" class="toolbar-actions-row">
              <button class="btn btn-dark" @click="showBanForm = !showBanForm">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
                手动封禁
              </button>
              <button class="btn btn-primary" @click="enterBatchMode">批量管理</button>
            </div>

            <div v-else key="batch" class="batch-mode-bar">
              <button class="btn btn-sm" @click="toggleSelectAll">
                <span class="checkbox-badge" :class="{ checked: isAllSelected }">
                  <svg v-if="isAllSelected" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
                </span>
                {{ isAllSelected ? '取消全选' : '全选' }}
              </button>
              <button class="btn btn-sm btn-danger" @click="batchDelete" :disabled="selectedCount === 0 || batchLoading">删除</button>
              <span class="batch-count">已选 {{ selectedCount }} 项</span>
              <button class="btn btn-sm btn-primary" @click="exitBatchMode">完成</button>
            </div>
          </Transition>
        </div>
      </div>
    </Transition>

    <!-- 手动封禁表单（折叠） -->
    <Transition name="fade-up">
      <div v-if="showBanForm" class="ban-panel">
        <div class="ban-form">
          <input v-model="banDeviceInput" type="text" placeholder="设备 ID（必填）" class="ban-input" />
          <input v-model="banReasonInput" type="text" placeholder="封禁原因（必填）" class="ban-input" @keyup.enter="manualBanDevice" />
          <button class="btn-ban" :disabled="banning" @click="manualBanDevice">
            <span v-if="banning" class="btn-spinner"></span>
            {{ banning ? '封禁中...' : '封禁' }}
          </button>
        </div>
      </div>
    </Transition>

    <!-- 统计行 -->
    <Transition name="fade-up" appear>
      <div class="stats-row">
        <div class="stat-chip">
          <div class="stat-icon stat-icon-total">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="5" y="2" width="14" height="20" rx="2"/><line x1="12" y1="18" x2="12.01" y2="18"/></svg>
          </div>
          <div class="stat-body"><span class="stat-num">{{ stats.total }}</span><span class="stat-label">设备总数</span></div>
        </div>
        <div class="stat-chip">
          <div class="stat-icon stat-icon-active">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
          </div>
          <div class="stat-body"><span class="stat-num">{{ stats.active }}</span><span class="stat-label">正常</span></div>
        </div>
        <div class="stat-chip">
          <div class="stat-icon stat-icon-banned">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="4.93" y1="4.93" x2="19.07" y2="19.07"/></svg>
          </div>
          <div class="stat-body"><span class="stat-num">{{ stats.banned }}</span><span class="stat-label">已封禁</span></div>
        </div>
      </div>
    </Transition>

    <!-- 设备表格 -->
    <Transition name="fade-up" appear>
      <div class="card">
        <div v-if="loading" class="state-box">
          <div class="spinner"></div>
          <span>加载中...</span>
        </div>
        <div v-else-if="devices.length === 0" class="state-box state-empty">
          <div class="empty-icon">
            <svg width="56" height="56" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2"><rect x="5" y="2" width="14" height="20" rx="2"/><line x1="12" y1="18" x2="12.01" y2="18"/></svg>
          </div>
          <p class="empty-title">暂无设备数据</p>
          <p class="empty-sub">所有已注册的设备会显示在这里</p>
        </div>
        <div v-else class="table-wrapper">
          <table>
            <thead>
              <tr>
                <th v-if="isBatchMode" class="col-check">
                  <span class="checkbox-badge" :class="{ checked: isAllSelected }" @click="toggleSelectAll">
                    <svg v-if="isAllSelected" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
                  </span>
                </th>
                <th>设备</th>
                <th>系统版本</th>
                <th>应用版本</th>
                <th>关联账号</th>
                <th>最后活跃</th>
                <th>状态</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(d, idx) in devices" :key="d.device_id" class="table-row" :class="{ 'row-selected': isBatchMode && selectedIds.has(d.device_id) }" :style="{ animationDelay: `${idx * 30}ms` }">
                <td v-if="isBatchMode" class="col-check">
                  <span class="checkbox-badge" :class="{ checked: selectedIds.has(d.device_id) }" @click="toggleSelect(d.device_id)">
                    <svg v-if="selectedIds.has(d.device_id)" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
                  </span>
                </td>
                <td class="col-device-cell">
                  <div class="device-cell">
                    <span class="device-model">{{ d.device_model || '未知型号' }}</span>
                    <span class="device-id" :title="d.device_id">{{ d.device_id }}</span>
                  </div>
                </td>
                <td class="col-os">{{ d.os_version || '-' }}</td>
                <td class="col-version">{{ d.app_version || '-' }}</td>
                <td class="col-account">
                  <span v-if="d.nickname || d.ciyuanxi_id" class="account-cell">
                    <span class="account-name">{{ d.nickname || '-' }}</span>
                    <span v-if="d.ciyuanxi_id" class="account-id">{{ d.ciyuanxi_id }}</span>
                    <span v-if="(d.account_count || 0) > 1" class="account-badge">{{ d.account_count }}个账号</span>
                  </span>
                  <span v-else class="muted">未关联</span>
                </td>
                <td class="col-time">{{ fmtDateTime(d.created_at) || '-' }}</td>
                <td>
                  <span v-if="d.ban_id" class="badge badge-error">已封禁</span>
                  <span v-else class="badge badge-success">正常</span>
                </td>
                <td>
                  <button class="btn btn-sm btn-primary" @click="openRowMenu(d)">操作</button>
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

    <!-- 设备详情弹窗 -->
    <Transition name="modal">
      <div v-if="showDetailModal" class="modal-overlay">
        <div class="modal" style="max-width:900px;">
          <div class="modal-head-bar">
            <h3>设备详情 {{ detailData?.device_id ? `· ${detailData.device_id.substring(0, 20)}` : '' }}</h3>
            <button class="modal-close-btn" @click="showDetailModal = false">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
          <div v-if="detailLoading" class="state-box"><div class="spinner"></div><span>加载中...</span></div>
          <div v-else-if="detailData">
            <!-- 设备信息 -->
            <div class="detail-section">
              <h4>设备信息</h4>
              <div class="detail-grid">
                <div class="detail-item"><span class="detail-label">设备ID</span><span class="detail-value mono">{{ detailData.device_id }}</span></div>
                <div v-if="detailData.device_info" class="detail-item">
                  <span class="detail-label">型号</span><span class="detail-value">{{ detailData.device_info.device_model || '-' }}</span>
                </div>
                <div v-if="detailData.device_info" class="detail-item">
                  <span class="detail-label">系统</span><span class="detail-value">{{ detailData.device_info.os_version || '-' }}</span>
                </div>
                <div v-if="detailData.device_info" class="detail-item">
                  <span class="detail-label">应用版本</span><span class="detail-value">{{ detailData.device_info.app_version || '-' }}</span>
                </div>
                <div v-if="detailData.device_info" class="detail-item">
                  <span class="detail-label">IP</span><span class="detail-value mono">{{ detailData.device_info.ip || '-' }}</span>
                </div>
                <div v-if="detailData.device_info" class="detail-item">
                  <span class="detail-label">最后活跃</span><span class="detail-value">{{ fmtDateTime(detailData.device_info.created_at) || '-' }}</span>
                </div>
                <div class="detail-item">
                  <span class="detail-label">封禁状态</span>
                  <span v-if="detailData.is_banned" class="badge badge-error">已封禁</span>
                  <span v-else class="badge badge-success">正常</span>
                </div>
              </div>
              <div v-if="detailData.is_banned && detailData.ban_info" class="ban-detail-box">
                <span>封禁原因: {{ detailData.ban_info.reason || '-' }}</span>
                <span>操作人: {{ detailData.ban_info.banned_by || '-' }}</span>
                <span>封禁时间: {{ fmtDateTime(detailData.ban_info.created_at) || '-' }}</span>
              </div>
            </div>

            <!-- 关联账号 -->
            <div class="detail-section">
              <h4>关联账号 ({{ detailData.account_count }})</h4>
              <div v-if="detailData.associated_accounts && detailData.associated_accounts.length > 0" class="table-wrapper">
                <table class="sub-table">
                  <thead>
                    <tr><th>昵称</th><th>弦予号</th><th>听歌时长</th><th>歌曲数</th><th>状态</th></tr>
                  </thead>
                  <tbody>
                    <tr v-for="(a, i) in detailData.associated_accounts" :key="i" :class="{ 'row-current': a.is_current }">
                      <td>{{ a.nickname || '-' }}</td>
                      <td class="mono">{{ a.ciyuanxi_id || '-' }}</td>
                      <td>{{ formatDuration(a.listen_duration) }}</td>
                      <td>{{ a.unique_songs_count || 0 }}</td>
                      <td><span v-if="a.is_current" class="badge badge-info">当前关联</span></td>
                    </tr>
                  </tbody>
                </table>
              </div>
              <div v-else class="empty">该设备未关联任何账号</div>
            </div>
          </div>
        </div>
      </div>
    </Transition>

    <!-- 插件查看弹窗 -->
    <Transition name="modal">
      <div v-if="showPluginsModal" class="modal-overlay">
        <div class="modal" style="max-width:900px;">
          <div class="modal-head-bar">
            <h3>设备插件 {{ pluginsData?.nickname ? `· ${pluginsData.nickname}` : '' }}</h3>
            <button class="modal-close-btn" @click="showPluginsModal = false">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
          <div v-if="pluginsLoading" class="state-box"><div class="spinner"></div><span>加载中...</span></div>
          <div v-else>
            <div class="plugins-meta">
              <span>插件数量: {{ pluginsData.plugin_count || 0 }}</span>
              <span v-if="pluginsData.uploaded_at">上传时间: {{ fmtDateTime(pluginsData.uploaded_at) }}</span>
              <span v-if="pluginsData.ciyuanxi_id">弦予号: {{ pluginsData.ciyuanxi_id }}</span>
              <span v-if="pluginsData.message" class="muted">{{ pluginsData.message }}</span>
            </div>
            <div v-if="pluginsData.plugins && pluginsData.plugins.length > 0" class="table-wrapper">
              <table class="sub-table">
                <thead><tr><th>名称</th><th>格式</th><th>版本</th><th>作者</th><th>状态</th><th>大小</th></tr></thead>
                <tbody>
                  <tr v-for="(p, i) in pluginsData.plugins" :key="i">
                    <td>{{ p.name }}<div v-if="p.description" class="sub-desc">{{ p.description }}</div></td>
                    <td><span class="badge badge-info">{{ p.format }}</span></td>
                    <td>{{ p.version || '-' }}</td>
                    <td>{{ p.author || '-' }}</td>
                    <td><span :class="['badge', p.enabled ? 'badge-success' : 'badge-error']">{{ p.enabled ? '启用' : '禁用' }}</span></td>
                    <td>{{ formatScriptSize(p.scriptSize) }}</td>
                  </tr>
                </tbody>
              </table>
            </div>
            <div v-else class="empty">该设备当前关联账号暂无插件数据</div>
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
import { fmtDateTime } from '@/utils/time'

interface Device {
  device_id: string
  device_model: string
  os_version: string
  app_version: string
  ciyuanxi_id: string
  nickname: string
  ip: string
  created_at: string
  ban_id: number | null
  ban_reason: string
  account_count?: number
  current_account_count?: number
  [key: string]: any
}

// ===== 列表数据 =====
const devices = ref<Device[]>([])
const loading = ref(true)
const keyword = ref('')
const page = ref(1)
const pageSize = 20
const total = ref(0)
const totalPages = ref(0)

const stats = computed(() => {
  const banned = devices.value.filter(d => d.ban_id).length
  return {
    total: total.value,
    active: total.value - banned,
    banned,
  }
})

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

async function loadDevices() {
  loading.value = true
  const res = await adminApi<{ total: number; total_pages: number; list: Device[] }>('list_all_devices', {
    page: page.value,
    page_size: pageSize,
    keyword: keyword.value,
  })
  if (res.code === 200 && res.data) {
    devices.value = res.data.list || []
    total.value = res.data.total
    totalPages.value = res.data.total_pages
  } else {
    devices.value = []
    showToast(res.msg || '加载失败')
  }
  loading.value = false
}

function handleSearch() {
  page.value = 1
  loadDevices()
}

function clearSearch() {
  keyword.value = ''
  page.value = 1
  loadDevices()
}

function goPage(p: number) {
  if (p < 1 || p > totalPages.value || p === page.value) return
  page.value = p
  loadDevices()
}

// ===== 手动封禁 =====
const showBanForm = ref(false)
const banDeviceInput = ref('')
const banReasonInput = ref('')
const banning = ref(false)

async function manualBanDevice() {
  const deviceId = banDeviceInput.value.trim()
  if (!deviceId) { showToast('请输入设备ID'); return }
  const reason = banReasonInput.value.trim()
  if (!reason) { showToast('封禁原因不能为空'); return }
  const ok = await webConfirm(`确定封禁设备 (${deviceId.substring(0, 16)}...) 吗？封禁后该设备将无法登录。`, {
    title: '封禁设备', confirmText: '确认封禁',
  })
  if (!ok) return
  banning.value = true
  const res = await adminApi('ban_device', { device_id: deviceId, reason })
  banning.value = false
  if (res.code === 200) {
    showToast('设备已封禁', 'success')
    banDeviceInput.value = ''
    banReasonInput.value = ''
    await loadDevices()
  } else {
    showToast(res.msg || '操作失败')
  }
}

// ===== 行操作菜单 =====
async function openRowMenu(d: Device) {
  const action = await webActionMenu(`设备操作 · ${d.device_id.substring(0, 20)}`, [
    { key: 'detail', label: '设备详情' },
    { key: 'plugins', label: '查看插件' },
    { key: 'reset', label: '重置听歌统计' },
    { key: 'ban', label: '封禁设备', danger: true, show: !d.ban_id },
    { key: 'unban', label: '解封设备', success: true, show: !!d.ban_id },
    { key: 'delete', label: '删除设备记录', danger: true },
  ])
  if (!action) return
  switch (action) {
    case 'detail': await openDetailModal(d); break
    case 'plugins': await openPluginsModal(d); break
    case 'reset': await resetListenStats(d); break
    case 'ban': await banDevice(d); break
    case 'unban': await unbanDevice(d); break
    case 'delete': await deleteDevice(d); break
  }
}

async function banDevice(d: Device) {
  const reason = await webPrompt(`请输入封禁设备 (${d.device_id.substring(0, 16)}...) 的原因：`, '', {
    title: '封禁设备', placeholder: '封禁原因（必填）',
  })
  if (reason === null) return
  const reasonText = reason.trim()
  if (!reasonText) { showToast('封禁原因不能为空'); return }
  const ok = await webConfirm('确定封禁此设备吗？封禁后将无法登录。', {
    title: '封禁设备', confirmText: '确认封禁',
  })
  if (!ok) return
  const res = await adminApi('ban_device', { device_id: d.device_id, reason: reasonText })
  if (res.code === 200) {
    showToast('设备已封禁', 'success')
    await loadDevices()
  } else {
    showToast(res.msg || '操作失败')
  }
}

async function unbanDevice(d: Device) {
  const ok = await webConfirm(`确定解封设备 (${d.device_id.substring(0, 16)}...) 吗？`, {
    title: '解封设备', confirmText: '确认解封',
  })
  if (!ok) return
  const res = await adminApi('unban_device', { device_id: d.device_id })
  if (res.code === 200) {
    showToast('设备已解封', 'success')
    await loadDevices()
  } else {
    showToast(res.msg || '操作失败')
  }
}

async function resetListenStats(d: Device) {
  const ok = await webConfirm(
    `确定重置设备 (${d.device_id.substring(0, 16)}...) 上所有关联账号的听歌时长和歌曲数吗？此操作不可恢复。`,
    { title: '重置听歌统计', confirmText: '确认重置' },
  )
  if (!ok) return
  const res = await adminApi('reset_device_listen_stats', { device_id: d.device_id })
  if (res.code === 200) {
    showToast(res.msg || '重置成功', 'success')
    await loadDevices()
  } else {
    showToast(res.msg || '操作失败')
  }
}

async function deleteDevice(d: Device) {
  const ok = await webConfirm(
    `确定删除设备 (${d.device_id.substring(0, 16)}...) 的所有记录吗？包括启动日志和封禁记录，此操作不可恢复。`,
    { title: '删除设备记录', confirmText: '确认删除' },
  )
  if (!ok) return
  const res = await adminApi('delete_device_record', { device_id: d.device_id })
  if (res.code === 200) {
    showToast('设备记录已删除', 'success')
    await loadDevices()
  } else {
    showToast(res.msg || '操作失败')
  }
}

// ===== 批量模式 =====
const isBatchMode = ref(false)
const selectedIds = ref<Set<string>>(new Set())
const selectedCount = computed(() => selectedIds.value.size)
const isAllSelected = computed(() => devices.value.length > 0 && selectedIds.value.size === devices.value.length)
const batchLoading = ref(false)

function enterBatchMode() {
  selectedIds.value = new Set()
  isBatchMode.value = true
}

function exitBatchMode() {
  isBatchMode.value = false
  selectedIds.value = new Set()
}

function toggleSelect(id: string) {
  const next = new Set(selectedIds.value)
  if (next.has(id)) next.delete(id)
  else next.add(id)
  selectedIds.value = next
}

function toggleSelectAll() {
  if (isAllSelected.value) {
    selectedIds.value = new Set()
  } else {
    selectedIds.value = new Set(devices.value.map(d => d.device_id))
  }
}

async function batchDelete() {
  const ids = [...selectedIds.value]
  if (ids.length === 0) return
  const ok = await webConfirm(`确定删除选中的 ${ids.length} 台设备的所有记录吗？此操作不可恢复。`, {
    title: '批量删除', confirmText: '确认删除',
  })
  if (!ok) return
  batchLoading.value = true
  const res = await adminApi('batch_delete_devices', { device_ids: ids })
  batchLoading.value = false
  if (res.code === 200) {
    showToast(res.msg || '批量删除成功', 'success')
    selectedIds.value = new Set()
    await loadDevices()
  } else {
    showToast(res.msg || '操作失败')
  }
}

// ===== 设备详情弹窗 =====
const showDetailModal = ref(false)
const detailLoading = ref(false)
const detailData = ref<any>(null)

async function openDetailModal(d: Device) {
  showDetailModal.value = true
  detailLoading.value = true
  detailData.value = { device_id: d.device_id }
  const res = await adminApi('get_device_detail', { device_id: d.device_id })
  detailLoading.value = false
  if (res.code === 200 && res.data) {
    detailData.value = res.data
  } else {
    showToast(res.msg || '加载失败')
  }
}

// ===== 插件查看弹窗 =====
const showPluginsModal = ref(false)
const pluginsLoading = ref(false)
const pluginsData = ref<any>({})

async function openPluginsModal(d: Device) {
  showPluginsModal.value = true
  pluginsLoading.value = true
  pluginsData.value = { device_id: d.device_id }
  const res = await adminApi('get_device_plugins', { device_id: d.device_id })
  pluginsLoading.value = false
  if (res.code === 200 && res.data) {
    pluginsData.value = res.data
  } else {
    showToast(res.msg || '加载失败')
  }
}

// ===== 工具函数 =====
function formatDuration(seconds: number): string {
  if (!seconds || seconds === 0) return '0分钟'
  const h = Math.floor(seconds / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  if (h > 0) return `${h}小时${m}分钟`
  return `${m}分钟`
}

function formatScriptSize(bytes: number): string {
  if (!bytes) return '-'
  if (bytes < 1024) return `${bytes}B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)}KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)}MB`
}

onMounted(loadDevices)
</script>

<style scoped>
.device-page {
  max-width: 1320px;
  margin: 0 auto;
}

/* 顶部工具栏 */
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
  background: var(--card-solid);
}
.search-box input:focus { border-color: var(--accent); }

.toolbar-actions { display: flex; align-items: center; gap: 10px; flex-wrap: wrap; }
.toolbar-actions-row { display: flex; align-items: center; gap: 10px; flex-wrap: wrap; }

/* 批量菜单切换动效 */
.batch-swap-enter-active, .batch-swap-leave-active { transition: all 0.28s cubic-bezier(0.16, 1, 0.3, 1); }
.batch-swap-enter-from { opacity: 0; transform: translateY(-6px) scale(0.97); }
.batch-swap-leave-to { opacity: 0; transform: translateY(4px) scale(0.97); }
.batch-mode-bar {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
  padding: 8px 12px;
  border: 1px solid var(--border);
  border-radius: 12px;
  background: var(--accent-soft, rgba(0,0,0,0.03));
}
.batch-count { font-size: 12px; color: var(--accent); font-weight: 600; margin-left: 2px; }

/* 复选框 */
.checkbox-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  border: 1.5px solid var(--border);
  border-radius: 5px;
  background: var(--card-solid);
  cursor: pointer;
  transition: all 0.15s;
  vertical-align: -2px;
  margin-right: 6px;
}
.checkbox-badge.checked { background: var(--accent); border-color: var(--accent); color: #fff; }
.checkbox-badge:hover { border-color: var(--accent); }
.col-check { width: 40px; text-align: center; }
.col-check .checkbox-badge { margin-right: 0; }
.row-selected { background: var(--accent-soft, rgba(0,0,0,0.03)); }

/* 手动封禁面板 */
.ban-panel {
  background: linear-gradient(135deg, var(--control-bg) 0%, rgba(236, 65, 65, 0.10) 100%);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 14px 16px;
  margin-bottom: 16px;
}
.ban-form { display: flex; gap: 10px; align-items: center; flex-wrap: wrap; }
.ban-input {
  height: 38px;
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 0 12px;
  font-size: 13px;
  min-width: 180px;
  outline: none;
  background: var(--card-solid);
  transition: border-color 0.2s;
}
.ban-input:focus { border-color: #dc2626; }
.btn-ban {
  height: 38px;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 0 18px;
  border-radius: 10px;
  border: none;
  background: #dc2626;
  color: #fff;
  font-size: 13px;
  font-weight: 700;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.2s;
}
.btn-ban:hover:not(:disabled) { background: #b91c1c; }
.btn-ban:disabled { opacity: 0.55; cursor: not-allowed; }
.btn-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: #fff;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

/* 统计行 */
.stats-row {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 12px;
  margin-bottom: 16px;
}
.stat-chip {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 18px;
  background: var(--card-solid);
  border: 1px solid var(--border);
  border-radius: 14px;
  transition: transform 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}
.stat-chip:hover { transform: translateY(-2px); }
.stat-icon {
  width: 38px; height: 38px;
  border-radius: 10px;
  display: flex; align-items: center; justify-content: center;
  flex-shrink: 0;
}
.stat-icon-total { background: #eff6ff; color: #3b82f6; }
.stat-icon-active { background: #f0fdf4; color: #16a34a; }
.stat-icon-banned { background: rgba(236, 65, 65, 0.12); color: #dc2626; }
.stat-body { display: flex; flex-direction: column; }
.stat-num { font-size: 22px; font-weight: 800; line-height: 1.2; }
.stat-label { font-size: 12px; color: var(--text-muted); }

/* 卡片 */
.card {
  background: var(--card-solid);
  border: 1px solid var(--border);
  border-radius: 14px;
  overflow: hidden;
}
.table-wrapper { overflow-x: auto; -webkit-overflow-scrolling: touch; }
table { width: 100%; border-collapse: collapse; min-width: 960px; }
thead th {
  padding: 12px 14px;
  text-align: left;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-muted);
  background: #fafafa;
  border-bottom: 1px solid var(--border);
  white-space: nowrap;
}
tbody td {
  padding: 12px 14px;
  font-size: 13px;
  color: var(--text);
  border-bottom: 1px solid #f5f5f5;
  vertical-align: middle;
}
tbody tr.table-row { animation: rowIn 0.4s cubic-bezier(0.16, 1, 0.3, 1) both; }
tbody tr:hover td { background: #fafbfc; }
@keyframes rowIn { from { opacity: 0; transform: translateY(8px); } to { opacity: 1; transform: translateY(0); } }

.col-device-cell { min-width: 200px; max-width: 240px; }
.device-cell { display: flex; flex-direction: column; gap: 3px; min-width: 0; }
.device-model {
  font-size: 13px;
  font-weight: 600;
  color: var(--text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 220px;
}
.device-id {
  font-family: monospace;
  font-size: 11px;
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 220px;
}
.col-os { white-space: nowrap; font-size: 12px; }
.col-version { white-space: nowrap; font-size: 12px; color: var(--text-muted); }
.col-account { min-width: 120px; }
.account-cell { display: flex; flex-direction: column; gap: 2px; }
.account-name { font-size: 13px; font-weight: 600; max-width: 160px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.account-id { font-size: 11px; font-family: monospace; color: var(--text-muted); }
.account-badge { font-size: 10px; color: var(--accent); background: var(--accent-soft, rgba(0,0,0,0.04)); padding: 1px 6px; border-radius: 8px; width: fit-content; }
.muted { color: var(--text-muted); }
.col-time { white-space: nowrap; font-size: 12px; color: var(--text-light); }

/* 徽章 */
.badge { display: inline-flex; padding: 3px 10px; border-radius: 20px; font-size: 11px; font-weight: 600; }
.badge-success { background: #f0fdf4; color: #16a34a; }
.badge-error { background: rgba(236, 65, 65, 0.12); color: #dc2626; }
.badge-info { background: #eff6ff; color: #3b82f6; }

/* 按钮 */
.btn { display: inline-flex; align-items: center; gap: 6px; padding: 8px 16px; border-radius: 8px; border: 1px solid var(--border); background: var(--card-solid); color: var(--text); font-size: 13px; font-weight: 600; cursor: pointer; transition: all 0.2s; white-space: nowrap; }
.btn:hover:not(:disabled) { border-color: var(--accent); color: var(--accent); }
.btn-primary { background: var(--accent); color: #fff; border-color: var(--accent); }
.btn-primary:hover:not(:disabled) { background: #000; border-color: #000; color: #fff; }
.btn-dark { background: var(--accent); color: #fff; border-color: var(--accent); }
.btn-dark:hover:not(:disabled) { background: #000; border-color: #000; }
.btn-danger { background: rgba(236, 65, 65, 0.12); color: #dc2626; border-color: rgba(236, 65, 65, 0.25); }
.btn-danger:hover:not(:disabled) { background: #dc2626; color: #fff; border-color: #dc2626; }
.btn-success { background: #f0fdf4; color: #16a34a; border-color: #bbf7d0; }
.btn-success:hover:not(:disabled) { background: #16a34a; color: #fff; border-color: #16a34a; }
.btn-sm { padding: 5px 12px; font-size: 12px; border-radius: 6px; }
.btn:disabled { opacity: 0.5; cursor: not-allowed; }

/* 状态/空态 */
.state-box {
  display: flex; flex-direction: column; align-items: center; justify-content: center;
  padding: 60px 20px; color: var(--text-muted); gap: 12px; font-size: 14px;
}
.state-empty { padding: 48px 20px; }
.empty-icon { color: #d0d0d0; }
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
.empty { padding: 32px 20px; text-align: center; color: var(--text-muted); font-size: 13px; }

/* 分页 */
.pagination {
  display: flex; justify-content: center; gap: 6px; margin-top: 16px; align-items: center;
  padding: 0 0 16px;
}
.pagination button {
  padding: 6px 12px; border: 1px solid var(--border); background: var(--card-solid);
  border-radius: 4px; cursor: pointer; font-size: 12px; transition: all 0.15s;
}
.pagination button:hover:not(.active):not(:disabled) { border-color: var(--accent); color: var(--accent); }
.pagination button.active { background: var(--accent); color: #fff; border-color: var(--accent); }
.pagination button:disabled { opacity: 0.4; cursor: not-allowed; }
.pagination span { font-size: 12px; color: var(--text-muted); margin-left: 8px; }

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
  background: var(--card-solid);
  border-radius: 8px;
  padding: 28px;
  width: 500px;
  max-width: 100%;
  max-height: 85vh;
  overflow-y: auto;
  border: 1px solid var(--border);
}
.modal h3 { font-size: 17px; margin-bottom: 20px; font-weight: 700; }
.modal-head-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 18px;
  padding-bottom: 14px;
  border-bottom: 1px solid var(--border, #eee);
}
.modal-head-bar h3 { margin: 0; font-size: 17px; font-weight: 700; }
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
.modal-close-btn:hover { background: rgba(0, 0, 0, 0.06); color: #e74c3c; }

/* 详情弹窗内容 */
.detail-section { margin-bottom: 24px; }
.detail-section h4 { font-size: 14px; font-weight: 700; margin-bottom: 12px; color: var(--text); }
.detail-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 12px;
}
.detail-item { display: flex; flex-direction: column; gap: 2px; }
.detail-label { font-size: 11px; color: var(--text-muted); }
.detail-value { font-size: 13px; font-weight: 500; }
.mono { font-family: monospace; font-size: 12px; }
.ban-detail-box {
  margin-top: 12px;
  padding: 10px 14px;
  background: rgba(236, 65, 65, 0.12);
  border: 1px solid #fecaca;
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 12px;
  color: #991b1b;
}

/* 子表格 */
.sub-table { min-width: 600px; }
.sub-table thead th { font-size: 11px; padding: 8px 10px; }
.sub-table tbody td { font-size: 12px; padding: 8px 10px; }
.sub-desc { font-size: 11px; color: #999; margin-top: 2px; }
.row-current { background: var(--accent-soft, rgba(0,0,0,0.03)); }
.plugins-meta {
  display: flex; gap: 16px; margin-bottom: 16px; font-size: 13px; color: #666; flex-wrap: wrap;
}

/* 弹窗动画 */
.modal-enter-active, .modal-leave-active { transition: opacity 0.3s ease; }
.modal-enter-from, .modal-leave-to { opacity: 0; }
.modal-enter-active .modal, .modal-leave-active .modal {
  transition: transform 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}
.modal-enter-from .modal, .modal-leave-to .modal {
  transform: scale(0.92) translateY(20px);
}

/* 过渡 */
.fade-up-enter-active, .fade-up-leave-active { transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-up-enter-from, .fade-up-leave-to { opacity: 0; transform: translateY(8px); }
.fade-down-enter-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-down-enter-from { opacity: 0; transform: translateY(-12px); }

/* 响应式 */
@media (max-width: 768px) {
  .stats-row { grid-template-columns: 1fr; gap: 8px; }
  .toolbar-row { flex-direction: column; }
  .search-box { width: 100%; }
  .ban-form { width: 100%; }
  .ban-input { flex: 1; min-width: 0; }
}
</style>
