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
        <button class="btn btn-dark" @click="showBanForm = !showBanForm">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
          手动封禁
        </button>
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
              <th>设备ID</th>
              <th>硬件型号</th>
              <th>系统版本</th>
              <th>应用版本</th>
              <th>所属账号</th>
              <th>最后活跃</th>
              <th>状态</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(d, idx) in devices" :key="d.device_id" class="table-row" :style="{ animationDelay: `${idx * 30}ms` }">
              <td class="col-device" :title="d.device_id">{{ d.device_id }}</td>
              <td class="col-model">{{ d.device_model || '-' }}</td>
              <td class="col-os">{{ d.os_version || '-' }}</td>
              <td class="col-version">{{ d.app_version || '-' }}</td>
              <td class="col-account">
                <span v-if="d.nickname || d.ciyuanxi_id" class="account-cell">
                  <span class="account-name">{{ d.nickname || '-' }}</span>
                  <span v-if="d.ciyuanxi_id" class="account-id">{{ d.ciyuanxi_id }}</span>
                </span>
                <span v-else class="muted">未关联</span>
              </td>
              <td class="col-time">{{ d.created_at || '-' }}</td>
              <td>
                <span v-if="d.ban_id" class="badge badge-error">已封禁</span>
                <span v-else class="badge badge-success">正常</span>
              </td>
              <td>
                <button v-if="!d.ban_id" class="btn btn-sm btn-danger" @click="banDevice(d)">封禁</button>
                <button v-else class="btn btn-sm btn-success" @click="unbanDevice(d)">解封</button>
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
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { adminApi, showToast } from '@/api/client'
import { webConfirm, webPrompt } from '@/utils/webDialog'

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

// ===== 加载设备列表 =====
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

// ===== 列表内封禁/解封 =====
async function banDevice(d: Device) {
  const reason = await webPrompt(`请输入封禁设备 (${d.device_id.substring(0, 16)}...) 的原因：`, '', {
    title: '封禁设备', placeholder: '封禁原因（必填）',
  })
  if (reason === null) return
  const reasonText = reason.trim()
  if (!reasonText) { showToast('封禁原因不能为空'); return }
  const ok = await webConfirm(`确定封禁此设备吗？封禁后将无法登录。`, {
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
  background: var(--white);
}
.search-box input:focus { border-color: var(--accent); }
.toolbar-actions { display: flex; align-items: center; gap: 10px; }

/* 手动封禁面板 */
.ban-panel {
  background: linear-gradient(135deg, #fff 0%, #fef2f2 100%);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 14px 16px;
  margin-bottom: 16px;
}
.ban-form {
  display: flex;
  gap: 10px;
  align-items: center;
  flex-wrap: wrap;
}
.ban-input {
  height: 38px;
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 0 12px;
  font-size: 13px;
  min-width: 180px;
  outline: none;
  background: var(--white);
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
  background: var(--white);
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
.stat-icon-banned { background: #fef2f2; color: #dc2626; }
.stat-body { display: flex; flex-direction: column; }
.stat-num { font-size: 22px; font-weight: 800; line-height: 1.2; }
.stat-label { font-size: 12px; color: var(--text-muted); }

/* 卡片 */
.card {
  background: var(--white);
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

.col-device { font-family: monospace; font-size: 11px; word-break: break-all; max-width: 200px; }
.col-model { white-space: nowrap; max-width: 140px; overflow: hidden; text-overflow: ellipsis; }
.col-os { white-space: nowrap; font-size: 12px; }
.col-version { white-space: nowrap; font-size: 12px; color: var(--text-muted); }
.col-account { min-width: 120px; }
.account-cell { display: flex; flex-direction: column; gap: 2px; }
.account-name { font-size: 13px; font-weight: 600; max-width: 160px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.account-id { font-size: 11px; font-family: monospace; color: var(--text-muted); }
.muted { color: var(--text-muted); }
.col-time { white-space: nowrap; font-size: 12px; color: var(--text-light); }

/* 徽章 */
.badge { display: inline-flex; padding: 3px 10px; border-radius: 20px; font-size: 11px; font-weight: 600; }
.badge-success { background: #f0fdf4; color: #16a34a; }
.badge-error { background: #fef2f2; color: #dc2626; }

/* 按钮 */
.btn { display: inline-flex; align-items: center; gap: 6px; padding: 8px 16px; border-radius: 8px; border: 1px solid var(--border); background: var(--white); color: var(--text); font-size: 13px; font-weight: 600; cursor: pointer; transition: all 0.2s; white-space: nowrap; }
.btn:hover:not(:disabled) { border-color: var(--accent); color: var(--accent); }
.btn-primary { background: var(--accent); color: #fff; border-color: var(--accent); }
.btn-primary:hover:not(:disabled) { background: #000; border-color: #000; color: #fff; }
.btn-dark { background: var(--accent); color: #fff; border-color: var(--accent); }
.btn-dark:hover:not(:disabled) { background: #000; border-color: #000; }
.btn-danger { background: #fef2f2; color: #dc2626; border-color: #fecaca; }
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

/* 分页 */
.pagination {
  display: flex; justify-content: center; gap: 6px; margin-top: 16px; align-items: center;
  padding: 0 0 16px;
}
.pagination button {
  padding: 6px 12px; border: 1px solid var(--border); background: var(--white);
  border-radius: 4px; cursor: pointer; font-size: 12px; transition: all 0.15s;
}
.pagination button:hover:not(.active):not(:disabled) { border-color: var(--accent); color: var(--accent); }
.pagination button.active { background: var(--accent); color: #fff; border-color: var(--accent); }
.pagination button:disabled { opacity: 0.4; cursor: not-allowed; }
.pagination span { font-size: 12px; color: var(--text-muted); margin-left: 8px; }

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
