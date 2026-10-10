<template>
  <div class="mobile-page">
    <!-- 页头 -->
    <div class="th-header">
      <div class="th-header-info">
        <h2 class="th-title">
          主题中心
          <span v-if="pendingCount > 0" class="pending-badge">{{ pendingCount }} 项待审核</span>
        </h2>
        <p class="th-desc">主题包按客户端平台隔离下发：用户上传的主题继承其客户端平台，审核通过后仅对该平台展示，下架后不再对用户可见。</p>
      </div>
      <button class="mobile-btn th-refresh-btn" :disabled="loading" @click="loadList()">
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.8" stroke-linecap="round" stroke-linejoin="round"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/></svg>
        刷新
      </button>
    </div>

    <!-- 平台切换 -->
    <div class="platform-tabs">
      <button
        v-for="p in PLATFORMS"
        :key="p.key"
        class="platform-tab"
        :class="{ active: platformFilter === p.key }"
        @click="platformFilter = p.key"
      >{{ p.label }}</button>
    </div>

    <!-- 加载中 -->
    <div v-if="loading" class="mobile-empty">加载中...</div>

    <!-- 状态筛选 -->
    <div v-else class="th-tabs">
      <button
        v-for="f in filters"
        :key="f.value"
        class="th-tab"
        :class="{ active: activeFilter === f.value }"
        @click="activeFilter = f.value"
      >
        {{ f.label }}
        <span v-if="countByStatus(f.value) > 0" class="tab-count">{{ countByStatus(f.value) }}</span>
      </button>
    </div>

    <!-- 空状态 -->
    <div v-if="!loading && filteredList.length === 0" class="mobile-empty">
      <div class="empty-title">{{ activeFilter === 'all' ? `暂无${currentPlatformLabel}主题` : '该分类下暂无主题' }}</div>
      <div class="empty-sub">用户上传的主题包会出现在这里等待审核</div>
    </div>

    <!-- 主题画廊 -->
    <div v-if="!loading && filteredList.length > 0" class="th-gallery">
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
            <svg width="30" height="30" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="13.5" cy="6.5" r=".5"/><circle cx="17.5" cy="10.5" r=".5"/><circle cx="8.5" cy="7.5" r=".5"/><circle cx="6.5" cy="12.5" r=".5"/><path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c.926 0 1.648-.746 1.648-1.688 0-.437-.18-.835-.437-1.125-.29-.289-.438-.652-.438-1.125a1.64 1.64 0 0 1 1.668-1.668h1.996c3.051 0 5.555-2.503 5.555-5.554C21.965 6.012 17.461 2 12 2z"/></svg>
          </div>
          <span class="status-tag" :class="`tag-${item.status}`">{{ statusLabel(item.status) }}</span>
        </div>

        <!-- 信息区 -->
        <div class="card-info">
          <div class="info-top">
            <h3 class="card-title">{{ item.name }}</h3>
            <div class="info-tags">
              <span class="card-platform" :class="`pf-${platformOf(item)}`">{{ platformLabelOf(item) }}</span>
            </div>
          </div>
          <p v-if="item.description" class="card-desc">{{ item.description }}</p>
          <div class="info-meta">
            <span class="meta-author">
              <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="13.5" cy="6.5" r=".5"/><circle cx="17.5" cy="10.5" r=".5"/><circle cx="8.5" cy="7.5" r=".5"/><circle cx="6.5" cy="12.5" r=".5"/><path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c.926 0 1.648-.746 1.648-1.688 0-.437-.18-.835-.437-1.125-.29-.289-.438-.652-.438-1.125a1.64 1.64 0 0 1 1.668-1.668h1.996c3.051 0 5.555-2.503 5.555-5.554C21.965 6.012 17.461 2 12 2z"/></svg>
              {{ item.author_name || '未知' }}
            </span>
            <span class="meta-date">{{ fmtDateTime(item.created_at) || '-' }}</span>
          </div>
          <div class="info-meta">
            <span class="meta-uploader">
              <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
              {{ uploaderText(item) }}
            </span>
          </div>
          <div v-if="item.reviewed_by && item.reviewed_at" class="review-info">审核人：{{ item.reviewed_by }} · {{ fmtDateTime(item.reviewed_at) }}</div>
          <div v-else-if="item.status === 'pending'" class="review-info review-pending">等待审核</div>

          <!-- 操作按钮 -->
          <div class="card-actions">
            <button class="act-btn act-detail" @click="openDetail(item)">详情</button>
            <template v-if="item.status === 'pending'">
              <button class="act-btn act-approve" @click="openReview(item)">编辑审核</button>
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
    </div>

    <!-- 主题详情弹层 -->
    <Transition name="modal" @before-leave="removeBackdropBlur">
      <div v-if="detailVisible && detailItem" class="modal-backdrop" @click.self="closeDetail">
        <div class="modal-dialog">
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
                <span>主题包 JSON</span>
                <button v-if="payloadText(detailItem)" class="text-btn" @click="copyPayload">复制</button>
              </div>
              <pre class="payload-pre">{{ payloadText(detailItem) || '（无）' }}</pre>
            </div>
            <p class="modal-tip">通过审核的主题将上架并对对应平台的用户可见。</p>
          </div>
          <div class="modal-foot">
            <button class="modal-btn cancel" @click="closeDetail">关闭</button>
          </div>
        </div>
      </div>
    </Transition>

    <!-- 审核弹层：iframe 轻量预览（/theme-editor?preview=1，nginx 301 到 topic 独立站），
         加载后 postMessage 传 payload 纯前端渲染，无身份校验/接口调用 -->
    <Transition name="modal" @before-leave="removeBackdropBlur">
      <div v-if="reviewVisible && reviewItem" class="modal-backdrop" @click.self="closeReview">
        <div class="modal-dialog editor-dialog">
          <div class="modal-head">
            <h3>编辑审核</h3>
            <div class="review-head-actions">
              <button class="review-btn review-btn--reject" @click="reviewAction('rejected')">拒绝</button>
              <button class="review-btn review-btn--pass" @click="reviewAction('normal')">通过</button>
              <button class="modal-close" @click="closeReview">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
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
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'
import './MobilePage.css'
import { mobileConfirm, removeBackdropBlur } from '@/utils/mobileDialog'
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

const statusMap: Record<string, string> = {
  normal: '已上架',
  disabled: '已下架',
  pending: '待审核',
  rejected: '已拒绝',
}

function statusLabel(s: string): string {
  return statusMap[s] || s
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
  themes.value = res.code === 200 && res.data ? (Array.isArray(res.data) ? res.data : []) : []
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
    const ok = await mobileConfirm(tips[status], { title: '主题状态变更', confirmText: '确认' })
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
  const ok = await mobileConfirm('确定要删除此主题吗？删除后不可恢复。', { title: '删除主题', confirmText: '确认删除', danger: true })
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

function closeDetail() {
  detailVisible.value = false
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
  // item 来自 reactive 列表，payload_json 是 Vue Proxy——结构化克隆不支持会抛
  // DataCloneError（消息静默丢失），必须先还原成纯对象
  try { pj = JSON.parse(JSON.stringify(pj)) } catch { pj = {} }
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
onUnmounted(() => stopPolling())

let pollTimer: ReturnType<typeof setInterval> | null = null
function startPolling() { stopPolling(); pollTimer = setInterval(() => loadList(true), 30000) }
function stopPolling() { if (pollTimer) { clearInterval(pollTimer); pollTimer = null } }
</script>
<style scoped>
.th-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}
.th-header-info { min-width: 0; flex: 1; }
.th-title { font-size: 18px; font-weight: 850; margin: 0 0 4px; color: var(--text); display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
.pending-badge { font-size: 11px; font-weight: 700; padding: 3px 10px; border-radius: 20px; background: rgba(245, 158, 11, 0.14); color: #f59e0b; }
.th-desc { font-size: 12px; color: var(--text-light); line-height: 1.6; margin: 0; }
.th-refresh-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  flex-shrink: 0;
  padding: 9px 16px;
  white-space: nowrap;
}
.th-refresh-btn:disabled { opacity: 0.55; }

.platform-tabs {
  display: inline-flex;
  gap: 4px;
  padding: 4px;
  margin-bottom: 14px;
  background: var(--control-bg, var(--card-solid));
  border: 1px solid var(--border);
  border-radius: 12px;
}
.platform-tab {
  border: none;
  border-radius: 9px;
  background: transparent;
  color: var(--text-light, var(--text-muted));
  padding: 7px 18px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}
.platform-tab.active {
  background: var(--card-solid);
  color: var(--accent);
}

.th-tabs {
  display: flex;
  gap: 8px;
  overflow-x: auto;
  padding-bottom: 2px;
}
.th-tab {
  flex: 0 0 auto;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border-radius: 999px;
  border: 1.5px solid var(--border);
  background: var(--card);
  font-size: 12px;
  font-weight: 700;
  color: var(--text-light);
  cursor: pointer;
  transition: all 0.18s;
}
.th-tab.active {
  background: #EC4141;
  border-color: #EC4141;
  color: #fff;
}
.tab-count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 18px;
  height: 18px;
  padding: 0 5px;
  border-radius: 9px;
  font-size: 10px;
  font-weight: 800;
  background: rgba(0, 0, 0, 0.08);
}
.th-tab.active .tab-count { background: rgba(255, 255, 255, 0.25); }

.th-gallery {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}
.th-card {
  border: 1px solid var(--border);
  border-radius: 16px;
  background: var(--card);
  overflow: hidden;
  box-shadow: var(--shadow-soft);
  display: flex;
  flex-direction: column;
  animation: cardIn 0.4s var(--motion, cubic-bezier(0.16, 1, 0.3, 1)) backwards;
}
@keyframes cardIn {
  from { opacity: 0; transform: translateY(14px); }
  to { opacity: 1; transform: translateY(0); }
}
.thumb-wrap {
  position: relative;
  width: 100%;
  aspect-ratio: 16 / 10;
  overflow: hidden;
  background: var(--control-bg);
}
.thumb-img { width: 100%; height: 100%; object-fit: cover; }
.thumb-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
}
.status-tag {
  position: absolute;
  top: 8px;
  left: 8px;
  padding: 2px 8px;
  border-radius: 999px;
  font-size: 10px;
  font-weight: 700;
  color: #fff;
}
.tag-normal { background: rgba(16, 185, 129, 0.92); }
.tag-disabled { background: rgba(156, 163, 175, 0.92); }
.tag-pending { background: rgba(245, 158, 11, 0.92); }
.tag-rejected { background: rgba(239, 68, 68, 0.92); }

.card-info { padding: 10px 12px; display: flex; flex-direction: column; gap: 5px; flex: 1; }
.info-top { display: flex; justify-content: space-between; align-items: flex-start; gap: 6px; }
.info-tags { display: inline-flex; align-items: center; gap: 4px; flex-shrink: 0; }
.card-platform { font-size: 10px; font-weight: 700; padding: 2px 7px; border-radius: 999px; white-space: nowrap; }
.pf-desktop { background: rgba(59, 130, 246, 0.12); color: #3b82f6; }
.pf-mobile { background: rgba(16, 185, 129, 0.12); color: #10b981; }
.card-title { font-size: 13px; font-weight: 750; margin: 0; color: var(--text); line-height: 1.35; word-break: break-word; }
.card-desc {
  font-size: 11px; color: var(--text-muted); margin: 0; line-height: 1.4;
  display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden;
}
.info-meta { display: flex; justify-content: space-between; align-items: center; gap: 6px; font-size: 10px; color: var(--text-muted); }
.meta-author, .meta-uploader { display: flex; align-items: center; gap: 3px; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.meta-date { flex-shrink: 0; }
.review-info { font-size: 10px; color: var(--text-muted); padding-top: 4px; border-top: 1px solid var(--border); }
.review-pending { color: #f59e0b; font-weight: 600; }

.card-actions { display: flex; gap: 6px; margin-top: auto; padding-top: 8px; }
.act-btn {
  flex: 1;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  padding: 6px 0;
  border-radius: 8px;
  border: none;
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
  transition: transform 0.15s;
}
.act-btn:active { transform: scale(0.94); }
.act-detail { background: rgba(59, 130, 246, 0.1); color: #3b82f6; }
.act-approve { background: rgba(16, 185, 129, 0.12); color: #10b981; }
.act-reject { background: rgba(239, 68, 68, 0.1); color: #ef4444; }
.act-disable { background: rgba(245, 158, 11, 0.12); color: #f59e0b; }
.act-delete { background: var(--control-bg); color: var(--text-muted); }

.empty-title { font-size: 14px; font-weight: 700; color: var(--text-light); margin-bottom: 4px; }
.empty-sub { font-size: 12px; color: var(--text-muted); }

.modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 32px 24px;
  background: rgba(15, 23, 42, 0.38);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
}
.modal-dialog {
  width: 100%;
  max-width: 360px;
  max-height: 88vh;
  border-radius: 22px;
  background: var(--card-solid, var(--card));
  box-shadow: 0 24px 60px rgba(15, 23, 42, 0.22);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
.editor-dialog { width: min(92vw, 82vh); max-width: min(92vw, 82vh); height: min(92vw, 82vh); }
.editor-frame-wrap { flex: 1; min-height: 0; }
.editor-frame { width: 100%; height: 100%; border: none; display: block; }
.review-head-actions { display: flex; align-items: center; gap: 8px; }
.review-btn {
  height: 32px; padding: 0 14px; border-radius: 999px; font-size: 13px; font-weight: 700;
  cursor: pointer; border: 1px solid transparent; white-space: nowrap;
}
.review-btn--reject { background: transparent; border-color: #ec4141; color: #ec4141; }
.review-btn--pass { background: #0a9d58; color: #fff; }
.modal-head { display: flex; align-items: center; justify-content: space-between; padding: 18px 20px 0; }
.modal-head h3 { margin: 0; font-size: 16px; font-weight: 850; color: var(--text); }
.modal-close { border: none; background: transparent; color: var(--text-muted); cursor: pointer; padding: 4px; border-radius: 8px; display: flex; }
.modal-close:active { background: var(--control-bg); color: var(--text); }
.modal-body {
  padding: 14px 20px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  overflow-y: auto;
}
.detail-cover {
  border-radius: 14px;
  overflow: hidden;
  border: 1px solid var(--border);
  background: var(--control-bg);
}
.detail-cover img {
  display: block;
  width: 100%;
  max-height: 240px;
  object-fit: contain;
}
.detail-grid {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.detail-row {
  display: flex;
  gap: 10px;
  font-size: 12px;
  line-height: 1.6;
}
.detail-label {
  flex-shrink: 0;
  width: 56px;
  font-size: 11px;
  font-weight: 700;
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
.detail-json-head { display: flex; align-items: center; justify-content: space-between; }
.detail-json-head span { font-size: 12px; font-weight: 700; color: var(--text-light); }
.text-btn { border: none; background: transparent; color: var(--accent); font-size: 12px; font-weight: 700; cursor: pointer; padding: 4px; }
.payload-pre {
  margin: 0;
  padding: 10px;
  background: var(--control-bg);
  border: 1px solid var(--border);
  border-radius: 12px;
  font-size: 10px;
  line-height: 1.6;
  font-family: Consolas, Monaco, 'Courier New', monospace;
  color: var(--text);
  white-space: pre-wrap;
  word-break: break-all;
  max-height: 240px;
  overflow: auto;
}
.modal-tip { margin: 0; font-size: 11px; color: var(--text-muted); line-height: 1.5; }
.modal-foot { display: flex; gap: 10px; padding: 14px 20px 18px; border-top: 1px solid var(--border); }
.modal-btn { flex: 1; padding: 11px; border-radius: 12px; font-size: 14px; font-weight: 750; cursor: pointer; display: inline-flex; align-items: center; justify-content: center; transition: all 0.18s; }
.modal-btn.cancel { border: 1px solid var(--border); background: transparent; color: var(--text-muted); }
.modal-btn.cancel:active { background: var(--control-bg); }

.modal-enter-active, .modal-leave-active { transition: opacity 0.24s var(--motion, cubic-bezier(0.16, 1, 0.3, 1)); }
.modal-enter-from, .modal-leave-to { opacity: 0; }
.modal-enter-active .modal-dialog { animation: modalIn 0.24s cubic-bezier(0.16, 1, 0.3, 1) forwards; }
.modal-leave-active .modal-dialog { animation: modalOut 0.2s ease forwards; }
@keyframes modalIn { from { opacity: 0; transform: scale(0.94); } to { opacity: 1; transform: scale(1); } }
@keyframes modalOut { from { opacity: 1; transform: scale(1); } to { opacity: 0; transform: scale(0.96); } }
</style>
