<template>
  <div class="fb-page">
    <!-- 页面头部 -->
    <Transition name="fade-down" appear>
      <div class="page-header">
        <div class="header-info">
          <h2 class="page-title">
            反馈与建议
            <span v-if="stats.pending > 0" class="pending-badge">{{ stats.pending }} 项待处理</span>
          </h2>
          <p class="page-desc">查看用户提交的反馈与建议，将问题标记为已解决或已拒绝。</p>
        </div>
        <button class="btn-refresh" @click="loadList" :disabled="loading">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" :class="{ spinning: loading }">
            <polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
          </svg>
          刷新
        </button>
      </div>
    </Transition>

    <!-- 提交限制配置 -->
    <Transition name="fade-up" appear>
      <div class="limit-panel">
        <div class="limit-info">
          <div class="limit-icon">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
              <path d="M12 8v4"/>
              <path d="M12 16h.01"/>
            </svg>
          </div>
          <div>
            <h3>每日反馈提交上限</h3>
            <p>当前每个用户每天最多可提交 {{ feedbackDailyLimit === 0 ? '不限' : `${feedbackDailyLimit} 条` }}反馈，修改后立即生效。</p>
          </div>
        </div>
        <div class="limit-actions">
          <input
            v-model.number="feedbackLimitInput"
            class="limit-input"
            type="number"
            min="0"
            max="10000"
            step="1"
            :disabled="limitLoading || limitSaving"
            @keyup.enter="saveFeedbackLimit"
          />
          <button class="btn-save-limit" :disabled="limitLoading || limitSaving" @click="saveFeedbackLimit">
            <span v-if="limitSaving" class="btn-spinner dark"></span>
            {{ limitSaving ? '保存中...' : '保存上限' }}
          </button>
        </div>
      </div>
    </Transition>

    <!-- 统计卡片 -->
    <Transition name="fade-up" appear>
      <div class="stats-row">
        <div class="stat-chip" :class="{ active: activeFilter === 'all' }" @click="activeFilter = 'all'">
          <div class="stat-icon stat-icon-total"><svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg></div>
          <div class="stat-body"><span class="stat-num">{{ stats.total }}</span><span class="stat-label">全部</span></div>
        </div>
        <div class="stat-chip" :class="{ active: activeFilter === 'pending' }" @click="activeFilter = 'pending'">
          <div class="stat-icon stat-icon-pending"><svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg></div>
          <div class="stat-body"><span class="stat-num">{{ stats.pending }}</span><span class="stat-label">待处理</span></div>
        </div>
        <div class="stat-chip" :class="{ active: activeFilter === 'processing' }" @click="activeFilter = 'processing'">
          <div class="stat-icon stat-icon-processing"><svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2v4"/><path d="M12 18v4"/><path d="M4.93 4.93l2.83 2.83"/><path d="M16.24 16.24l2.83 2.83"/><circle cx="12" cy="12" r="4"/></svg></div>
          <div class="stat-body"><span class="stat-num">{{ stats.processing }}</span><span class="stat-label">处理中</span></div>
        </div>
        <div class="stat-chip" :class="{ active: activeFilter === 'resolved' }" @click="activeFilter = 'resolved'">
          <div class="stat-icon stat-icon-resolved"><svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg></div>
          <div class="stat-body"><span class="stat-num">{{ stats.resolved }}</span><span class="stat-label">已解决</span></div>
        </div>
        <div class="stat-chip" :class="{ active: activeFilter === 'rejected' }" @click="activeFilter = 'rejected'">
          <div class="stat-icon stat-icon-rejected"><svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg></div>
          <div class="stat-body"><span class="stat-num">{{ stats.rejected }}</span><span class="stat-label">已拒绝</span></div>
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
      <Transition name="fade-up" appear v-if="filteredList.length === 0">
        <div class="state-box state-empty">
          <div class="empty-icon">
            <svg width="56" height="56" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>
          </div>
          <p class="empty-title">{{ activeFilter === 'all' ? '暂无反馈记录' : '该状态下暂无反馈' }}</p>
          <p class="empty-sub">用户提交的反馈与建议将显示在这里</p>
        </div>
      </Transition>

      <!-- 反馈卡片列表 -->
      <div v-else class="fb-list">
        <TransitionGroup name="fb-card">
          <div
            v-for="(item, idx) in filteredList"
            :key="item.id"
            class="fb-card"
            :class="`st-${item.status}`"
            :style="{ animationDelay: `${idx * 60}ms` }"
          >
            <!-- 卡片头部 -->
            <div class="card-top">
              <div class="card-user">
                <div class="user-avatar">
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
                </div>
                <div class="user-info">
                  <span class="user-name">{{ item.nickname || '匿名用户' }}</span>
                  <span class="user-id">{{ item.ciyuanxi_id || '-' }}</span>
                </div>
              </div>
              <span class="status-badge" :class="`badge-${item.status}`">
                <svg v-if="item.status === 'resolved'" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><polyline points="20 6 9 17 4 12"/></svg>
                <svg v-else-if="item.status === 'rejected'" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
                <svg v-else-if="item.status === 'processing'" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
                {{ statusLabel(item.status) }}
              </span>
            </div>

            <!-- 标题和内容 -->
            <div class="card-body">
              <h3 class="fb-title">{{ item.title || '无标题' }}</h3>
              <p class="fb-content">{{ item.content || '无内容' }}</p>
              <div v-if="hasErrorLogs(item) || hasAllLogs(item)" class="log-summary">
                <span v-if="hasErrorLogs(item)" class="log-chip">错误日志 {{ formatLogSize(item.error_logs_chars) }}</span>
                <span v-if="hasAllLogs(item)" class="log-chip">全量日志 {{ formatLogSize(item.all_logs_chars) }}</span>
              </div>
            </div>

            <!-- 卡片底部 -->
            <div class="card-foot">
              <div class="foot-meta">
                <span class="meta-ip">
                  <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="2" width="20" height="8" rx="2"/><rect x="2" y="14" width="20" height="8" rx="2"/><line x1="6" y1="6" x2="6.01" y2="6"/><line x1="6" y1="18" x2="6.01" y2="18"/></svg>
                  {{ item.ip || '未知' }}
                </span>
                <span class="meta-time">
                  <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
                  {{ item.created_at || '-' }}
                </span>
              </div>
              <div class="foot-actions">
                <button v-if="hasErrorLogs(item) || hasAllLogs(item)" class="act-btn act-log" @click="openLogModal(item)">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="8" y1="13" x2="16" y2="13"/><line x1="8" y1="17" x2="16" y2="17"/></svg>
                  日志
                </button>
                <button v-if="isEditable(item)" class="act-btn act-resolve" @click="changeStatus(item.id, 'resolved')">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12"/></svg>
                  完成
                </button>
                <button v-if="isEditable(item)" class="act-btn act-reject" @click="changeStatus(item.id, 'rejected')">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
                  拒绝
                </button>
              </div>
            </div>
          </div>
        </TransitionGroup>
      </div>
    </template>

    <!-- 日志弹窗 -->
    <Transition name="modal">
      <div v-if="logModalVisible" class="modal-backdrop" @click.self="closeLogModal">
        <div class="modal-dialog log-dialog">
          <div class="modal-head">
            <h3>反馈日志</h3>
            <button class="modal-close" @click="closeLogModal">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
          <div class="modal-body">
            <div v-if="logTarget" class="log-target-info">
              <strong>{{ logTarget.title || '无标题' }}</strong>
              <span>{{ logTarget.nickname || '匿名用户' }} · {{ logTarget.ciyuanxi_id || '-' }}</span>
            </div>
            <div class="log-tabs">
              <button
                class="log-tab"
                :class="{ active: activeLogTab === 'error' }"
                :disabled="!logTarget?.error_logs"
                @click="activeLogTab = 'error'"
              >
                错误日志
              </button>
              <button
                class="log-tab"
                :class="{ active: activeLogTab === 'all' }"
                :disabled="!logTarget?.all_logs"
                @click="activeLogTab = 'all'"
              >
                全量日志
              </button>
            </div>
            <div v-if="logLoading" class="state-box compact">
              <div class="spinner"></div>
              <span>正在加载日志...</span>
            </div>
            <pre v-else class="log-content">{{ currentLogText || '暂无日志内容' }}</pre>
          </div>
          <div class="modal-foot">
            <button class="btn-cancel" @click="closeLogModal">关闭</button>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { adminApi, showToast } from '@/api/client'
import { webConfirm } from '@/utils/webDialog'

interface Feedback {
  id: number
  ciyuanxi_id: string
  nickname: string
  title: string
  content: string
  status: string
  admin_reply: string | null
  error_logs?: string | null
  all_logs?: string | null
  log_meta?: string | null
  error_logs_chars?: number
  all_logs_chars?: number
  has_error_logs?: number | string | boolean
  has_all_logs?: number | string | boolean
  replied_at: string | null
  replied_by: string
  ip: string
  created_at: string
  updated_at: string
  [key: string]: any
}

interface FbStats {
  total: number
  pending: number
  processing: number
  resolved: number
  rejected: number
}

interface FeedbackLimit {
  feedback_daily_limit: number
}

// ===== 状态 =====
const loading = ref(true)
const feedbackList = ref<Feedback[]>([])
const activeFilter = ref('all')
const stats = ref<FbStats>({ total: 0, pending: 0, processing: 0, resolved: 0, rejected: 0 })
const limitLoading = ref(false)
const limitSaving = ref(false)
const feedbackDailyLimit = ref(20)
const feedbackLimitInput = ref(20)

const statusMap: Record<string, string> = {
  pending: '待处理',
  processing: '处理中',
  resolved: '已解决',
  rejected: '已拒绝',
}

function statusLabel(s: string): string {
  return statusMap[s] || s
}

// 仅待处理/处理中可执行完成或拒绝操作，终态（已解决/已拒绝）不再显示操作按钮
function isEditable(item: Feedback): boolean {
  return item.status === 'pending' || item.status === 'processing'
}

function formatLogSize(chars?: number | string): string {
  const n = Number(chars || 0)
  if (n <= 0) return ''
  if (n < 1024) return `${n} 字`
  return `${(n / 1024).toFixed(1)}K 字`
}

function truthyFlag(value: unknown): boolean {
  return value === true || value === 1 || value === '1'
}

function hasErrorLogs(item: Feedback): boolean {
  return truthyFlag(item.has_error_logs) || !!item.error_logs
}

function hasAllLogs(item: Feedback): boolean {
  return truthyFlag(item.has_all_logs) || !!item.all_logs
}

const filteredList = computed(() => {
  if (activeFilter.value === 'all') return feedbackList.value
  return feedbackList.value.filter(f => f.status === activeFilter.value)
})

// ===== 加载数据 =====
async function loadList() {
  loading.value = true
  const res = await adminApi<{ list: Feedback[]; stats: FbStats }>('list_feedback', {
    status_filter: activeFilter.value === 'all' ? '' : activeFilter.value,
  })
  if (res.code === 200 && res.data) {
    feedbackList.value = res.data.list || []
    if (res.data.stats) {
      stats.value = res.data.stats
    }
  } else {
    feedbackList.value = []
  }
  loading.value = false
}

async function loadFeedbackLimit() {
  limitLoading.value = true
  const res = await adminApi<FeedbackLimit>('get_feedback_limit')
  if (res.code === 200 && res.data) {
    const limit = Number(res.data.feedback_daily_limit ?? 20)
    feedbackDailyLimit.value = Number.isFinite(limit) ? limit : 20
    feedbackLimitInput.value = feedbackDailyLimit.value
  } else {
    showToast(res.msg || '反馈上限加载失败')
  }
  limitLoading.value = false
}

async function saveFeedbackLimit() {
  const limit = Number(feedbackLimitInput.value)
  if (!Number.isInteger(limit) || limit < 0 || limit > 10000) {
    showToast('每日上限需为 0 到 10000 的整数')
    return
  }
  limitSaving.value = true
  const res = await adminApi<FeedbackLimit>('update_feedback_limit', {
    feedback_daily_limit: limit,
  })
  limitSaving.value = false
  if (res.code === 200) {
    feedbackDailyLimit.value = Number(res.data?.feedback_daily_limit ?? limit)
    feedbackLimitInput.value = feedbackDailyLimit.value
    showToast('反馈提交上限已保存', 'success')
  } else {
    showToast(res.msg || '保存失败')
  }
}

// ===== 状态变更 =====
async function changeStatus(id: number, status: string) {
  const tips: Record<string, string> = {
    resolved: '确认将此反馈标记为已解决？',
    rejected: '确认拒绝此反馈？',
  }
  if (tips[status]) {
    const ok = await webConfirm(tips[status], { title: '更新反馈状态', confirmText: '确认' })
    if (!ok) return
  }
  const res = await adminApi('update_feedback_status', { id, status })
  if (res.code === 200) {
    showToast('状态已更新', 'success')
    // 本地更新
    const item = feedbackList.value.find(f => f.id === id)
    if (item) {
      const oldStatus = item.status
      item.status = status
      // 更新统计
      if (stats.value[oldStatus as keyof FbStats] !== undefined) {
        stats.value[oldStatus as keyof FbStats]--
      }
      if (stats.value[status as keyof FbStats] !== undefined) {
        stats.value[status as keyof FbStats]++
      }
    }
  } else {
    showToast(res.msg || '操作失败')
  }
}

// ===== 日志弹窗 =====
const logModalVisible = ref(false)
const logTarget = ref<Feedback | null>(null)
const logLoading = ref(false)
const activeLogTab = ref<'error' | 'all'>('error')

const currentLogText = computed(() => {
  if (!logTarget.value) return ''
  return activeLogTab.value === 'error'
    ? (logTarget.value.error_logs || '')
    : (logTarget.value.all_logs || '')
})

async function openLogModal(item: Feedback) {
  logModalVisible.value = true
  logTarget.value = item
  activeLogTab.value = hasErrorLogs(item) ? 'error' : 'all'
  logLoading.value = true
  const res = await adminApi<Feedback>('get_feedback_detail', { id: item.id })
  logLoading.value = false
  if (res.code === 200 && res.data) {
    logTarget.value = res.data
    activeLogTab.value = res.data.error_logs ? 'error' : 'all'
  } else {
    showToast(res.msg || '日志加载失败')
  }
}

function closeLogModal() {
  if (logLoading.value) return
  logModalVisible.value = false
  logTarget.value = null
}

onMounted(() => {
  loadFeedbackLimit()
  loadList()
})
</script>

<style scoped>
.fb-page {
  max-width: 920px;
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
  background: #fffbeb;
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
  max-width: 560px;
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

/* ===== 提交限制配置 ===== */
.limit-panel {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
  background: linear-gradient(135deg, #ffffff 0%, #f8fafc 100%);
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 16px 18px;
  margin-bottom: 20px;
  box-shadow: 0 6px 20px rgba(15, 23, 42, 0.04);
}
.limit-info {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}
.limit-icon {
  width: 42px;
  height: 42px;
  border-radius: 12px;
  background: #eef2ff;
  color: #4f46e5;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.limit-info h3 {
  font-size: 15px;
  font-weight: 800;
  color: var(--text);
  margin: 0 0 4px 0;
}
.limit-info p {
  font-size: 12px;
  color: var(--text-muted);
  margin: 0;
  line-height: 1.5;
}
.limit-actions {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-shrink: 0;
}
.limit-input {
  width: 110px;
  height: 38px;
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 0 12px;
  font-size: 14px;
  font-weight: 700;
  color: var(--text);
  outline: none;
  transition: border-color 0.2s, box-shadow 0.2s;
}
.limit-input:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 3px rgba(26, 26, 26, 0.08);
}
.limit-input:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
.btn-save-limit {
  height: 38px;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 0 16px;
  border-radius: 10px;
  border: none;
  background: var(--accent);
  color: #fff;
  font-size: 13px;
  font-weight: 700;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.2s;
}
.btn-save-limit:hover { opacity: 0.86; transform: translateY(-1px); }
.btn-save-limit:active { transform: scale(0.96); }
.btn-save-limit:disabled { opacity: 0.55; cursor: not-allowed; transform: none; }

/* ===== 统计卡片 ===== */
.stats-row {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
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
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}
.stat-chip:hover { transform: translateY(-2px); box-shadow: 0 6px 20px rgba(0, 0, 0, 0.06); }
.stat-chip.active { border-color: var(--accent); box-shadow: 0 0 0 2px rgba(26, 26, 26, 0.08); }
.stat-icon {
  width: 38px; height: 38px;
  border-radius: 10px;
  display: flex; align-items: center; justify-content: center;
  flex-shrink: 0;
}
.stat-icon-total { background: #f0f0f0; color: #1a1a1a; }
.stat-icon-pending { background: #fffbeb; color: #f59e0b; }
.stat-icon-processing { background: #eff6ff; color: #3b82f6; }
.stat-icon-resolved { background: #f0fdf4; color: #16a34a; }
.stat-icon-rejected { background: #fef2f2; color: #dc2626; }
.stat-body { display: flex; flex-direction: column; }
.stat-num { font-size: 22px; font-weight: 800; line-height: 1.1; color: var(--text); }
.stat-label { font-size: 11px; color: var(--text-muted); margin-top: 2px; }

/* ===== 反馈卡片 ===== */
.fb-list { display: flex; flex-direction: column; gap: 14px; }
.fb-card {
  background: var(--white);
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 18px 20px;
  transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
  animation: cardIn 0.5s cubic-bezier(0.16, 1, 0.3, 1) both;
  border-left: 3px solid transparent;
}
.fb-card:hover { box-shadow: 0 8px 24px rgba(0, 0, 0, 0.06); transform: translateY(-2px); }
.fb-card.st-pending { border-left-color: #f59e0b; }
.fb-card.st-processing { border-left-color: #3b82f6; }
.fb-card.st-resolved { border-left-color: #16a34a; }
.fb-card.st-rejected { border-left-color: #dc2626; }
@keyframes cardIn {
  from { opacity: 0; transform: translateY(16px); }
  to { opacity: 1; transform: translateY(0); }
}

.card-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}
.card-user { display: flex; align-items: center; gap: 10px; }
.user-avatar {
  width: 36px; height: 36px;
  border-radius: 50%;
  background: #f0f0f0;
  display: flex; align-items: center; justify-content: center;
  color: #999;
  flex-shrink: 0;
}
.user-info { display: flex; flex-direction: column; gap: 1px; }
.user-name { font-size: 14px; font-weight: 600; color: var(--text); }
.user-id { font-size: 11px; color: var(--text-muted); }

.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 12px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 600;
}
.badge-pending { background: #fffbeb; color: #f59e0b; }
.badge-processing { background: #eff6ff; color: #3b82f6; }
.badge-resolved { background: #f0fdf4; color: #16a34a; }
.badge-rejected { background: #fef2f2; color: #dc2626; }

.card-body { margin-bottom: 12px; }
.fb-title { font-size: 15px; font-weight: 700; color: var(--text); margin: 0 0 6px 0; }
.fb-content {
  font-size: 13px;
  color: var(--text-light);
  line-height: 1.6;
  margin: 0;
  white-space: pre-wrap;
  word-break: break-word;
}
.log-summary {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 10px;
}
.log-chip {
  display: inline-flex;
  align-items: center;
  padding: 3px 8px;
  border-radius: 999px;
  background: #f4f4f5;
  color: #52525b;
  font-size: 11px;
  font-weight: 600;
}

/* 卡片底部 */
.card-foot {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}
.foot-meta { display: flex; align-items: center; gap: 12px; }
.meta-ip, .meta-time {
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
.act-log { background: #f4f4f5; color: #52525b; }
.act-log:hover { background: #e4e4e7; }
.act-resolve { background: #f0fdf4; color: #16a34a; }
.act-resolve:hover { background: #dcfce7; }
.act-reject { background: #fef2f2; color: #dc2626; }
.act-reject:hover { background: #fee2e2; }

/* ===== 回复弹窗 ===== */
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
  max-width: 480px;
  overflow: hidden;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.15);
}
.log-dialog {
  max-width: min(920px, calc(100vw - 40px));
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
.log-target-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 12px 14px;
  background: #f8f9fc;
  border-radius: 10px;
  margin-bottom: 16px;
}
.log-target-info strong { font-size: 14px; color: var(--text); }
.log-target-info span { font-size: 12px; color: var(--text-muted); }
.field { margin-bottom: 0; }
.field label {
  display: block;
  font-size: 13px;
  font-weight: 600;
  color: var(--text);
  margin-bottom: 8px;
}
.field textarea {
  width: 100%;
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 12px;
  font-size: 13px;
  font-family: inherit;
  resize: vertical;
  min-height: 100px;
  outline: none;
  transition: border-color 0.2s;
  box-sizing: border-box;
}
.field textarea:focus { border-color: var(--accent); }
.log-tabs {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
}
.log-tab {
  border: 1px solid var(--border);
  background: var(--white);
  color: var(--text-light);
  border-radius: 8px;
  padding: 7px 12px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
}
.log-tab.active {
  background: var(--accent);
  color: #fff;
  border-color: var(--accent);
}
.log-tab:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}
.log-content {
  max-height: 520px;
  overflow: auto;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: #0f172a;
  color: #e5e7eb;
  padding: 14px;
  font-size: 12px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-word;
}
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
.btn-spinner.dark {
  border-color: rgba(255,255,255,0.35);
  border-top-color: #fff;
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
.state-box.compact {
  padding: 24px 20px;
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

.fade-up-enter-active, .fade-up-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-up-enter-from { opacity: 0; transform: translateY(12px); }

.fb-card-enter-active { transition: all 0.5s cubic-bezier(0.16, 1, 0.3, 1); }
.fb-card-enter-from { opacity: 0; transform: translateY(16px); }
.fb-card-leave-active { transition: all 0.3s ease; }
.fb-card-leave-to { opacity: 0; transform: scale(0.95); }
.fb-card-move { transition: transform 0.4s cubic-bezier(0.16, 1, 0.3, 1); }

.modal-enter-active, .modal-leave-active { transition: opacity 0.3s ease; }
.modal-enter-from, .modal-leave-to { opacity: 0; }
.modal-enter-active .modal-dialog { animation: modalIn 0.3s cubic-bezier(0.16, 1, 0.3, 1); }
@keyframes modalIn {
  from { opacity: 0; transform: scale(0.92) translateY(20px); }
  to { opacity: 1; transform: scale(1) translateY(0); }
}

/* ===== 响应式 ===== */
@media (max-width: 768px) {
  .stats-row { grid-template-columns: 1fr 1fr 1fr; gap: 8px; }
  .stat-chip { padding: 10px 12px; flex-direction: column; align-items: flex-start; gap: 6px; }
  .stat-num { font-size: 18px; }
  .stat-label { font-size: 10px; }
  .card-foot { flex-direction: column; align-items: stretch; }
  .foot-actions { justify-content: flex-end; }
  .fb-card { padding: 14px 16px; }
}
</style>
