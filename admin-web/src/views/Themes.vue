<template>
  <div class="th-page">
    <!-- 页面头部 -->
    <Transition name="fade-down" appear>
      <div class="page-header">
        <div class="header-info">
          <h2 class="page-title">
            主题中心
            <span v-if="pendingCount > 0" class="pending-badge">{{ pendingCount }} 项待审核</span>
          </h2>
          <p class="page-desc">
            主题包按客户端平台隔离下发：用户上传的主题继承其客户端平台，审核通过后仅对该平台展示，下架后不再对用户可见。
          </p>
        </div>
        <button class="btn-refresh" :disabled="loading" @click="loadList()">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/>
          </svg>
          刷新
        </button>
      </div>
    </Transition>

    <!-- 平台切换 -->
    <Transition name="fade-up" appear>
      <div class="platform-tabs">
        <button
          v-for="p in PLATFORMS"
          :key="p.key"
          class="platform-tab"
          :class="{ active: platformFilter === p.key }"
          @click="platformFilter = p.key"
        >{{ p.label }}</button>
      </div>
    </Transition>

    <!-- 状态筛选 -->
    <Transition name="fade-up" appear>
      <div class="filter-bar">
        <button
          v-for="f in filters"
          :key="f.value"
          class="filter-pill"
          :class="{ active: activeFilter === f.value }"
          @click="activeFilter = f.value"
        >
          {{ f.label }}
          <span v-if="countByStatus(f.value) > 0" class="pill-count">{{ countByStatus(f.value) }}</span>
        </button>
      </div>
    </Transition>

    <!-- 加载中 -->
    <div v-if="loading" class="state-box">
      <div class="spinner"></div>
      <span>加载中...</span>
    </div>

    <!-- 空状态 -->
    <Transition name="fade-up" appear v-else-if="filteredList.length === 0">
      <div class="state-box state-empty">
        <div class="empty-icon">
          <svg width="56" height="56" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="13.5" cy="6.5" r=".5"/><circle cx="17.5" cy="10.5" r=".5"/><circle cx="8.5" cy="7.5" r=".5"/><circle cx="6.5" cy="12.5" r=".5"/><path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c.926 0 1.648-.746 1.648-1.688 0-.437-.18-.835-.437-1.125-.29-.289-.438-.652-.438-1.125a1.64 1.64 0 0 1 1.668-1.668h1.996c3.051 0 5.555-2.503 5.555-5.554C21.965 6.012 17.461 2 12 2z"/>
          </svg>
        </div>
        <p class="empty-title">{{ activeFilter === 'all' ? `暂无${currentPlatformLabel}主题` : '该分类下暂无主题' }}</p>
        <p class="empty-sub">用户上传的主题包会出现在这里等待审核</p>
      </div>
    </Transition>

    <!-- 主题画廊 -->
    <div v-else class="gallery-grid">
      <TransitionGroup name="card">
        <div
          v-for="(item, idx) in filteredList"
          :key="item.id"
          class="th-card"
          :class="[`st-${item.status}`, { 'is-user': isUserUpload(item) }]"
          :style="{ animationDelay: `${idx * 50}ms` }"
        >
          <!-- 缩略图 -->
          <div class="thumb-wrap">
            <img
              v-if="item.thumbnail_url || item.preview_url"
              :src="item.thumbnail_url || item.preview_url"
              :alt="item.name"
              class="thumb-img"
              loading="lazy"
              @error="onImgError"
            />
            <div v-else class="thumb-placeholder">
              <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="13.5" cy="6.5" r=".5"/><circle cx="17.5" cy="10.5" r=".5"/><circle cx="8.5" cy="7.5" r=".5"/><circle cx="6.5" cy="12.5" r=".5"/><path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c.926 0 1.648-.746 1.648-1.688 0-.437-.18-.835-.437-1.125-.29-.289-.438-.652-.438-1.125a1.64 1.64 0 0 1 1.668-1.668h1.996c3.051 0 5.555-2.503 5.555-5.554C21.965 6.012 17.461 2 12 2z"/></svg>
            </div>

            <!-- hover 遮罩 -->
            <div class="thumb-overlay">
              <button v-if="item.preview_url" class="overlay-btn" title="查看大图" @click="openDetail(item)">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/><line x1="11" y1="8" x2="11" y2="14"/><line x1="8" y1="11" x2="14" y2="11"/></svg>
              </button>
            </div>

            <!-- 状态徽章 -->
            <span class="status-tag" :class="`tag-${item.status}`">{{ statusLabel(item.status) }}</span>
          </div>

          <!-- 信息区 -->
          <div class="card-info">
            <div class="info-top">
              <h3 class="th-title">{{ item.name }}</h3>
              <div class="info-tags">
                <span class="th-platform" :class="`pf-${platformOf(item)}`">{{ platformLabelOf(item) }}</span>
              </div>
            </div>
            <p v-if="item.description" class="th-desc">{{ item.description }}</p>

            <div class="info-meta">
              <span class="meta-author">
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="13.5" cy="6.5" r=".5"/><circle cx="17.5" cy="10.5" r=".5"/><circle cx="8.5" cy="7.5" r=".5"/><circle cx="6.5" cy="12.5" r=".5"/><path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c.926 0 1.648-.746 1.648-1.688 0-.437-.18-.835-.437-1.125-.29-.289-.438-.652-.438-1.125a1.64 1.64 0 0 1 1.668-1.668h1.996c3.051 0 5.555-2.503 5.555-5.554C21.965 6.012 17.461 2 12 2z"/></svg>
                作者：{{ item.author_name || '未知' }}
              </span>
              <span class="meta-date">{{ fmtDateTime(item.created_at) || '-' }}</span>
            </div>

            <div class="info-meta">
              <span class="meta-uploader">
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
                {{ uploaderText(item) }}
              </span>
            </div>

            <!-- 审核信息 -->
            <div v-if="item.reviewed_by && item.reviewed_at" class="review-info">
              审核人：{{ item.reviewed_by }} · {{ fmtDateTime(item.reviewed_at) }}
            </div>
            <div v-else-if="item.status === 'pending'" class="review-info review-pending">
              等待审核
            </div>

            <!-- 操作按钮 -->
            <div class="card-actions">
              <button class="act-btn act-detail" @click="openDetail(item)">详情</button>
              <template v-if="item.status === 'pending'">
                <button class="act-btn act-approve" @click="openReview(item)">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>
                  编辑审核
                </button>
              </template>
              <template v-else-if="item.status === 'rejected'">
                <button class="act-btn act-delete" @click="deleteTheme(item.id)">删除</button>
              </template>
              <template v-else-if="item.status === 'normal'">
                <button class="act-btn act-disable" @click="changeStatus(item.id, 'disabled')">下架</button>
                <button class="act-btn act-delete" @click="deleteTheme(item.id)">删除</button>
              </template>
              <template v-else-if="item.status === 'disabled'">
                <button class="act-btn act-approve" @click="changeStatus(item.id, 'normal')">恢复</button>
                <button class="act-btn act-delete" @click="deleteTheme(item.id)">删除</button>
              </template>
            </div>
          </div>
        </div>
      </TransitionGroup>
    </div>

    <!-- 主题详情弹层 -->
    <Transition name="modal">
      <div v-if="detailVisible && detailItem" class="modal-backdrop" @click.self="closeDetail">
        <div class="modal-dialog detail-dialog">
          <div class="modal-head">
            <h3>主题详情</h3>
            <button class="modal-close" @click="closeDetail">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
          <div class="modal-body">
            <div v-if="detailItem.preview_url" class="detail-cover">
              <img :src="detailItem.preview_url" :alt="detailItem.name" @error="onImgError" />
            </div>
            <div class="detail-grid">
              <div class="detail-row">
                <span class="detail-label">名称</span>
                <span class="detail-value">{{ detailItem.name }}</span>
              </div>
              <div class="detail-row">
                <span class="detail-label">作者</span>
                <span class="detail-value">{{ detailItem.author_name || '-' }}</span>
              </div>
              <div class="detail-row">
                <span class="detail-label">平台</span>
                <span class="detail-value">{{ platformLabelOf(detailItem) }}</span>
              </div>
              <div class="detail-row">
                <span class="detail-label">状态</span>
                <span class="detail-value">{{ statusLabel(detailItem.status) }}</span>
              </div>
              <div class="detail-row">
                <span class="detail-label">上传者</span>
                <span class="detail-value">{{ uploaderText(detailItem) }}</span>
              </div>
              <div class="detail-row">
                <span class="detail-label">上传时间</span>
                <span class="detail-value">{{ fmtDateTime(detailItem.created_at) || '-' }}</span>
              </div>
              <div v-if="detailItem.reviewed_by && detailItem.reviewed_at" class="detail-row">
                <span class="detail-label">审核信息</span>
                <span class="detail-value">{{ detailItem.reviewed_by }} · {{ fmtDateTime(detailItem.reviewed_at) }}</span>
              </div>
              <div v-if="detailItem.description" class="detail-row">
                <span class="detail-label">描述</span>
                <span class="detail-value">{{ detailItem.description }}</span>
              </div>
            </div>
            <div class="detail-json">
              <div class="detail-json-head">
                <span class="detail-label">主题包 JSON</span>
                <button v-if="payloadText(detailItem)" class="text-btn" @click="copyPayload">复制</button>
              </div>
              <pre class="payload-pre">{{ payloadText(detailItem) || '（无）' }}</pre>
            </div>
          </div>
          <div class="modal-foot">
            <button class="btn-cancel" @click="closeDetail">关闭</button>
          </div>
        </div>
      </div>
    </Transition>

    <Transition name="modal">
      <div v-if="reviewVisible && reviewItem" class="modal-backdrop" @click.self="closeReview">
        <div class="modal-dialog editor-dialog">
          <div class="modal-head">
            <h3>主题审核</h3>
            <div class="review-head-actions">
              <button class="review-btn review-btn--reject" @click="reviewAction('rejected')">拒绝</button>
              <button class="review-btn review-btn--pass" @click="reviewAction('normal')">通过</button>
              <button class="modal-close" @click="closeReview">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
              </button>
            </div>
          </div>
          <div class="editor-frame-wrap">
            <iframe v-if="reviewVisible && reviewItem" ref="reviewFrame" src="/theme-editor?preview=1" class="editor-frame" title="主题预览" @load="postPreviewPayload"></iframe>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { adminApi, showToast } from '@/api/client'
import { webConfirm } from '@/utils/webDialog'
import { fmtDateTime } from '@/utils/time'

interface ThemeItem {
  id: number
  name: string
  description: string
  payload: string
  payload_json: Record<string, any> | null
  platform: string
  preview_url: string
  thumbnail_url: string
  author_name: string
  wallpaper_id: number | null
  sort_order: number
  status: string
  uploaded_by: string
  uploaded_by_nickname: string
  reviewed_at: string | null
  reviewed_by: string
  created_at: string
  [key: string]: any
}

const filters = [
  { value: 'all', label: '全部' },
  { value: 'pending', label: '待审核' },
  { value: 'normal', label: '已上架' },
  { value: 'rejected', label: '已拒绝' },
  { value: 'disabled', label: '已下架' },
]

type PlatformKey = 'all' | 'desktop' | 'mobile'

const PLATFORMS: { key: PlatformKey; label: string }[] = [
  { key: 'all', label: '全部' },
  { key: 'mobile', label: '移动端' },
  { key: 'desktop', label: '桌面端' },
]

function platformOf(item: ThemeItem): 'mobile' | 'desktop' {
  return item?.platform === 'mobile' ? 'mobile' : 'desktop'
}

function platformLabelKey(key: string): string {
  return PLATFORMS.find(p => p.key === key)?.label || key
}

function platformLabelOf(item: ThemeItem): string {
  return platformLabelKey(platformOf(item))
}

const statusMap: Record<string, { label: string; cls: string }> = {
  normal: { label: '已上架', cls: 'tag-normal' },
  disabled: { label: '已下架', cls: 'tag-disabled' },
  pending: { label: '待审核', cls: 'tag-pending' },
  rejected: { label: '已拒绝', cls: 'tag-rejected' },
}

function statusLabel(s: string): string {
  return statusMap[s]?.label || s
}

function isUserUpload(item: ThemeItem): boolean {
  return !!item.uploaded_by && item.uploaded_by !== 'admin'
}

function uploaderText(item: ThemeItem): string {
  if (isUserUpload(item)) {
    const nick = (item.uploaded_by_nickname || '').trim()
    return nick ? `${nick}（${item.uploaded_by}）` : item.uploaded_by
  }
  return '管理员'
}

function countByStatus(status: string): number {
  if (status === 'all') return platformList.value.length
  return platformList.value.filter(t => t.status === status).length
}

// ===== 列表 =====
const themes = ref<ThemeItem[]>([])
const loading = ref(true)
const activeFilter = ref('all')
const platformFilter = ref<PlatformKey>('all')

const pendingCount = computed(() => themes.value.filter(t => t.status === 'pending').length)
const platformList = computed(() =>
  platformFilter.value === 'all' ? themes.value : themes.value.filter(t => platformOf(t) === platformFilter.value)
)
const currentPlatformLabel = computed(() => platformLabelKey(platformFilter.value))
const filteredList = computed(() => {
  if (activeFilter.value === 'all') return platformList.value
  return platformList.value.filter(t => t.status === activeFilter.value)
})

async function loadList(silent = false) {
  if (!silent) loading.value = true
  const res = await adminApi<ThemeItem[]>('list_themes')
  if (res.code === 200 && res.data) {
    themes.value = Array.isArray(res.data) ? res.data : []
  } else {
    themes.value = []
  }
  if (!silent) loading.value = false
}

function onImgError(e: Event) {
  const img = e.target as HTMLImageElement
  img.style.display = 'none'
}

// ===== 状态变更 =====
async function changeStatus(id: number, status: string) {
  const tips: Record<string, string> = {
    normal: '确定通过审核并上架此主题吗？',
    rejected: '确定拒绝此主题吗？',
    disabled: '确定下架此主题吗？',
  }
  if (tips[status]) {
    const ok = await webConfirm(tips[status], { title: '主题状态变更', confirmText: '确认' })
    if (!ok) return
  }
  const res = await adminApi('change_theme_status', { id, status })
  if (res.code === 200) {
    showToast('操作成功', 'success')
    loadList()
  } else {
    showToast(res.msg || '操作失败')
  }
}

// ===== 删除 =====
async function deleteTheme(id: number) {
  const ok = await webConfirm('确定要删除此主题吗？删除后不可恢复。', { title: '删除主题', confirmText: '确认删除' })
  if (!ok) return
  const res = await adminApi('delete_theme', { id })
  if (res.code === 200) {
    showToast('删除成功', 'success')
    loadList()
  } else {
    showToast(res.msg || '删除失败')
  }
}

// ===== 详情弹层 =====
const detailVisible = ref(false)
const detailItem = ref<ThemeItem | null>(null)

function openDetail(item: ThemeItem) {
  detailItem.value = item
  detailVisible.value = true
}

const reviewVisible = ref(false)
const reviewItem = ref<ThemeItem | null>(null)
const reviewId = ref(0)
const reviewFrame = ref<HTMLIFrameElement | null>(null)

function openReview(item: ThemeItem) {
  reviewItem.value = item
  reviewId.value = item.id
  reviewVisible.value = true
}

// 轻量预览：iframe（/theme-editor?preview=1，nginx 会 301 到 topic 独立站）加载后
// postMessage 传入 payload，编辑器纯前端渲染，不做任何身份校验/接口调用
function postPreviewPayload() {
  const item = reviewItem.value
  const frame = reviewFrame.value
  if (!item || !frame?.contentWindow) return
  let pj = item.payload_json
  if (!pj || typeof pj !== 'object') {
    try { pj = JSON.parse(item.payload || '{}') } catch { pj = {} }
  }
  frame.contentWindow.postMessage(
    { type: 'xy_theme_preview', payload: { id: item.id, name: item.name, description: item.description, platform: item.platform, payload_json: pj } },
    '*',
  )
}

function closeReview() {
  reviewVisible.value = false
  reviewItem.value = null
  loadList(true)
}

async function reviewAction(status: 'normal' | 'rejected') {
  reviewVisible.value = false
  reviewItem.value = null
  await changeStatus(reviewId.value, status)
}

function closeDetail() {
  detailVisible.value = false
}

function payloadText(item: ThemeItem): string {
  if (item.payload_json && typeof item.payload_json === 'object') {
    try {
      return JSON.stringify(item.payload_json, null, 2)
    } catch {
      /* ignore */
    }
  }
  return item.payload || ''
}

async function copyPayload() {
  if (!detailItem.value) return
  const text = payloadText(detailItem.value)
  if (!text) return
  try {
    await navigator.clipboard.writeText(text)
    showToast('已复制到剪贴板', 'success')
  } catch {
    showToast('复制失败，请手动复制')
  }
}

onMounted(() => {
  loadList()
  startPolling()
})

onUnmounted(() => {
  stopPolling()
})

let pollTimer: ReturnType<typeof setInterval> | null = null
function startPolling() { stopPolling(); pollTimer = setInterval(() => loadList(true), 30000) }
function stopPolling() { if (pollTimer) { clearInterval(pollTimer); pollTimer = null } }
</script>

<style scoped>
.th-page {
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
  padding: 10px 20px;
  border-radius: 10px;
  border: 1px solid var(--border);
  background: var(--card-solid);
  color: var(--text-light);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}
.btn-refresh:hover { transform: translateY(-2px); box-shadow: 0 6px 20px rgba(0, 0, 0, 0.12); color: var(--text); }
.btn-refresh:active { transform: scale(0.96); }
.btn-refresh:disabled { cursor: not-allowed; opacity: 0.6; }

.platform-tabs {
  display: inline-flex;
  gap: 4px;
  padding: 4px;
  margin-bottom: 18px;
  background: var(--card-solid);
  border: 1px solid var(--border);
  border-radius: 12px;
}
.platform-tab {
  border: none;
  border-radius: 9px;
  background: transparent;
  color: var(--text-light);
  padding: 8px 20px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}
.platform-tab:hover { color: var(--text); }
.platform-tab.active {
  background: var(--accent-soft);
  color: var(--accent);
}

/* ===== 筛选栏 ===== */
.filter-bar {
  display: flex;
  gap: 8px;
  margin-bottom: 24px;
  flex-wrap: wrap;
}
.filter-pill {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  border-radius: 20px;
  border: 1.5px solid var(--border);
  background: var(--card-solid);
  font-size: 13px;
  font-weight: 500;
  color: var(--text-light);
  cursor: pointer;
  transition: all 0.2s;
}
.filter-pill:hover { border-color: var(--text-muted); }
.filter-pill.active {
  background: var(--accent);
  border-color: var(--accent);
  color: var(--white);
  font-weight: 600;
}
.pill-count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 18px;
  height: 18px;
  padding: 0 5px;
  border-radius: 9px;
  font-size: 11px;
  font-weight: 700;
  background: rgba(0, 0, 0, 0.08);
}
.filter-pill.active .pill-count { background: rgba(255, 255, 255, 0.25); }

/* ===== 状态 ===== */
.state-box {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 60px 20px;
  color: var(--text-muted);
  font-size: 14px;
}
.state-empty { padding: 80px 20px; }
.empty-icon { color: #d1d5db; margin-bottom: 4px; }
.empty-title { font-size: 16px; font-weight: 600; color: var(--text-light); margin: 0; }
.empty-sub { font-size: 13px; color: var(--text-muted); margin: 0; }
.spinner {
  width: 28px; height: 28px;
  border: 3px solid var(--border);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }

/* ===== 画廊网格 ===== */
.gallery-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 18px;
}

.th-card {
  background: var(--card-solid);
  border-radius: 14px;
  border: 1px solid var(--border);
  overflow: hidden;
  transition: transform 0.3s cubic-bezier(0.16, 1, 0.3, 1), box-shadow 0.3s ease, border-color 0.2s;
  animation: cardEnter 0.5s cubic-bezier(0.16, 1, 0.3, 1) backwards;
  display: flex;
  flex-direction: column;
}
.th-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.08);
  border-color: transparent;
}
@keyframes cardEnter {
  from { opacity: 0; transform: translateY(20px); }
  to { opacity: 1; transform: translateY(0); }
}

.thumb-wrap {
  position: relative;
  width: 100%;
  aspect-ratio: 16 / 10;
  overflow: hidden;
  background: #f5f5f5;
}
.thumb-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: transform 0.4s cubic-bezier(0.16, 1, 0.3, 1);
}
.th-card:hover .thumb-img { transform: scale(1.06); }
.thumb-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #d1d5db;
}

.thumb-overlay {
  position: absolute;
  inset: 0;
  background: linear-gradient(to top, rgba(0,0,0,0.4), transparent 50%);
  opacity: 0;
  transition: opacity 0.3s;
  display: flex;
  align-items: flex-end;
  justify-content: flex-end;
  padding: 10px;
}
.th-card:hover .thumb-overlay { opacity: 1; }
.overlay-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  border: none;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.9);
  color: var(--text);
  cursor: pointer;
  backdrop-filter: blur(4px);
  transition: background 0.2s;
}
.overlay-btn:hover { background: var(--card-solid); }

.status-tag {
  position: absolute;
  top: 10px;
  left: 10px;
  padding: 3px 10px;
  border-radius: 20px;
  font-size: 11px;
  font-weight: 600;
  backdrop-filter: blur(4px);
}
.tag-normal { background: rgba(16, 185, 129, 0.9); color: #fff; }
.tag-disabled { background: rgba(156, 163, 175, 0.9); color: #fff; }
.tag-pending { background: rgba(245, 158, 11, 0.9); color: #fff; }
.tag-rejected { background: rgba(239, 68, 68, 0.9); color: #fff; }

.card-info {
  padding: 14px 16px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  flex: 1;
}
.info-top {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 8px;
}
.info-tags {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}
.th-platform {
  font-size: 11px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 10px;
  white-space: nowrap;
}
.pf-desktop { background: rgba(59, 130, 246, 0.12); color: #3b82f6; }
.pf-mobile { background: rgba(16, 185, 129, 0.12); color: #10b981; }
.th-title {
  font-size: 14px;
  font-weight: 700;
  margin: 0;
  color: var(--text);
  line-height: 1.4;
}
.th-desc {
  font-size: 12px;
  color: var(--text-muted);
  margin: 0;
  line-height: 1.5;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
.info-meta {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 2px;
}
.meta-author,
.meta-uploader {
  display: flex;
  align-items: center;
  gap: 4px;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.st-normal .meta-uploader, .st-disabled .meta-uploader { color: #888; }
.st-pending .meta-uploader, .st-rejected .meta-uploader { color: #6366f1; }

.review-info {
  font-size: 11px;
  color: var(--text-muted);
  padding-top: 4px;
  border-top: 1px solid #f5f5f5;
}
.review-pending { color: #f59e0b; font-weight: 500; }

.card-actions {
  display: flex;
  gap: 6px;
  margin-top: auto;
  padding-top: 8px;
}
.act-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 6px 12px;
  border-radius: 8px;
  border: none;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}
.act-btn:active { transform: scale(0.95); }
.act-detail { background: rgba(59, 130, 246, 0.1); color: #3b82f6; }
.act-detail:hover { background: rgba(59, 130, 246, 0.18); }
.act-approve { background: #ecfdf5; color: #10b981; }
.act-approve:hover { background: #d1fae5; }
.act-reject { background: rgba(236, 65, 65, 0.12); color: #ef4444; }
.act-reject:hover { background: #fee2e2; }
.act-disable { background: rgba(245, 158, 11, 0.14); color: #f59e0b; }
.act-disable:hover { background: #fef3c7; }
.act-delete { background: #f5f5f5; color: #888; }
.act-delete:hover { background: rgba(236, 65, 65, 0.12); color: #ef4444; }

/* ===== 弹窗 ===== */
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.35);
  backdrop-filter: blur(6px);
  -webkit-backdrop-filter: blur(6px);
  z-index: 9999;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
}
.modal-dialog {
  background: var(--card-solid);
  border-radius: 18px;
  width: 100%;
  max-width: 480px;
  max-height: 90vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.16);
}
.detail-dialog { max-width: 640px; }
.editor-dialog { width: min(88vh, 92vw, 780px); max-width: min(88vh, 92vw, 780px); height: min(88vh, 92vw, 780px); }
.editor-frame-wrap { flex: 1; min-height: 0; }
.editor-frame { width: 100%; height: 100%; border: none; display: block; }
.review-head-actions { display: flex; align-items: center; gap: 10px; }
.review-btn {
  height: 34px; padding: 0 16px; border-radius: 999px; font-size: 13px; font-weight: 700;
  cursor: pointer; border: 1px solid transparent; transition: all .2s; white-space: nowrap;
}
.review-btn--reject { background: transparent; border-color: #ec4141; color: #ec4141; }
.review-btn--reject:hover { background: #ec4141; color: #fff; }
.review-btn--pass { background: #0a9d58; color: #fff; }
.review-btn--pass:hover { background: #08854b; }
.modal-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px 16px;
}
.modal-head h3 { font-size: 17px; font-weight: 700; margin: 0; }
.modal-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: 8px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.2s;
}
.modal-close:hover { background: #f5f5f5; color: var(--text); }

.modal-body {
  padding: 0 24px;
  overflow-y: auto;
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.detail-cover {
  border-radius: 12px;
  overflow: hidden;
  border: 1px solid var(--border);
  background: #f5f5f5;
}
.detail-cover img {
  display: block;
  width: 100%;
  max-height: 320px;
  object-fit: contain;
}
.detail-grid {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.detail-row {
  display: flex;
  gap: 12px;
  font-size: 13px;
  line-height: 1.6;
}
.detail-label {
  flex-shrink: 0;
  width: 64px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-muted);
}
.detail-value {
  min-width: 0;
  color: var(--text);
  word-break: break-all;
}
.detail-json {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.detail-json-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.text-btn {
  border: none;
  background: transparent;
  color: var(--accent);
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
}
.payload-pre {
  margin: 0;
  padding: 12px;
  background: #f8fafc;
  border: 1px solid var(--border);
  border-radius: 10px;
  font-size: 12px;
  line-height: 1.6;
  font-family: Consolas, Monaco, 'Courier New', monospace;
  color: var(--text);
  white-space: pre-wrap;
  word-break: break-all;
  max-height: 320px;
  overflow: auto;
}

.modal-foot {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  padding: 16px 24px 20px;
}
.btn-cancel {
  padding: 10px 20px;
  border-radius: 10px;
  border: 1.5px solid var(--border);
  background: var(--card-solid);
  font-size: 14px;
  font-weight: 500;
  color: var(--text-light);
  cursor: pointer;
  transition: all 0.2s;
}
.btn-cancel:hover { background: #f5f5f5; }

/* ===== Transition 动画 ===== */
.fade-down-enter-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-down-enter-from { opacity: 0; transform: translateY(-12px); }
.fade-up-enter-active { transition: all 0.5s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-up-enter-from { opacity: 0; transform: translateY(16px); }

.modal-enter-active, .modal-leave-active { transition: opacity 0.3s; }
.modal-enter-active .modal-dialog, .modal-leave-active .modal-dialog {
  transition: transform 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}
.modal-enter-from, .modal-leave-to { opacity: 0; }
.modal-enter-from .modal-dialog, .modal-leave-to .modal-dialog { transform: scale(0.92) translateY(20px); }

.card-enter-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.card-leave-active { transition: all 0.3s ease; }
.card-enter-from { opacity: 0; transform: translateY(20px); }
.card-leave-to { opacity: 0; transform: scale(0.9); }

@media (max-width: 768px) {
  .page-header { flex-direction: column; }
  .gallery-grid { grid-template-columns: repeat(auto-fill, minmax(160px, 1fr)); gap: 12px; }
}
</style>
