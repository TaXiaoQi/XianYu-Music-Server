<template>
  <div class="aa-page">
    <!-- 页面头部 -->
    <Transition name="fade-down" appear>
      <div class="page-header">
        <div class="header-info">
          <h2 class="page-title">
            头像审核
            <span v-if="avatarStats.pending > 0" class="pending-badge">{{ avatarStats.pending }} 项待审核</span>
          </h2>
          <p class="page-desc">
            审核用户上传的头像和改名申请。通过后头像将更新为用户的新头像，改名申请通过后用户名将立即生效。
          </p>
        </div>
        <button class="btn-refresh" @click="loadAll" :disabled="loading">
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
        <div class="stat-chip stat-pending" :class="{ pulse: avatarStats.pending > 0 }">
          <div class="stat-icon">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
          </div>
          <div class="stat-body">
            <span class="stat-num">{{ avatarStats.pending }}</span>
            <span class="stat-label">待审核</span>
          </div>
        </div>
        <div class="stat-chip stat-approved">
          <div class="stat-icon">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
          </div>
          <div class="stat-body">
            <span class="stat-num">{{ avatarStats.approved }}</span>
            <span class="stat-label">已通过</span>
          </div>
        </div>
        <div class="stat-chip stat-rejected">
          <div class="stat-icon">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>
          </div>
          <div class="stat-body">
            <span class="stat-num">{{ avatarStats.rejected }}</span>
            <span class="stat-label">已拒绝</span>
          </div>
        </div>
        <div class="stat-chip stat-nickname">
          <div class="stat-icon">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M22 11h-6"/><path d="M19 8v6"/></svg>
          </div>
          <div class="stat-body">
            <span class="stat-num">{{ nicknameList.length }}</span>
            <span class="stat-label">待审核改名</span>
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
      <!-- ===== 改名审核 ===== -->
      <Transition name="fade-up" appear>
        <section class="section-block" v-show="nicknameList.length > 0 || nickLoaded">
          <div class="section-head">
            <h3 class="section-title">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M22 11h-6"/><path d="M19 8v6"/>
              </svg>
              改名审核
              <span v-if="nicknameList.length > 0" class="section-count">{{ nicknameList.length }}</span>
            </h3>
          </div>

          <TransitionGroup v-if="nicknameList.length > 0" name="nick-row" tag="div" class="nick-table-wrap">
            <div v-for="(item, idx) in nicknameList" :key="item.id" class="nick-row" :style="{ animationDelay: `${idx * 60}ms` }">
              <div class="nick-user">
                <div class="nick-avatar-placeholder">
                  <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
                </div>
                <div class="nick-user-info">
                  <span class="nick-id">{{ item.ciyuanxi_id || '未知' }}</span>
                  <span class="nick-time">{{ item.created_at || '-' }}</span>
                </div>
              </div>

              <div class="nick-names">
                <span class="nick-old">{{ item.old_name || '未知' }}</span>
                <svg class="nick-arrow" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="5" y1="12" x2="19" y2="12"/><polyline points="12 5 19 12 12 19"/></svg>
                <span class="nick-new">{{ item.new_name }}</span>
              </div>

              <div class="nick-actions">
                <button class="act-btn act-approve" @click="handleApproveNickname(item.id)">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12"/></svg>
                  通过
                </button>
                <button class="act-btn act-reject" @click="handleRejectNickname(item.id)">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
                  拒绝
                </button>
              </div>
            </div>
          </TransitionGroup>

          <div v-else class="empty-inline">
            <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
            <span>暂无待审核改名申请</span>
          </div>
        </section>
      </Transition>

      <!-- ===== 头像审核 ===== -->
      <section class="section-block">
        <div class="section-head">
          <h3 class="section-title">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M23 7l-7 5 7 5V7z"/><rect x="1" y="5" width="15" height="14" rx="2" ry="2"/>
            </svg>
            头像审核
            <span v-if="avatarList.length > 0" class="section-count">{{ avatarList.length }}</span>
          </h3>
        </div>

        <!-- 空状态 -->
        <Transition name="fade-up" appear v-if="avatarList.length === 0">
          <div class="state-box state-empty">
            <div class="empty-icon">
              <svg width="56" height="56" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/>
              </svg>
            </div>
            <p class="empty-title">暂无待审核头像</p>
            <p class="empty-sub">用户在桌面端上传新头像后将出现在此处</p>
          </div>
        </Transition>

        <!-- 头像卡片网格 -->
        <div v-else class="avatar-grid">
          <TransitionGroup name="avatar-card">
            <div
              v-for="(item, idx) in avatarList"
              :key="item.id"
              class="avatar-card"
              :style="{ animationDelay: `${idx * 80}ms` }"
            >
              <!-- 头像对比区 -->
              <div class="avatar-compare">
                <!-- 新头像（待审核） -->
                <div class="avatar-new">
                  <img v-if="item.avatar_data" :src="item.avatar_data" :alt="item.username" class="avatar-img" @error="onImgError" />
                  <div v-else class="avatar-fallback">
                    <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
                  </div>
                  <span class="compare-label label-new">新头像</span>
                </div>

                <!-- VS 分隔 -->
                <div class="compare-vs">VS</div>

                <!-- 当前头像 -->
                <div class="avatar-old">
                  <img v-if="item.current_avatar" :src="item.current_avatar" :alt="'当前头像'" class="avatar-img avatar-img-old" @error="onImgError" />
                  <div v-else class="avatar-fallback avatar-fallback-old">
                    <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
                  </div>
                  <span class="compare-label label-old">当前</span>
                </div>
              </div>

              <!-- 用户信息 -->
              <div class="avatar-info">
                <h4 class="avatar-username">{{ item.username || '未知用户' }}</h4>
                <p class="avatar-id">弦予号：{{ item.ciyuanxi_id || '-' }}</p>
                <p class="avatar-time">
                  <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
                  {{ item.created_at || '-' }}
                </p>
              </div>

              <!-- 操作按钮 -->
              <div class="avatar-actions">
                <button class="act-btn act-approve" @click="handleApproveAvatar(item.id)">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12"/></svg>
                  通过
                </button>
                <button class="act-btn act-reject" @click="handleRejectAvatar(item.id)">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
                  拒绝
                </button>
              </div>
            </div>
          </TransitionGroup>
        </div>
      </section>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { adminApi, showToast } from '@/api/client'

interface AvatarPending {
  id: number
  ciyuanxi_id: string
  avatar_data: string
  status: string
  created_at: string
  username: string
  current_avatar: string
  [key: string]: any
}

interface NicknamePending {
  id: number
  ciyuanxi_id: string
  new_name: string
  created_at: string
  old_name: string
  [key: string]: any
}

// ===== 状态 =====
const loading = ref(true)
const nickLoaded = ref(false)
const avatarList = ref<AvatarPending[]>([])
const nicknameList = ref<NicknamePending[]>([])
const avatarStats = reactive({ pending: 0, approved: 0, rejected: 0 })

// ===== 加载数据 =====
async function loadAvatars() {
  const res = await adminApi<{ list: AvatarPending[]; stats: { pending: number; approved: number; rejected: number } }>('list_avatar_pending')
  if (res.code === 200 && res.data) {
    avatarList.value = res.data.list || []
    const s = res.data.stats
    avatarStats.pending = s?.pending ?? 0
    avatarStats.approved = s?.approved ?? 0
    avatarStats.rejected = s?.rejected ?? 0
  } else {
    avatarList.value = []
  }
}

async function loadNicknames() {
  const res = await adminApi<NicknamePending[]>('list_nickname_pending')
  if (res.code === 200 && res.data) {
    nicknameList.value = Array.isArray(res.data) ? res.data : []
  } else {
    nicknameList.value = []
  }
  nickLoaded.value = true
}

async function loadAll() {
  loading.value = true
  await Promise.all([loadAvatars(), loadNicknames()])
  loading.value = false
}

// ===== 头像审核 =====
async function handleApproveAvatar(id: number) {
  if (!confirm('确认通过该头像审核？通过后将更新为用户的新头像。')) return
  const res = await adminApi('approve_avatar', { id })
  if (res.code === 200) {
    showToast('审核通过', 'success')
    avatarList.value = avatarList.value.filter(a => a.id !== id)
    avatarStats.pending = Math.max(0, avatarStats.pending - 1)
    avatarStats.approved += 1
  } else {
    showToast(res.msg || '操作失败')
  }
}

async function handleRejectAvatar(id: number) {
  if (!confirm('确认拒绝该头像审核？')) return
  const res = await adminApi('reject_avatar', { id })
  if (res.code === 200) {
    showToast('已拒绝', 'success')
    avatarList.value = avatarList.value.filter(a => a.id !== id)
    avatarStats.pending = Math.max(0, avatarStats.pending - 1)
    avatarStats.rejected += 1
  } else {
    showToast(res.msg || '操作失败')
  }
}

// ===== 改名审核 =====
async function handleApproveNickname(id: number) {
  if (!confirm('确认通过该改名申请？通过后用户名将更新。')) return
  const res = await adminApi('approve_nickname', { id })
  if (res.code === 200) {
    showToast('审核通过', 'success')
    nicknameList.value = nicknameList.value.filter(n => n.id !== id)
  } else {
    showToast(res.msg || '操作失败')
  }
}

async function handleRejectNickname(id: number) {
  if (!confirm('确认拒绝该改名申请？')) return
  const res = await adminApi('reject_nickname', { id })
  if (res.code === 200) {
    showToast('已拒绝', 'success')
    nicknameList.value = nicknameList.value.filter(n => n.id !== id)
  } else {
    showToast(res.msg || '操作失败')
  }
}

// ===== 工具 =====
function onImgError(e: Event) {
  const img = e.target as HTMLImageElement
  img.style.display = 'none'
}

onMounted(() => {
  loadAll()
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
  max-width: 620px;
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

/* ===== 统计卡片 ===== */
.stats-row {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 14px;
  margin-bottom: 24px;
}
.stat-chip {
  background: var(--white);
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 18px 20px;
  display: flex;
  align-items: center;
  gap: 14px;
  transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}
.stat-chip:hover { transform: translateY(-3px); box-shadow: 0 8px 24px rgba(0, 0, 0, 0.06); }
.stat-icon {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.stat-pending .stat-icon { background: #fffbeb; color: #f59e0b; }
.stat-approved .stat-icon { background: #f0fdf4; color: #16a34a; }
.stat-rejected .stat-icon { background: #fef2f2; color: #dc2626; }
.stat-nickname .stat-icon { background: #eff6ff; color: #2563eb; }

.stat-pending.pulse { border-color: #fbbf24; }
.stat-pending.pulse .stat-icon { animation: iconPulse 2s ease-in-out infinite; }
@keyframes iconPulse {
  0%, 100% { transform: scale(1); }
  50% { transform: scale(1.08); }
}

.stat-body { display: flex; flex-direction: column; }
.stat-num { font-size: 26px; font-weight: 800; line-height: 1.1; color: var(--text); }
.stat-label { font-size: 12px; color: var(--text-muted); margin-top: 2px; }

/* ===== 区块 ===== */
.section-block {
  background: var(--white);
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 24px;
  margin-bottom: 20px;
}
.section-head { margin-bottom: 18px; }
.section-title {
  font-size: 16px;
  font-weight: 700;
  color: var(--text);
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0;
}
.section-title svg { color: var(--text-muted); }
.section-count {
  font-size: 12px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 10px;
  background: #fffbeb;
  color: #f59e0b;
}

/* ===== 改名审核行 ===== */
.nick-table-wrap { display: flex; flex-direction: column; gap: 8px; }
.nick-row {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 14px 16px;
  background: #fafafa;
  border: 1px solid #f0f0f0;
  border-radius: 10px;
  transition: all 0.25s ease;
  animation: slideIn 0.4s cubic-bezier(0.16, 1, 0.3, 1) both;
}
.nick-row:hover { background: #f5f5f5; border-color: #e0e0e0; }
@keyframes slideIn {
  from { opacity: 0; transform: translateX(-12px); }
  to { opacity: 1; transform: translateX(0); }
}

.nick-user { display: flex; align-items: center; gap: 10px; min-width: 180px; }
.nick-avatar-placeholder {
  width: 36px; height: 36px;
  border-radius: 50%;
  background: #e8e8e8;
  display: flex; align-items: center; justify-content: center;
  color: #999;
  flex-shrink: 0;
}
.nick-user-info { display: flex; flex-direction: column; gap: 2px; }
.nick-id { font-size: 13px; font-weight: 600; color: var(--text); }
.nick-time { font-size: 11px; color: var(--text-muted); }

.nick-names { display: flex; align-items: center; gap: 12px; flex: 1; }
.nick-old { font-size: 14px; color: var(--text-muted); text-decoration: line-through; }
.nick-arrow { color: #ccc; flex-shrink: 0; }
.nick-new { font-size: 15px; font-weight: 700; color: #16a34a; }

.nick-actions { display: flex; gap: 8px; flex-shrink: 0; }

/* ===== 头像卡片网格 ===== */
.avatar-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 16px;
}
.avatar-card {
  background: #fafafa;
  border: 1px solid #f0f0f0;
  border-radius: 14px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  align-items: center;
  transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
  animation: cardIn 0.5s cubic-bezier(0.16, 1, 0.3, 1) both;
}
.avatar-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.08);
  border-color: #e0e0e0;
  background: var(--white);
}
@keyframes cardIn {
  from { opacity: 0; transform: translateY(16px) scale(0.96); }
  to { opacity: 1; transform: translateY(0) scale(1); }
}

/* 头像对比 */
.avatar-compare {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
}
.avatar-new, .avatar-old {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
}
.avatar-img {
  width: 80px; height: 80px;
  border-radius: 50%;
  object-fit: cover;
  border: 3px solid #e5e5e5;
  transition: border-color 0.3s;
}
.avatar-img-old { width: 56px; height: 56px; opacity: 0.7; border-width: 2px; }
.avatar-card:hover .avatar-img { border-color: var(--accent); }

.avatar-fallback {
  width: 80px; height: 80px;
  border-radius: 50%;
  background: #e8e8e8;
  display: flex; align-items: center; justify-content: center;
  color: #bbb;
  border: 3px solid #e5e5e5;
}
.avatar-fallback-old { width: 56px; height: 56px; border-width: 2px; }

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

/* 用户信息 */
.avatar-info { text-align: center; margin-bottom: 16px; }
.avatar-username { font-size: 15px; font-weight: 700; color: var(--text); margin: 0 0 4px 0; }
.avatar-id { font-size: 12px; color: var(--text-muted); margin: 0 0 2px 0; }
.avatar-time {
  font-size: 11px;
  color: #bbb;
  margin: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
}

/* 操作按钮 */
.avatar-actions { display: flex; gap: 10px; width: 100%; }
.act-btn {
  flex: 1;
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
.act-reject { background: var(--white); color: #dc2626; border: 1px solid #fecaca; }
.act-reject:hover { background: #fef2f2; border-color: #fca5a5; }

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
.state-error { color: #dc2626; }
.state-empty { padding: 48px 20px; }
.empty-icon { color: #d0d0d0; margin-bottom: 4px; }
.empty-title { font-size: 15px; font-weight: 600; color: var(--text-light); margin: 0; }
.empty-sub { font-size: 13px; color: var(--text-muted); margin: 0; }

.empty-inline {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 24px;
  color: var(--text-muted);
  font-size: 13px;
  background: #fafafa;
  border-radius: 10px;
  border: 1px dashed #e5e5e5;
}

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

.avatar-card-enter-active { transition: all 0.5s cubic-bezier(0.16, 1, 0.3, 1); }
.avatar-card-enter-from { opacity: 0; transform: translateY(16px) scale(0.96); }
.avatar-card-leave-active { transition: all 0.3s ease; }
.avatar-card-leave-to { opacity: 0; transform: scale(0.9); }
.avatar-card-move { transition: transform 0.4s cubic-bezier(0.16, 1, 0.3, 1); }

.nick-row-enter-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.nick-row-enter-from { opacity: 0; transform: translateX(-12px); }
.nick-row-leave-active { transition: all 0.3s ease; }
.nick-row-leave-to { opacity: 0; transform: translateX(20px); }
.nick-row-move { transition: transform 0.4s cubic-bezier(0.16, 1, 0.3, 1); }

/* ===== 响应式 ===== */
@media (max-width: 768px) {
  .stats-row { grid-template-columns: 1fr 1fr; gap: 10px; }
  .stat-chip { padding: 14px; }
  .stat-num { font-size: 22px; }
  .nick-row { flex-direction: column; align-items: stretch; gap: 10px; }
  .nick-names { justify-content: center; }
  .nick-actions { justify-content: center; }
  .avatar-grid { grid-template-columns: 1fr; }
  .section-block { padding: 16px; }
}
</style>
