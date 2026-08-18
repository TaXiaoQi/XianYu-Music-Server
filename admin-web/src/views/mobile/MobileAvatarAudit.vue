<template>
  <div class="mobile-page">
    <!-- 页头 -->
    <div class="audit-header">
      <div class="audit-header-info">
        <h2 class="audit-title">
          头像/改名审核
          <span v-if="stats.pending > 0" class="pending-badge">{{ stats.pending }} 项待审核</span>
        </h2>
        <p class="audit-desc">审核用户上传的头像和改名申请，通过后立即生效。</p>
      </div>
      <button class="refresh-btn" :disabled="loading" @click="loadAll()">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" :class="{ spin: loading }">
          <polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
        </svg>
      </button>
    </div>

    <!-- 统计卡片 -->
    <div class="stat-grid">
      <div class="stat-chip pending" :class="{ active: tab === 'pending' }" @click="switchTab('pending')">
        <span class="stat-num">{{ stats.pending }}</span>
        <span class="stat-label">待审核</span>
      </div>
      <div class="stat-chip approved" :class="{ active: tab === 'approved' }" @click="switchTab('approved')">
        <span class="stat-num">{{ stats.approved }}</span>
        <span class="stat-label">已通过</span>
      </div>
      <div class="stat-chip rejected" :class="{ active: tab === 'rejected' }" @click="switchTab('rejected')">
        <span class="stat-num">{{ stats.rejected }}</span>
        <span class="stat-label">已拒绝</span>
      </div>
    </div>

    <!-- tabs -->
    <div class="mobile-tabs">
      <button class="mobile-btn" :class="{ primary: tab === 'pending' }" @click="switchTab('pending')">待审核 {{ stats.pending }}</button>
      <button class="mobile-btn" :class="{ primary: tab === 'approved' }" @click="switchTab('approved')">已通过 {{ stats.approved }}</button>
      <button class="mobile-btn" :class="{ primary: tab === 'rejected' }" @click="switchTab('rejected')">已拒绝 {{ stats.rejected }}</button>
    </div>

    <!-- 加载中 -->
    <div v-if="loading" class="mobile-empty">加载中...</div>

    <!-- 空状态 -->
    <div v-else-if="currentList.length === 0" class="mobile-empty">
      {{ emptyText }}
    </div>

    <!-- 记录列表 -->
    <div v-else class="mobile-list">
      <div v-for="item in currentList" :key="item.type + '-' + item.id" class="mobile-item audit-item">
        <!-- 类型 + 用户信息 -->
        <div class="mobile-item-head">
          <span class="audit-type-badge" :class="'t-' + item.type">{{ item.type === 'avatar' ? '头像' : '改名' }}</span>
          <span class="mobile-item-title">{{ item.username || item.ciyuanxi_id || '未知用户' }}</span>
        </div>
        <div class="mobile-item-sub">弦予号：{{ item.ciyuanxi_id || '-' }} · {{ fmtDateTime(item.created_at) || '-' }}</div>

        <!-- 头像对比 -->
        <div v-if="item.type === 'avatar'" class="avatar-compare">
          <div class="av-box">
            <img v-if="item.avatar_data" :src="item.avatar_data" :alt="'新头像'" class="av-img" @error="onImgError" />
            <div v-else class="av-fallback">
              <svg width="30" height="30" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
            </div>
            <span class="av-label label-new">新头像</span>
          </div>
          <span class="av-vs">VS</span>
          <div class="av-box">
            <img v-if="item.current_avatar" :src="item.current_avatar" :alt="'当前头像'" class="av-img av-img-old" @error="onImgError" />
            <div v-else class="av-fallback av-fallback-old">
              <svg width="26" height="26" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
            </div>
            <span class="av-label label-old">当前</span>
          </div>
        </div>

        <!-- 改名对比 -->
        <div v-else class="nick-change">
          <span class="nick-old">{{ item.old_name || '未知' }}</span>
          <span class="nick-arrow">→</span>
          <span class="nick-new">{{ item.new_name }}</span>
        </div>

        <!-- 操作 / 状态 -->
        <div v-if="item.status === 'pending'" class="mobile-actions">
          <button class="mobile-btn primary" @click="handleApprove(item)">通过</button>
          <button class="mobile-btn danger" @click="handleReject(item)">拒绝</button>
        </div>
        <div v-else class="history-status">
          <span class="mobile-badge" :class="item.status === 'approved' ? 'green' : 'red'">{{ item.status === 'approved' ? '已通过' : '已拒绝' }}</span>
          <span class="history-meta">审核人：{{ item.reviewed_by || '-' }}<template v-if="item.reviewed_at"> · {{ fmtDateTime(item.reviewed_at) }}</template></span>
        </div>
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'
import { mobileConfirm } from '@/utils/mobileDialog'
import { fmtDateTime } from '@/utils/time'
import './MobilePage.css'

type TabKey = 'pending' | 'approved' | 'rejected'

const tab = ref<TabKey>('pending')
const loading = ref(true)
const records = reactive<Record<TabKey, any[]>>({ pending: [], approved: [], rejected: [] })
const loadedTabs = reactive<Record<TabKey, boolean>>({ pending: false, approved: false, rejected: false })
const stats = reactive({ pending: 0, approved: 0, rejected: 0 })

const currentList = computed(() => records[tab.value])
const emptyText = computed(() => {
  if (tab.value === 'pending') return '暂无待审核记录'
  if (tab.value === 'approved') return '暂无已通过记录'
  return '暂无已拒绝记录'
})

async function loadTab(t: TabKey, silent = false) {
  if (!silent) loading.value = true
  const res = await adminApi<any>('list_audit_records', { status: t })
  if (res.code === 200 && res.data) {
    records[t] = res.data.list || []
    loadedTabs[t] = true
    const s = res.data.stats
    if (s) {
      stats.pending = s.pending ?? 0
      stats.approved = s.approved ?? 0
      stats.rejected = s.rejected ?? 0
    }
  } else records[t] = []
  if (!silent) loading.value = false
}

function switchTab(t: TabKey) {
  if (tab.value === t) return
  tab.value = t
  if (!loadedTabs[t]) loadTab(t)
}

async function loadAll(silent = false) {
  if (!silent) loading.value = true
  await loadTab('pending', true)
  if (loadedTabs.approved) await loadTab('approved', true)
  if (loadedTabs.rejected) await loadTab('rejected', true)
  if (!silent) loading.value = false
}

async function handleApprove(i: any) {
  const isAvatar = i.type === 'avatar'
  const label = isAvatar ? '头像' : '改名'
  const ok = await mobileConfirm(`确认通过该${label}审核？${isAvatar ? '通过后将更新为用户的新头像。' : '通过后用户名将更新。'}`, { title: '通过审核', confirmText: '确认通过' })
  if (!ok) return
  const res = await adminApi(isAvatar ? 'approve_avatar' : 'approve_nickname', { id: i.id })
  if (res.code === 200) {
    showToast('审核通过', 'success')
    records.pending = records.pending.filter(r => !(r.type === i.type && r.id === i.id))
    stats.pending = Math.max(0, stats.pending - 1)
    stats.approved += 1
    loadedTabs.approved = false
  } else showToast(res.msg || '操作失败')
}

async function handleReject(i: any) {
  const isAvatar = i.type === 'avatar'
  const label = isAvatar ? '头像' : '改名'
  const ok = await mobileConfirm(`确认拒绝该${label}审核？`, { title: '拒绝审核', confirmText: '确认拒绝', danger: true })
  if (!ok) return
  const res = await adminApi(isAvatar ? 'reject_avatar' : 'reject_nickname', { id: i.id })
  if (res.code === 200) {
    showToast('已拒绝', 'success')
    records.pending = records.pending.filter(r => !(r.type === i.type && r.id === i.id))
    stats.pending = Math.max(0, stats.pending - 1)
    stats.rejected += 1
    loadedTabs.rejected = false
  } else showToast(res.msg || '操作失败')
}

function onImgError(e: Event) {
  const img = e.target as HTMLImageElement
  img.style.display = 'none'
}

let pollTimer: ReturnType<typeof setInterval> | null = null
function startPolling() { stopPolling(); pollTimer = setInterval(() => loadAll(true), 30000) }
function stopPolling() { if (pollTimer) { clearInterval(pollTimer); pollTimer = null } }
onMounted(() => { loadAll(); startPolling() })
onUnmounted(() => stopPolling())
</script>
<style scoped>
.audit-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}
.audit-header-info { min-width: 0; }
.audit-title {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  font-size: 18px;
  font-weight: 850;
  margin: 0 0 4px;
  color: var(--text);
}
.pending-badge {
  font-size: 11px;
  font-weight: 700;
  padding: 3px 9px;
  border-radius: 20px;
  background: #fffbeb;
  color: #f59e0b;
  animation: badgePulse 2s ease-in-out infinite;
}
@keyframes badgePulse {
  0%, 100% { box-shadow: 0 0 0 0 rgba(245, 158, 11, 0.3); }
  50% { box-shadow: 0 0 0 5px rgba(245, 158, 11, 0); }
}
.audit-desc {
  font-size: 12px;
  color: var(--text-light);
  line-height: 1.6;
  margin: 0;
}
.refresh-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 38px;
  height: 38px;
  flex-shrink: 0;
  border-radius: 12px;
  border: 1px solid var(--border);
  background: var(--card);
  color: var(--text);
  cursor: pointer;
  transition: all 0.2s;
}
.refresh-btn:active { transform: scale(0.92); }
.refresh-btn:disabled { opacity: 0.5; }
.spin { animation: spin 0.8s linear infinite; }
@keyframes spin { from { transform: rotate(0); } to { transform: rotate(360deg); } }

.stat-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 8px;
}
.stat-chip {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 12px 10px;
  border-radius: 14px;
  background: var(--card);
  border: 1px solid var(--border);
  cursor: pointer;
  transition: all 0.2s;
}
.stat-chip.active { border-color: var(--accent); box-shadow: 0 0 0 2px var(--accent-soft); }
.stat-num { font-size: 20px; font-weight: 850; line-height: 1.1; }
.stat-label { font-size: 11px; color: var(--text-muted); }
.stat-chip.pending .stat-num { color: #f59e0b; }
.stat-chip.approved .stat-num { color: #16a34a; }
.stat-chip.rejected .stat-num { color: #dc2626; }

/* 记录卡片 */
.audit-item { display: flex; flex-direction: column; gap: 6px; }
.audit-type-badge {
  flex-shrink: 0;
  font-size: 10px;
  font-weight: 800;
  padding: 2px 8px;
  border-radius: 6px;
}
.audit-type-badge.t-avatar { background: #eff6ff; color: #2563eb; }
.audit-type-badge.t-nickname { background: #f5f3ff; color: #7c3aed; }

.avatar-compare {
  display: flex;
  align-items: center;
  gap: 14px;
  margin: 8px 0;
}
.av-box { display: flex; flex-direction: column; align-items: center; gap: 6px; }
.av-img {
  width: 72px; height: 72px;
  border-radius: 50%;
  object-fit: cover;
  border: 3px solid var(--border);
  background: var(--control-bg);
}
.av-img-old { width: 52px; height: 52px; opacity: 0.7; border-width: 2px; }
.av-fallback {
  width: 72px; height: 72px;
  border-radius: 50%;
  background: var(--control-bg);
  display: flex; align-items: center; justify-content: center;
  color: var(--text-muted);
  border: 3px solid var(--border);
}
.av-fallback-old { width: 52px; height: 52px; border-width: 2px; }
.av-vs { font-size: 11px; font-weight: 800; color: var(--text-muted); letter-spacing: 0.05em; }
.av-label { font-size: 10px; font-weight: 700; padding: 2px 8px; border-radius: 8px; }
.label-new { background: #eff6ff; color: #2563eb; }
.label-old { background: var(--control-bg); color: var(--text-muted); }

.nick-change {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  border-radius: 14px;
  background: var(--control-bg);
  border: 1px solid var(--border);
  margin: 8px 0;
}
.nick-old { flex: 1; min-width: 0; font-size: 13px; font-weight: 700; color: var(--text-muted); text-decoration: line-through; word-break: break-word; }
.nick-arrow { flex: 0 0 auto; color: #EC4141; font-weight: 900; }
.nick-new { flex: 1; min-width: 0; font-size: 13px; font-weight: 800; color: var(--text); word-break: break-word; }

.history-status {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  margin-top: 4px;
}
.history-meta {
  font-size: 11px;
  color: var(--text-muted);
  word-break: break-all;
}
</style>
