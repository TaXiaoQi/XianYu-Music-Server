<template>
  <div class="mobile-page">
    <!-- 搜索工具栏 -->
    <div class="mobile-card mobile-toolbar">
      <div v-if="!isBatchMode" class="mobile-search-row">
        <input v-model="keyword" class="mobile-input" placeholder="搜索设备ID / 型号 / 弦予号" @keyup.enter="loadDevices" />
        <button class="mobile-btn primary" @click="loadDevices">搜索</button>
        <button class="mobile-btn" @click="showBanForm = !showBanForm">{{ showBanForm ? '收起' : '手动封禁' }}</button>
        <button class="mobile-btn primary" @click="enterBatchMode">批量管理</button>
      </div>
      <div v-else class="mobile-batch-bar">
        <button class="mobile-btn" @click="toggleSelectAll">{{ isAllSelected ? '取消全选' : '全选' }}</button>
        <button class="mobile-btn danger" :disabled="selectedCount === 0 || batchLoading" @click="batchDelete">删除</button>
        <span class="mobile-batch-count">已选 {{ selectedCount }} 项</span>
        <button class="mobile-btn primary" @click="exitBatchMode">完成</button>
      </div>
    </div>

    <!-- 手动封禁表单 -->
    <transition name="expand">
      <div v-if="showBanForm" class="mobile-card mobile-form">
        <h3 class="mobile-card-title">手动封禁设备</h3>
        <input v-model="banDeviceInput" class="mobile-input" placeholder="设备 ID（必填）" />
        <input v-model="banReasonInput" class="mobile-input" placeholder="封禁原因（必填）" @keyup.enter="manualBanDevice" />
        <button class="mobile-btn danger" :disabled="banning" @click="manualBanDevice">
          {{ banning ? '封禁中...' : '确认封禁' }}
        </button>
      </div>
    </transition>

    <!-- 统计 -->
    <div class="mobile-grid">
      <div class="mobile-stat">
        <span>设备总数</span>
        <strong>{{ stats.total }}</strong>
      </div>
      <div class="mobile-stat">
        <span>正常</span>
        <strong>{{ stats.active }}</strong>
      </div>
      <div class="mobile-stat">
        <span>已封禁</span>
        <strong>{{ stats.banned }}</strong>
      </div>
    </div>

    <!-- 设备列表 -->
    <div v-if="loading" class="mobile-empty">加载中...</div>
    <div v-else-if="devices.length === 0" class="mobile-empty">暂无设备数据</div>
    <div v-else class="mobile-list">
      <div v-for="d in devices" :key="d.device_id" class="mobile-item" :class="{ 'batch-selected': isBatchMode && selectedIds.has(d.device_id) }">
        <div class="mobile-item-head">
          <span v-if="isBatchMode" class="mobile-select-badge" :class="{ checked: selectedIds.has(d.device_id) }" @click="toggleSelect(d.device_id)">
            <svg v-if="selectedIds.has(d.device_id)" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
          </span>
          <div class="mobile-item-main">
            <div class="mobile-item-title-row">
              <span class="mobile-item-title">{{ d.device_model || '未知设备' }}</span>
              <span class="mobile-badge" :class="d.ban_id ? 'red' : 'green'">{{ d.ban_id ? '已封禁' : '正常' }}</span>
            </div>
            <div class="mobile-item-sub monospace">{{ d.device_id }}</div>
            <div class="mobile-item-sub">
              <template v-if="d.os_version">{{ d.os_version }}<template v-if="d.app_version"> · v{{ d.app_version }}</template></template>
              <template v-else-if="d.app_version">v{{ d.app_version }}</template>
              <template v-else>-</template>
            </div>
            <div class="mobile-item-sub">
              <template v-if="d.nickname || d.ciyuanxi_id">
                账号：{{ d.nickname || '-' }}
                <template v-if="d.ciyuanxi_id">（{{ d.ciyuanxi_id }}）</template>
              </template>
              <template v-else>账号：未关联</template>
              <span v-if="(d.account_count || 0) > 1" class="mobile-badge" style="margin-left:6px">{{ d.account_count }}个账号</span>
            </div>
            <div class="mobile-item-sub muted-time">最后活跃 {{ d.created_at || '-' }}</div>
          </div>
        </div>
        <div v-if="!isBatchMode" class="mobile-item-foot">
          <div class="mobile-item-foot-info">
            <span v-if="d.ip">IP: {{ d.ip }}</span>
          </div>
          <button class="mobile-op-btn" @click="openActionMenu(d)">操作</button>
        </div>
      </div>
    </div>

    <!-- 设备详情弹窗 -->
    <Transition name="mobile-fade" @before-leave="removeBackdropBlur">
      <div v-if="showDetailModal" class="mobile-dialog-overlay">
        <div class="mobile-dialog" style="display:flex;flex-direction:column;max-width:440px;max-height:88vh;">
          <div class="mobile-dialog-head">
            <span class="mobile-dialog-head-title">设备详情{{ detailData?.device_id ? ` · ${detailData.device_id.substring(0, 20)}` : '' }}</span>
            <button class="mobile-dialog-close" @click="closeDetail">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
          <div class="mobile-dialog-body popup-body">
            <div v-if="detailLoading" class="mobile-empty">加载中...</div>
            <template v-else-if="detailData">
              <!-- 设备信息 -->
              <div class="popup-section">
                <h4 class="popup-sec-title">设备信息</h4>
                <div class="detail-grid">
                  <div class="detail-item"><span class="detail-label">设备ID</span><span class="detail-value mono">{{ detailData.device_id }}</span></div>
                  <template v-if="detailData.device_info">
                    <div class="detail-item"><span class="detail-label">型号</span><span class="detail-value">{{ detailData.device_info.device_model || '-' }}</span></div>
                    <div class="detail-item"><span class="detail-label">系统</span><span class="detail-value">{{ detailData.device_info.os_version || '-' }}</span></div>
                    <div class="detail-item"><span class="detail-label">应用版本</span><span class="detail-value">{{ detailData.device_info.app_version || '-' }}</span></div>
                    <div class="detail-item"><span class="detail-label">IP</span><span class="detail-value mono">{{ detailData.device_info.ip || '-' }}</span></div>
                    <div class="detail-item"><span class="detail-label">最后活跃</span><span class="detail-value">{{ detailData.device_info.created_at || '-' }}</span></div>
                  </template>
                  <div class="detail-item">
                    <span class="detail-label">封禁状态</span>
                    <span v-if="detailData.is_banned" class="badge badge-error">已封禁</span>
                    <span v-else class="badge badge-success">正常</span>
                  </div>
                </div>
                <div v-if="detailData.is_banned && detailData.ban_info" class="ban-detail-box">
                  <div>封禁原因：{{ detailData.ban_info.reason || '-' }}</div>
                  <div>操作人：{{ detailData.ban_info.banned_by || '-' }}</div>
                  <div>封禁时间：{{ detailData.ban_info.created_at || '-' }}</div>
                </div>
              </div>

              <!-- 关联账号 -->
              <div class="popup-section">
                <h4 class="popup-sec-title">关联账号 ({{ detailData.account_count }})</h4>
                <div v-if="detailData.associated_accounts && detailData.associated_accounts.length > 0" class="popup-table-wrap">
                  <table class="popup-table">
                    <thead><tr><th>昵称</th><th>弦予号</th><th>听歌时长</th><th>歌曲数</th><th>状态</th></tr></thead>
                    <tbody>
                      <tr v-for="(a, i) in detailData.associated_accounts" :key="i" :class="{ 'row-current': a.is_current }">
                        <td>{{ a.nickname || '-' }}</td>
                        <td class="td-mono">{{ a.ciyuanxi_id || '-' }}</td>
                        <td>{{ formatDuration(a.listen_duration) }}</td>
                        <td>{{ a.unique_songs_count || 0 }}</td>
                        <td><span v-if="a.is_current" class="badge badge-info">当前</span></td>
                      </tr>
                    </tbody>
                  </table>
                </div>
                <div v-else class="mobile-empty">该设备未关联任何账号</div>
              </div>
            </template>
          </div>
        </div>
      </div>
    </Transition>

    <!-- 插件查看弹窗 -->
    <Transition name="mobile-fade" @before-leave="removeBackdropBlur">
      <div v-if="showPluginsModal" class="mobile-dialog-overlay">
        <div class="mobile-dialog" style="display:flex;flex-direction:column;max-width:440px;max-height:88vh;">
          <div class="mobile-dialog-head">
            <span class="mobile-dialog-head-title">设备插件{{ pluginsData?.nickname ? ` · ${pluginsData.nickname}` : '' }}</span>
            <button class="mobile-dialog-close" @click="closePlugins">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
          <div class="mobile-dialog-body popup-body">
            <div v-if="pluginsLoading" class="mobile-empty">加载中...</div>
            <template v-else>
              <div class="popup-meta">
                <span>插件数量：{{ pluginsData.plugin_count || 0 }}</span>
                <span v-if="pluginsData.uploaded_at">上传时间：{{ pluginsData.uploaded_at }}</span>
                <span v-if="pluginsData.ciyuanxi_id">弦予号：{{ pluginsData.ciyuanxi_id }}</span>
                <span v-if="pluginsData.message" class="mobile-muted">{{ pluginsData.message }}</span>
              </div>
              <div v-if="pluginsData.plugins && pluginsData.plugins.length > 0" class="popup-table-wrap">
                <table class="popup-table">
                  <thead><tr><th>名称</th><th>格式</th><th>版本</th><th>作者</th><th>状态</th><th>大小</th></tr></thead>
                  <tbody>
                    <tr v-for="(p, i) in pluginsData.plugins" :key="i">
                      <td>
                        {{ p.name }}
                        <div v-if="p.description" class="td-desc">{{ p.description }}</div>
                      </td>
                      <td><span class="badge badge-info">{{ p.format }}</span></td>
                      <td>{{ p.version || '-' }}</td>
                      <td>{{ p.author || '-' }}</td>
                      <td><span :class="['badge', p.enabled ? 'badge-success' : 'badge-error']">{{ p.enabled ? '启用' : '禁用' }}</span></td>
                      <td>{{ formatScriptSize(p.scriptSize) }}</td>
                    </tr>
                  </tbody>
                </table>
              </div>
              <div v-else class="mobile-empty">该设备当前关联账号暂无插件数据</div>
            </template>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { adminApi, showToast } from '@/api/client'
import { mobileConfirm, mobilePrompt, mobileActionMenu, removeBackdropBlur } from '@/utils/mobileDialog'
import './MobilePage.css'

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
  [key: string]: any
}

// ===== 列表数据 =====
const devices = ref<Device[]>([])
const loading = ref(true)
const keyword = ref('')
const total = ref(0)

const stats = computed(() => {
  const banned = devices.value.filter(d => d.ban_id).length
  return {
    total: total.value,
    active: total.value - banned,
    banned,
  }
})

async function loadDevices() {
  loading.value = true
  const res = await adminApi<{ total: number; list: Device[] }>('list_all_devices', {
    page: 1,
    page_size: 50,
    keyword: keyword.value,
  })
  if (res.code === 200 && res.data) {
    devices.value = res.data.list || []
    total.value = res.data.total
  } else {
    devices.value = []
    showToast(res.msg || '加载失败')
  }
  loading.value = false
}

// ===== 手动封禁 =====
const showBanForm = ref(false)
const banDeviceInput = ref('')
const banReasonInput = ref('')
const banning = ref(false)

async function manualBanDevice() {
  const deviceId = banDeviceInput.value.trim()
  if (!deviceId) return showToast('请输入设备ID')
  const reason = banReasonInput.value.trim()
  if (!reason) return showToast('封禁原因不能为空')
  if (!(await mobileConfirm(`确定封禁设备 (${deviceId.substring(0, 16)}...) 吗？封禁后该设备将无法登录。`))) return
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
async function openActionMenu(d: Device) {
  const action = await mobileActionMenu(`设备操作 · ${(d.device_model || d.device_id).substring(0, 20)}`, [
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
  const reason = await mobilePrompt(`请输入封禁设备 (${d.device_id.substring(0, 16)}...) 的原因：`, '')
  if (reason === null) return
  const reasonText = reason.trim()
  if (!reasonText) return showToast('封禁原因不能为空')
  if (!(await mobileConfirm('确定封禁此设备吗？封禁后将无法登录。'))) return
  const res = await adminApi('ban_device', { device_id: d.device_id, reason: reasonText })
  if (res.code === 200) {
    showToast('设备已封禁', 'success')
    await loadDevices()
  } else {
    showToast(res.msg || '操作失败')
  }
}

async function unbanDevice(d: Device) {
  if (!(await mobileConfirm(`确定解封设备 (${d.device_id.substring(0, 16)}...) 吗？`))) return
  const res = await adminApi('unban_device', { device_id: d.device_id })
  if (res.code === 200) {
    showToast('设备已解封', 'success')
    await loadDevices()
  } else {
    showToast(res.msg || '操作失败')
  }
}

async function resetListenStats(d: Device) {
  if (!(await mobileConfirm(`确定重置设备 (${d.device_id.substring(0, 16)}...) 上所有关联账号的听歌时长和歌曲数吗？此操作不可恢复。`))) return
  const res = await adminApi('reset_device_listen_stats', { device_id: d.device_id })
  if (res.code === 200) {
    showToast(res.msg || '重置成功', 'success')
    await loadDevices()
  } else {
    showToast(res.msg || '操作失败')
  }
}

async function deleteDevice(d: Device) {
  if (!(await mobileConfirm(`确定删除设备 (${d.device_id.substring(0, 16)}...) 的所有记录吗？包括启动日志和封禁记录，此操作不可恢复。`))) return
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
  if (!(await mobileConfirm(`确定删除选中的 ${ids.length} 台设备的所有记录吗？此操作不可恢复。`))) return
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

function closeDetail() {
  if (!detailLoading.value) showDetailModal.value = false
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

function closePlugins() {
  if (!pluginsLoading.value) showPluginsModal.value = false
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
/* 顶部工具栏 */
.mobile-toolbar {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.mobile-search-row {
  display: flex;
  align-items: center;
  gap: 8px;
}
.mobile-search-row .mobile-input {
  flex: 1;
  min-width: 0;
}
.mobile-search-row .mobile-btn {
  flex: 0 0 auto;
  white-space: nowrap;
}

/* 批量模式 */
.mobile-batch-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  padding: 10px 0 2px;
  border-top: 1px dashed var(--border);
  margin-top: 2px;
}
.mobile-batch-count {
  margin-left: auto;
  font-size: 12px;
  color: var(--accent);
  white-space: nowrap;
}
.mobile-select-badge {
  width: 22px;
  height: 22px;
  flex: none;
  border-radius: 6px;
  border: 1.5px solid var(--border);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  margin-right: 4px;
  cursor: pointer;
  transition: all 0.15s ease;
}
.mobile-select-badge.checked {
  background: var(--accent);
  border-color: var(--accent);
}
.mobile-item.batch-selected {
  border-color: var(--accent);
  background: color-mix(in srgb, var(--accent) 6%, transparent);
}
.mobile-item-head .mobile-select-badge {
  align-self: center;
  margin-top: 0;
}

/* 列表项 */
.mobile-item-main {
  flex: 1;
  min-width: 0;
}
.mobile-item-title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
.mobile-item-foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  margin-top: 10px;
  color: var(--text-muted);
  font-size: 12px;
}
.mobile-item-foot-info {
  display: flex;
  flex-wrap: wrap;
  gap: 4px 12px;
  min-width: 0;
}
.mobile-op-btn {
  flex: 0 0 auto;
  border: none;
  border-radius: 999px;
  padding: 8px 18px;
  background: #EC4141;
  color: #fff;
  font-size: 13px;
  font-weight: 800;
  cursor: pointer;
  box-shadow: 0 4px 12px rgba(236, 65, 65, 0.28);
  transition: transform 0.16s var(--motion, cubic-bezier(0.16, 1, 0.3, 1)),
              opacity 0.16s, box-shadow 0.16s;
}
.mobile-op-btn:active {
  transform: scale(0.94);
  opacity: 0.85;
}

/* 弹窗头部 */
.mobile-dialog-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 16px 18px 0;
}
.mobile-dialog-head-title {
  font-size: 15px;
  font-weight: 850;
  color: var(--text);
  word-break: break-all;
}
.mobile-dialog-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  flex-shrink: 0;
  border: none;
  border-radius: 10px;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.2s;
}
.mobile-dialog-close:hover {
  background: var(--control-bg);
  color: #EC4141;
}

/* 弹窗主体 */
.popup-body {
  padding: 12px 0 0;
  overflow-y: auto;
  max-height: calc(88vh - 60px);
}
.popup-body .mobile-empty {
  padding: 24px 0;
}
.popup-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 6px 14px;
  padding: 0 18px 12px;
  font-size: 12px;
  color: var(--text-light);
}
.popup-section {
  padding: 0 18px 14px;
}
.popup-sec-title {
  margin: 0 0 8px;
  font-size: 13px;
  font-weight: 850;
  color: var(--text);
}

/* 详情网格 */
.detail-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 10px;
}
.detail-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.detail-label {
  font-size: 11px;
  color: var(--text-muted);
}
.detail-value {
  font-size: 13px;
  font-weight: 500;
  word-break: break-all;
}
.mono {
  font-family: ui-monospace, SFMono-Regular, monospace;
  font-size: 12px;
}
.ban-detail-box {
  margin-top: 10px;
  padding: 10px 12px;
  background: rgba(236, 65, 65, 0.08);
  border: 1px solid rgba(236, 65, 65, 0.2);
  border-radius: 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 12px;
  color: #EC4141;
}

/* 子表格 */
.popup-table-wrap {
  overflow-x: auto;
  border: 1px solid var(--border);
  border-radius: 12px;
}
.popup-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 12px;
  min-width: 420px;
}
.popup-table th,
.popup-table td {
  padding: 9px 10px;
  text-align: left;
  border-bottom: 1px solid var(--border);
  white-space: nowrap;
}
.popup-table thead th {
  background: var(--control-bg);
  color: var(--text-light);
  font-weight: 800;
  font-size: 11px;
}
.popup-table tbody tr:last-child td {
  border-bottom: 0;
}
.popup-table tbody tr.row-current {
  background: color-mix(in srgb, var(--accent) 6%, transparent);
}
.td-desc {
  font-size: 11px;
  color: var(--text-muted);
  white-space: normal;
}
.td-mono {
  font-family: ui-monospace, SFMono-Regular, monospace;
  font-size: 11px;
}
.badge {
  display: inline-block;
  padding: 2px 8px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 800;
}
.badge-info { background: rgba(59, 130, 246, 0.10); color: #3b82f6; }
.badge-success { background: rgba(34, 197, 94, 0.12); color: #16a34a; }
.badge-error { background: rgba(236, 65, 65, 0.10); color: #EC4141; }
</style>
