<template>
  <div class="device-banned-page">
    <!-- 页面头部 -->
    <Transition name="fade-down" appear>
      <div class="page-header">
        <div class="header-info">
          <h2 class="page-title">设备管理</h2>
          <p class="page-desc">手动封禁或解封登录设备，被封禁的设备将无法登录客户端。</p>
        </div>
        <button class="btn-refresh" @click="loadBannedDevices" :disabled="bannedLoading">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" :class="{ spinning: bannedLoading }">
            <polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
          </svg>
          刷新
        </button>
      </div>
    </Transition>

    <!-- 手动封禁表单 -->
    <Transition name="fade-up" appear>
      <div class="ban-panel">
        <div class="ban-icon">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
        </div>
        <div class="ban-info">
          <h3>手动封禁设备</h3>
          <p>输入设备 ID 与封禁原因，被禁设备将无法登录。</p>
        </div>
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

    <!-- 封禁列表 -->
    <Transition name="fade-up" appear>
      <div class="table-card">
        <div v-if="bannedLoading" class="state-box">
          <div class="spinner"></div>
          <span>加载中...</span>
        </div>
        <Transition name="fade-up" appear v-else-if="bannedDevices.length === 0">
          <div class="state-box state-empty">
            <div class="empty-icon">
              <svg width="56" height="56" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
            </div>
            <p class="empty-title">暂无封禁设备</p>
            <p class="empty-sub">所有已封禁的设备会显示在这里</p>
          </div>
        </Transition>
        <div v-else class="table-wrap">
          <table class="data-table">
            <thead>
              <tr>
                <th>ID</th>
                <th>设备ID</th>
                <th>硬件型号</th>
                <th>系统版本</th>
                <th>所属账号</th>
                <th>应用版本</th>
                <th>原因</th>
                <th>操作人</th>
                <th>封禁时间</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(d, idx) in bannedDevices" :key="d.id" class="table-row" :style="{ animationDelay: `${idx * 40}ms` }">
                <td class="col-id">{{ d.id }}</td>
                <td class="col-device" :title="d.device_id">{{ d.device_id }}</td>
                <td class="col-model" :title="d.device_model || '-'">{{ d.device_model || '-' }}</td>
                <td class="col-os" :title="d.os_version || '-'">{{ d.os_version || '-' }}</td>
                <td class="col-account" :title="(d.nickname || '') + (d.ciyuanxi_id ? '（' + d.ciyuanxi_id + '）' : '')">
                  <span v-if="d.nickname || d.ciyuanxi_id" class="account-cell">
                    <span class="account-name">{{ d.nickname || '-' }}</span>
                    <span v-if="d.ciyuanxi_id" class="account-id">{{ d.ciyuanxi_id }}</span>
                  </span>
                  <span v-else class="muted">未关联</span>
                </td>
                <td class="col-version" :title="d.app_version || '-'">{{ d.app_version || '-' }}</td>
                <td class="col-reason">{{ d.reason || '-' }}</td>
                <td>{{ d.banned_by || '-' }}</td>
                <td class="col-time">{{ d.created_at }}</td>
                <td>
                  <button class="btn-unban" @click="unbanDeviceById(d.id, d.device_id)">解封</button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { adminApi, showToast } from '@/api/client'
import { webConfirm } from '@/utils/webDialog'

const bannedLoading = ref(false)
const bannedDevices = ref<any[]>([])
const banDeviceInput = ref('')
const banReasonInput = ref('')
const banning = ref(false)

async function loadBannedDevices() {
  bannedLoading.value = true
  const res = await adminApi<{ list: any[] }>('list_banned_devices', { page: 1, page_size: 100 })
  if (res.code === 200 && res.data) {
    bannedDevices.value = res.data.list || []
  } else {
    bannedDevices.value = []
    if (res.code !== 200) showToast(res.msg || '加载失败')
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
  const ok = await webConfirm(`确定封禁设备 (${deviceId.substring(0, 16)}...) 吗？封禁后该设备将无法登录。`, {
    title: '封禁设备',
    confirmText: '确认封禁',
  })
  if (!ok) return
  banning.value = true
  const res = await adminApi('ban_device', { device_id: deviceId, reason })
  banning.value = false
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
  const ok = await webConfirm(`确定解封设备 (${deviceId.substring(0, 16)}...) 吗？`, {
    title: '解封设备',
    confirmText: '确认解封',
  })
  if (!ok) return
  const res = await adminApi('unban_device', { id, device_id: deviceId })
  if (res.code === 200) {
    showToast('设备已解封', 'success')
    await loadBannedDevices()
  } else {
    showToast(res.msg || '操作失败')
  }
}

onMounted(loadBannedDevices)
</script>

<style scoped>
.device-banned-page {
  max-width: 1100px;
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
.btn-refresh {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 10px 18px;
  border-radius: 10px;
  border: 1px solid var(--border);
  background: var(--white);
  color: var(--text);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}
.btn-refresh:hover { border-color: var(--accent); transform: translateY(-1px); }
.btn-refresh:active { transform: scale(0.96); }
.btn-refresh:disabled { opacity: 0.5; cursor: not-allowed; }
.spinning { animation: spin 0.8s linear infinite; }
@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }

/* ===== 手动封禁表单 ===== */
.ban-panel {
  display: flex;
  align-items: center;
  gap: 14px;
  background: linear-gradient(135deg, #fff 0%, #fef2f2 100%);
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 16px 18px;
  margin-bottom: 20px;
  box-shadow: 0 6px 20px rgba(15, 23, 42, 0.04);
  flex-wrap: wrap;
}
.ban-icon {
  width: 42px;
  height: 42px;
  border-radius: 12px;
  background: #fef2f2;
  color: #dc2626;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.ban-info h3 {
  font-size: 15px;
  font-weight: 800;
  color: var(--text);
  margin: 0 0 4px 0;
  white-space: nowrap;
}
.ban-info p {
  font-size: 12px;
  color: var(--text-muted);
  margin: 0;
  line-height: 1.5;
  white-space: nowrap;
}
.ban-form {
  display: flex;
  gap: 10px;
  align-items: center;
  margin-left: auto;
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
.btn-ban:hover:not(:disabled) { background: #b91c1c; transform: translateY(-1px); }
.btn-ban:active:not(:disabled) { transform: scale(0.96); }
.btn-ban:disabled { opacity: 0.55; cursor: not-allowed; }
.btn-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: #fff;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

/* ===== 表格 ===== */
.table-card {
  background: var(--white);
  border: 1px solid var(--border);
  border-radius: 14px;
  overflow: hidden;
}
.table-wrap { overflow-x: auto; -webkit-overflow-scrolling: touch; }
.data-table {
  width: 100%;
  border-collapse: collapse;
  background: var(--white);
  min-width: 1080px;
}
.data-table th {
  padding: 12px 14px;
  text-align: left;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-muted);
  background: #fafafa;
  border-bottom: 1px solid var(--border);
  white-space: nowrap;
}
.data-table td {
  padding: 12px 14px;
  font-size: 13px;
  color: var(--text);
  border-bottom: 1px solid #f5f5f5;
  vertical-align: middle;
}
.data-table tr.table-row { animation: rowIn 0.4s cubic-bezier(0.16, 1, 0.3, 1) both; }
.data-table tr.table-row:hover td { background: #fafbfc; }
@keyframes rowIn {
  from { opacity: 0; transform: translateY(8px); }
  to { opacity: 1; transform: translateY(0); }
}
.col-id { font-weight: 600; color: var(--text-muted); white-space: nowrap; }
.col-device {
  font-family: monospace;
  font-size: 11px;
  word-break: break-all;
  max-width: 220px;
}
.col-model { color: var(--text); white-space: nowrap; max-width: 140px; overflow: hidden; text-overflow: ellipsis; font-size: 13px; }
.col-os { color: var(--text); white-space: nowrap; font-size: 12px; }
.col-version { color: var(--text-muted); white-space: nowrap; font-size: 12px; }
.col-account { min-width: 120px; }
.account-cell { display: flex; flex-direction: column; gap: 2px; }
.account-name { color: var(--text); font-size: 13px; font-weight: 600; max-width: 160px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.account-id { color: var(--text-muted); font-size: 11px; font-family: monospace; }
.muted { color: var(--text-muted); }
.col-reason { color: var(--text-light); max-width: 180px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.col-time { white-space: nowrap; font-size: 12px; color: var(--text-light); }
.btn-unban {
  padding: 6px 14px;
  border-radius: 8px;
  border: none;
  background: #f0fdf4;
  color: #16a34a;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}
.btn-unban:hover { background: #dcfce7; }

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
.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid #e5e5e5;
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

/* ===== 过渡动画 ===== */
.fade-down-enter-active, .fade-down-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-down-enter-from { opacity: 0; transform: translateY(-12px); }
.fade-up-enter-active, .fade-up-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-up-enter-from { opacity: 0; transform: translateY(12px); }

/* ===== 响应式 ===== */
@media (max-width: 768px) {
  .ban-form { width: 100%; margin-left: 0; }
  .ban-input { flex: 1 1 40%; }
  .ban-info { flex: 1 1 200px; }
}
</style>