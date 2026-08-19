<template>
  <div class="aa-page">
    <!-- 页面头部 -->
    <Transition name="fade-down" appear>
      <div class="page-header">
        <div class="header-info">
          <h2 class="page-title">
            头像/改名审核
            <span v-if="stats.pending > 0" class="pending-badge">{{ stats.pending }} 项待审核</span>
          </h2>
          <p class="page-desc">
            审核用户上传的头像和改名申请。通过后头像将更新为用户的新头像，改名申请通过后用户名将立即生效。
          </p>
        </div>
        <button class="btn-refresh" @click="loadAll()" :disabled="loading">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" :class="{ spinning: loading }">
            <polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
          </svg>
          刷新
        </button>
      </div>
    </Transition>

    <!-- 统计卡片 -->
    <Transition name="fade-up" appear>
      <div class="stats-row">
        <div class="stat-chip stat-pending" :class="{ pulse: stats.pending > 0, active: activeTab === 'pending' }" @click="switchTab('pending')">
          <div class="stat-icon">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
          </div>
          <div class="stat-body">
            <span class="stat-num">{{ stats.pending }}</span>
            <span class="stat-label">待审核</span>
          </div>
        </div>
        <div class="stat-chip stat-approved" :class="{ active: activeTab === 'approved' }" @click="switchTab('approved')">
          <div class="stat-icon">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
          </div>
          <div class="stat-body">
            <span class="stat-num">{{ stats.approved }}</span>
            <span class="stat-label">已通过</span>
          </div>
        </div>
        <div class="stat-chip stat-rejected" :class="{ active: activeTab === 'rejected' }" @click="switchTab('rejected')">
          <div class="stat-icon">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>
          </div>
          <div class="stat-body">
            <span class="stat-num">{{ stats.rejected }}</span>
            <span class="stat-label">已拒绝</span>
          </div>
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
      <Transition name="fade-up" appear v-if="currentList.length === 0">
        <div class="state-box state-empty">
          <div class="empty-icon">
            <svg width="56" height="56" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/>
            </svg>
          </div>
          <p class="empty-title">{{ emptyText }}</p>
          <p class="empty-sub">{{ emptySubText }}</p>
        </div>
      </Transition>

      <!-- 审核记录列表 -->
      <TransitionGroup v-else name="audit-card" tag="div" class="audit-list">
        <div
          v-for="(item, idx) in currentList"
          :key="item.type + '-' + item.id"
          class="audit-card"
          :class="[item.type, item.status]"
          :style="{ animationDelay: `${Math.min(idx, 12) * 50}ms` }"
        >
          <!-- 卡片顶部：类型 + 用户信息 -->
          <div class="audit-card-top">
            <span class="audit-type-badge" :class="'t-' + item.type">{{ item.type === 'avatar' ? '头像' : '改名' }}</span>
            <div class="audit-user">
              <strong>{{ item.username || item.ciyuanxi_id || '未知用户' }}</strong>
              <span>弦予号：{{ item.ciyuanxi_id || '-' }}</span>
            </div>
            <span class="audit-time">{{ fmtDateTime(item.created_at) || '-' }}</span>
          </div>

          <!-- 卡片主体：内容对比 -->
          <div class="audit-body">
            <!-- 头像对比 -->
            <div v-if="item.type === 'avatar'" class="avatar-compare">
              <div class="avatar-new">
                <img v-if="item.avatar_data" :src="item.avatar_data" :alt="'新头像'" class="avatar-img" @error="onImgError" />
                <div v-else class="avatar-fallback">
                  <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
                </div>
                <span class="compare-label label-new">新头像</span>
              </div>
              <div class="compare-vs">VS</div>
              <div class="avatar-old">
                <img v-if="item.current_avatar" :src="item.current_avatar" :alt="'当前头像'" class="avatar-img avatar-img-old" @error="onImgError" />
                <div v-else class="avatar-fallback avatar-fallback-old">
                  <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
                </div>
                <span class="compare-label label-old">当前</span>
              </div>
            </div>

            <!-- 改名对比 -->
            <div v-else class="nick-names">
              <span class="nick-old">{{ item.old_name || '未知' }}</span>
              <svg class="nick-arrow" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="5" y1="12" x2="19" y2="12"/><polyline points="12 5 19 12 12 19"/></svg>
              <span class="nick-new">{{ item.new_name }}</span>
            </div>
          </div>

          <!-- 卡片底部：操作 / 状态 -->
          <div class="audit-card-foot">
            <template v-if="item.status === 'pending'">
              <button class="act-btn act-approve" @click="handleApprove(item)">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12"/></svg>
                通过
              </button>
              <button class="act-btn act-reject" @click="handleReject(item)">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
                拒绝
              </button>
            </template>
            <template v-else>
              <span class="history-status" :class="item.status">
                <span class="status-dot"></span>
                {{ item.status === 'approved' ? '已通过' : '已拒绝' }}
              </span>
              <span class="history-meta">
                审核人：{{ item.reviewed_by || '-' }}
                <template v-if="item.reviewed_at"> · {{ fmtDateTime(item.reviewed_at) }}</template>
              </span>
            </template>
          </div>
        </div>
      </TransitionGroup>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted } from 'vue'
import { adminApi, showToast } from '@/api/client'
import { webConfirm } from '@/utils/webDialog'
import { fmtDateTime } from '@/utils/time'

interface AuditRecord {
  type: 'avatar' | 'nickname'
  id: number
  ciyuanxi_id: string
  username: string
  avatar_data: string
  current_avatar: string
  old_name: string
  new_name: string
  status: 'pending' | 'approved' | 'rejected'
  created_at: string
  reviewed_at: string
  reviewed_by: string
  [key: string]: any
}

type TabKey = 'pending' | 'approved' | 'rejected'

// ===== 状态 =====
const loading = ref(true)
const activeTab = ref<TabKey>('pending')
const records = reactive<Record<TabKey, AuditRecord[]>>({ pending: [], approved: [], rejected: [] })
const loadedTabs = reactive<Record<TabKey, boolean>>({ pending: false, approved: false, rejected: false })
const stats = reactive({ pending: 0, approved: 0, rejected: 0 })

const currentList = computed(() => records[activeTab.value])
const emptyText = computed(() => {
  if (activeTab.value === 'pending') return '暂无待审核记录'
  if (activeTab.value === 'approved') return '暂无已通过记录'
  return '暂无已拒绝记录'
})
const emptySubText = computed(() => {
  if (activeTab.value === 'pending') return '用户上传头像或提交改名申请后将出现在此处'
  return '审核过的头像与改名记录将保留在此处'
})

// ===== 加载数据 =====
async function loadTab(tab: TabKey, silent = false) {
  if (!silent) loading.value = true
  const res = await adminApi<{ list: AuditRecord[]; stats: { pending: number; approved: number; rejected: number } }>('list_audit_records', { status: tab })
  if (res.code === 200 && res.data) {
    records[tab] = res.data.list || []
    loadedTabs[tab] = true
    const s = res.data.stats
    if (s) {
      stats.pending = s.pending ?? 0
      stats.approved = s.approved ?? 0
      stats.rejected = s.rejected ?? 0
    }
  } else {
    records[tab] = []
  }
  if (!silent) loading.value = false
}

function switchTab(tab: TabKey) {
  if (activeTab.value === tab) return
  activeTab.value = tab
  if (!loadedTabs[tab]) {
    loadTab(tab)
  }
}

async function loadAll(silent = false) {
  if (!silent) loading.value = true
  // 待审核 + 统计始终刷新；历史标签按需加载
  await loadTab('pending', true)
  if (loadedTabs.approved) await loadTab('approved', true)
  if (loadedTabs.rejected) await loadTab('rejected', true)
  if (!silent) loading.value = false
}

// ===== 审核操作 =====
async function handleApprove(item: AuditRecord) {
  const isAvatar = item.type === 'avatar'
  const label = isAvatar ? '头像' : '改名'
  const ok = await webConfirm(`确认通过该${label}审核？${isAvatar ? '通过后将更新为用户的新头像。' : '通过后用户名将更新。'}`, { title: '通过审核', confirmText: '确认通过' })
  if (!ok) return
  const res = await adminApi(isAvatar ? 'approve_avatar' : 'approve_nickname', { id: item.id })
  if (res.code === 200) {
    showToast('审核通过', 'success')
    records.pending = records.pending.filter(r => !(r.type === item.type && r.id === item.id))
    stats.pending = Math.max(0, stats.pending - 1)
    stats.approved += 1
    loadedTabs.approved = false
  } else {
    showToast(res.msg || '操作失败')
  }
}

async function handleReject(item: AuditRecord) {
  const isAvatar = item.type === 'avatar'
  const label = isAvatar ? '头像' : '改名'
  const ok = await webConfirm(`确认拒绝该${label}审核？`, { title: '拒绝审核', confirmText: '确认拒绝' })
  if (!ok) return
  const res = await adminApi(isAvatar ? 'reject_avatar' : 'reject_nickname', { id: item.id })
  if (res.code === 200) {
    showToast('已拒绝', 'success')
    records.pending = records.pending.filter(r => !(r.type === item.type && r.id === item.id))
    stats.pending = Math.max(0, stats.pending - 1)
    stats.rejected += 1
    loadedTabs.rejected = false
  } else {
    showToast(res.msg || '操作失败')
  }
}

// ===== 工具 =====
function onImgError(e: Event) {
  const img = e.target as HTMLImageElement
  img.style.display = 'none'
}

// ===== 自动刷新 =====
const POLL_INTERVAL = 30000 // 30秒
let pollTimer: ReturnType<typeof setInterval> | null = null

function startPolling() {
  stopPolling()
  pollTimer = setInterval(() => {
    loadAll(true) // 静默刷新，不显示 loading
  }, POLL_INTERVAL)
}

function stopPolling() {
  if (pollTimer) {
    clearInterval(pollTimer)
    pollTimer = null
  }
}

onMounted(() => {
  loadAll()
  startPolling()
})

onUnmounted(() => {
  stopPolling()
})
</script>

<style scoped>
.aa-page {
  max-width: 1320px;
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
  display: flex;
  align-items: center;
  gap: 10px;
}
.pending-badge {
  font-size: 12px;
  font-weight: 600;
  padding: 3px 10px;
  border-radius: 20px;
  background: rgba(245, 158, 11, 0.14);
  color: #f59e0b;
  animation: badgePulse 2s ease-in-out infinite;
}
@keyframes badgePulse {
  0%, 100% { box-shadow: 0 0 0 0 rgba(245, 158, 11, 0.3); }
  50% { box-shadow: 0 0 0 6px rgba(245, 158, 11, 0); }
}
.page-desc {
  font-size: 13px;
  color: var(--text-muted);
  line-height: 1.6;
  margin: 0;
  max-width: 620px;
}

.btn-refresh {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 10px 18px;
  border-radius: 10px;
  border: 1px solid var(--border);
  background: var(--card-solid);
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

/* ===== 统计卡片 ===== */
.stats-row {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 14px;
  margin-bottom: 20px;
}
.stat-chip {
  background: var(--card-solid);
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 18px 20px;
  display: flex;
  align-items: center;
  gap: 14px;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}
.stat-chip:hover { transform: translateY(-3px); box-shadow: 0 8px 24px rgba(0, 0, 0, 0.06); }
.stat-chip.active { border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-soft); }
.stat-icon {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.stat-pending .stat-icon { background: rgba(245, 158, 11, 0.14); color: #f59e0b; }
.stat-approved .stat-icon { background: #f0fdf4; color: #16a34a; }
.stat-rejected .stat-icon { background: rgba(236, 65, 65, 0.12); color: #dc2626; }

.stat-pending.pulse { border-color: #fbbf24; }
.stat-pending.pulse .stat-icon { animation: iconPulse 2s ease-in-out infinite; }
@keyframes iconPulse {
  0%, 100% { transform: scale(1); }
  50% { transform: scale(1.08); }
}

.stat-body { display: flex; flex-direction: column; }
.stat-num { font-size: 26px; font-weight: 800; line-height: 1.1; color: var(--text); }
.stat-label { font-size: 12px; color: var(--text-muted); margin-top: 2px; }

/* ===== 标签页 ===== */
.audit-tabs {
  display: flex;
  gap: 8px;
  margin-bottom: 20px;
  flex-wrap: wrap;
}
.audit-tab {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 10px 18px;
  border: 1px solid var(--border);
  border-radius: 12px;
  background: var(--card-solid);
  font-size: 14px;
  font-weight: 600;
  color: var(--text-muted);
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}
.audit-tab:hover { border-color: var(--accent); color: var(--accent); }
.audit-tab.active {
  background: var(--accent);
  border-color: var(--accent);
  color: #fff;
  box-shadow: 0 4px 14px rgba(0, 0, 0, 0.12);
}
.tab-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #d1d5db;
  transition: background 0.2s;
}
.tab-dot.pending { background: #f59e0b; }
.tab-dot.approved { background: #16a34a; }
.tab-dot.rejected { background: #dc2626; }
.audit-tab.active .tab-dot { background: rgba(255, 255, 255, 0.9); }
.tab-count {
  font-size: 11px;
  font-weight: 700;
  padding: 1px 8px;
  border-radius: 10px;
  background: #f3f4f6;
  color: var(--text-muted);
}
.tab-count.pending { background: rgba(245, 158, 11, 0.14); color: #f59e0b; }
.tab-count.approved { background: #f0fdf4; color: #16a34a; }
.tab-count.rejected { background: rgba(236, 65, 65, 0.12); color: #dc2626; }
.audit-tab.active .tab-count { background: rgba(255, 255, 255, 0.22); color: #fff; }

/* ===== 审核记录列表 ===== */
.audit-list {
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.audit-card {
  background: var(--card-solid);
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 18px 20px;
  transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
  animation: cardIn 0.5s cubic-bezier(0.16, 1, 0.3, 1) both;
}
.audit-card:hover {
  transform: translateY(-3px);
  box-shadow: 0 10px 28px rgba(0, 0, 0, 0.07);
  border-color: #e0e0e0;
}
@keyframes cardIn {
  from { opacity: 0; transform: translateY(14px) scale(0.98); }
  to { opacity: 1; transform: translateY(0) scale(1); }
}

.audit-card-top {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 14px;
}
.audit-type-badge {
  flex-shrink: 0;
  font-size: 11px;
  font-weight: 800;
  padding: 3px 10px;
  border-radius: 8px;
}
.audit-type-badge.t-avatar { background: #eff6ff; color: #2563eb; }
.audit-type-badge.t-nickname { background: #f5f3ff; color: #7c3aed; }
.audit-user {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 1px;
}
.audit-user strong { font-size: 14px; font-weight: 700; color: var(--text); }
.audit-user span { font-size: 12px; color: var(--text-muted); }
.audit-time {
  font-size: 11px;
  color: #bbb;
  white-space: nowrap;
  flex-shrink: 0;
}

/* 卡片主体 */
.audit-body { margin-bottom: 14px; }

/* 头像对比 */
.avatar-compare {
  display: flex;
  align-items: center;
  gap: 14px;
}
.avatar-new, .avatar-old {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
}
.avatar-img {
  width: 72px; height: 72px;
  border-radius: 50%;
  object-fit: cover;
  border: 3px solid #e5e5e5;
  transition: border-color 0.3s;
}
.avatar-img-old { width: 52px; height: 52px; opacity: 0.7; border-width: 2px; }
.audit-card:hover .avatar-img { border-color: var(--accent); }

.avatar-fallback {
  width: 72px; height: 72px;
  border-radius: 50%;
  background: #e8e8e8;
  display: flex; align-items: center; justify-content: center;
  color: #bbb;
  border: 3px solid #e5e5e5;
}
.avatar-fallback-old { width: 52px; height: 52px; border-width: 2px; }

.compare-vs {
  font-size: 11px;
  font-weight: 800;
  color: #ccc;
  letter-spacing: 0.05em;
}
.compare-label {
  font-size: 10px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 8px;
}
.label-new { background: #eff6ff; color: #2563eb; }
.label-old { background: #f5f5f5; color: #999; }

/* 改名对比 */
.nick-names {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}
.nick-old { font-size: 14px; color: var(--text-muted); text-decoration: line-through; }
.nick-arrow { color: #ccc; flex-shrink: 0; }
.nick-new { font-size: 15px; font-weight: 700; color: #16a34a; }

/* 卡片底部 */
.audit-card-foot {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}
.act-btn {
  flex: 1;
  max-width: 160px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
  padding: 10px 0;
  border-radius: 10px;
  border: none;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}
.act-btn:active { transform: scale(0.95); }
.act-approve { background: #16a34a; color: #fff; }
.act-approve:hover { background: #15803d; box-shadow: 0 4px 12px rgba(22, 163, 74, 0.3); }
.act-reject { background: var(--card-solid); color: #dc2626; border: 1px solid #fecaca; }
.act-reject:hover { background: rgba(236, 65, 65, 0.12); border-color: rgba(236, 65, 65, 0.25); }

.history-status {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 14px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 700;
}
.history-status .status-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
}
.history-status.approved { background: #f0fdf4; color: #16a34a; }
.history-status.approved .status-dot { background: #16a34a; }
.history-status.rejected { background: rgba(236, 65, 65, 0.12); color: #dc2626; }
.history-status.rejected .status-dot { background: #dc2626; }
.history-meta {
  font-size: 12px;
  color: var(--text-muted);
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

/* ===== 过渡动画 ===== */
.fade-down-enter-active, .fade-down-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-down-enter-from { opacity: 0; transform: translateY(-12px); }
.fade-down-leave-to { opacity: 0; transform: translateY(-8px); }

.fade-up-enter-active, .fade-up-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-up-enter-from { opacity: 0; transform: translateY(12px); }
.fade-up-leave-to { opacity: 0; transform: translateY(8px); }

.audit-card-enter-active { transition: all 0.5s cubic-bezier(0.16, 1, 0.3, 1); }
.audit-card-enter-from { opacity: 0; transform: translateY(14px) scale(0.98); }
.audit-card-leave-active { transition: all 0.3s ease; }
.audit-card-leave-to { opacity: 0; transform: scale(0.92); }
.audit-card-move { transition: transform 0.4s cubic-bezier(0.16, 1, 0.3, 1); }

/* ===== 响应式 ===== */
@media (max-width: 768px) {
  .stats-row { grid-template-columns: 1fr 1fr 1fr; gap: 10px; }
  .stat-chip { padding: 14px; flex-direction: column; align-items: flex-start; gap: 8px; }
  .stat-num { font-size: 22px; }
  .audit-card { padding: 14px; }
  .audit-card-top { flex-wrap: wrap; }
  .audit-time { width: 100%; }
  .act-btn { max-width: none; }
}
</style>
