<template>
  <div class="mobile-page">
    <!-- 反馈限制 -->
    <section class="mobile-card mobile-form">
      <h3 class="mobile-card-title">反馈限制</h3>
      <input v-model="limitInput" class="mobile-input" type="number" placeholder="每日提交上限，0 表示不限" />
      <button class="mobile-btn primary" :disabled="limitSaving" @click="saveLimit">{{ limitSaving ? '保存中...' : '保存限制' }}</button>
    </section>

    <!-- 操作栏：排序 + 新建 -->
    <div class="mfb-ops">
      <div class="mfb-sort">
        <select v-model="sortMode" class="mobile-select" @change="loadList">
          <option value="post_time_desc">最新提交</option>
          <option value="post_time_asc">最早提交</option>
          <option value="update_desc">最近更新</option>
        </select>
      </div>
      <div class="mfb-op-btns">
        <button class="mobile-btn" @click="openStats">统计</button>
        <button class="mobile-btn primary" @click="openCreate">新建</button>
      </div>
    </div>

    <!-- 筛选标签 -->
    <div class="mobile-tabs">
      <button class="mobile-btn" :class="{ primary: status === '' }" @click="setStatus('')">全部</button>
      <button class="mobile-btn" :class="{ primary: status === 'pending' }" @click="setStatus('pending')">待处理</button>
      <button class="mobile-btn" :class="{ primary: status === 'processing' }" @click="setStatus('processing')">处理中</button>
      <button class="mobile-btn" :class="{ primary: status === 'resolved' }" @click="setStatus('resolved')">已解决</button>
      <button class="mobile-btn" :class="{ primary: status === 'rejected' }" @click="setStatus('rejected')">已拒绝</button>
    </div>
    <div class="mobile-tabs">
      <button class="mobile-btn" :class="{ primary: type === '' }" @click="type = ''">全部类型</button>
      <button class="mobile-btn" :class="{ primary: type === 'problem' }" @click="type = 'problem'">问题反馈</button>
      <button class="mobile-btn" :class="{ primary: type === 'suggestion' }" @click="type = 'suggestion'">功能建议</button>
    </div>

    <div v-if="loading" class="mobile-empty">加载中...</div>
    <div v-else-if="filteredList.length === 0" class="mobile-empty">暂无反馈</div>
    <div v-else class="mobile-list">
      <div v-for="f in filteredList" :key="f.id" class="mobile-item mfb-item">
        <div class="mobile-item-head">
          <div class="mfb-title-wrap">
            <span class="mfb-type" :class="f.feedback_type === 'suggestion' ? 'type-suggestion' : 'type-problem'">
              {{ f.feedback_type === 'suggestion' ? '建议' : '问题' }}
            </span>
            <div class="mobile-item-title">{{ f.title || '无标题' }}</div>
          </div>
          <span class="mobile-badge" :class="badgeClass(f.status)">{{ statusLabel(f.status) }}</span>
        </div>
        <div class="mfb-main">
          <div class="mfb-left">
            <div class="mfb-sub">{{ f.nickname || '匿名用户' }}<span v-if="f.ciyuanxi_id"> · {{ f.ciyuanxi_id }}</span><span v-if="f.assignee"> · 认领：{{ f.assignee }}</span></div>
            <div class="mobile-item-sub mfb-content">{{ f.content || '无内容' }}</div>
            <div v-if="f.status === 'resolved' && f.resolve_note" class="mfb-line mfb-resolve">完成说明：{{ f.resolve_note }}</div>
          </div>
          <div v-if="imagesOf(f).length > 0" class="mfb-right">
            <img v-for="(img, i) in imagesOf(f)" :key="i" :src="img" class="mfb-thumb" @click.stop="openViewer(imagesOf(f), i)" />
          </div>
        </div>
        <div class="mfb-timeline">
          <span>发布时间{{ fmtTime(f.created_at) }}</span>
          <span v-if="f.claimed_at"> | 认领时间：{{ fmtTime(f.claimed_at) }}</span>
          <span v-if="f.status === 'resolved' && f.resolved_at"> | 完成时间：{{ fmtTime(f.resolved_at) }}</span>
        </div>
        <div class="mobile-actions" @click.stop>
          <button v-if="f.status === 'pending'" class="mobile-btn" @click="claim(f)">认领</button>
          <button v-if="f.status === 'processing'" class="mobile-btn primary" @click="openResolve(f)">完成</button>
          <button v-if="f.status === 'pending' || f.status === 'processing'" class="mobile-btn danger" @click="setStatusOf(f, 'rejected')">拒绝</button>
          <button v-if="imagesOf(f).length > 0" class="mobile-btn" @click="openViewer(imagesOf(f), 0)">查看图片</button>
        </div>
      </div>
    </div>

    <!-- 完成说明弹窗 -->
    <div v-if="resolveVisible" class="mobile-dialog-overlay">
      <div class="mobile-dialog" style="display:flex;flex-direction:column;">
        <div class="mobile-dialog-title">完成反馈</div>
        <textarea v-model="resolveNote" class="mobile-dialog-input" rows="4" type="text" placeholder="请填写完成说明（必填）" style="min-height:90px;resize:vertical;"></textarea>
        <div class="mobile-dialog-actions">
          <button class="mobile-dialog-btn cancel" @click="closeResolve">取消</button>
          <button class="mobile-dialog-btn confirm" :disabled="!resolveNote.trim()" @click="confirmResolve">确认完成</button>
        </div>
      </div>
    </div>

    <!-- 新建事项弹窗 -->
    <div v-if="createVisible" class="mobile-dialog-overlay">
      <div class="mobile-dialog mfb-create" style="display:flex;flex-direction:column;max-width:420px;max-height:88vh;">
        <div class="mobile-dialog-title">新建事项</div>
        <div class="mfb-create-body">
          <div class="mfb-type-row">
            <button class="mfb-type-btn" :class="{ active: createType === 'problem' }" @click="createType = 'problem'">问题反馈</button>
            <button class="mfb-type-btn" :class="{ active: createType === 'suggestion' }" @click="createType = 'suggestion'">功能建议</button>
          </div>
          <input v-model="createTitle" class="mobile-dialog-input" type="text" placeholder="标题（最多 60 字）" maxlength="60" />
          <textarea v-model="createContent" class="mobile-dialog-input" rows="3" type="text" placeholder="内容描述（最多 1000 字）" maxlength="1000" style="min-height:80px;resize:vertical;"></textarea>
          <div class="mfb-dropzone" :class="{ dragging: createDragging, has: createImages.length > 0 }" @dragover.prevent="createDragging = true" @dragleave.prevent="createDragging = false" @drop.prevent="onDrop" @click="fileInput?.click()">
            <input ref="fileInput" type="file" accept="image/*" multiple hidden @change="onFileChange" />
            <div class="mfb-dropzone-text">
              <strong>{{ createImages.length > 0 ? `${createImages.length} 张图片已选择` : '点击或拖拽图片到此处' }}</strong>
              <span>最多 6 张，单张不超过 8MB</span>
            </div>
          </div>
          <div v-if="createImages.length > 0" class="mfb-preview">
            <div v-for="(img, i) in createImages" :key="i" class="mfb-preview-item">
              <img :src="img" class="mfb-preview-img" @click.stop="openViewer(createImages, i)" />
              <button class="mfb-preview-remove" @click.stop="createImages.splice(i, 1)">×</button>
            </div>
          </div>
          <label class="mfb-notify">
            <input v-model="createNotify" type="checkbox" />
            <span class="mfb-notify-box" :class="{ checked: createNotify }">✓</span>
            <span class="mfb-notify-text">
              <strong>外部同步通知</strong>
              <span>发布后向「外部通知」启用的邮箱发送邮件提醒</span>
            </span>
          </label>
        </div>
        <div class="mobile-dialog-actions">
          <button class="mobile-dialog-btn cancel" @click="closeCreate">取消</button>
          <button class="mobile-dialog-btn confirm" :disabled="createSaving || !createTitle.trim() || !createContent.trim()" @click="submitCreate">{{ createSaving ? '发布中...' : '发布' }}</button>
        </div>
      </div>
    </div>

    <!-- 统计弹窗 -->
    <div v-if="statsVisible" class="mobile-dialog-overlay">
      <div class="mobile-dialog mfb-stats" style="display:flex;flex-direction:column;max-width:420px;max-height:88vh;">
        <div class="mobile-dialog-title">管理员处理统计</div>
        <div class="mfb-stats-body">
          <div v-if="statsLoading" class="mobile-empty">加载中...</div>
          <template v-else>
            <div class="mfb-stats-total">
              <strong>{{ statsGrand }}</strong>
              <span>累计处理反馈总量</span>
            </div>
            <div v-if="statsList.length === 0" class="mobile-empty">暂无统计数据</div>
            <div v-else class="mfb-stats-list">
              <div v-for="(row, i) in statsList" :key="i" class="mfb-stats-row">
                <div class="mfb-stats-admin"><span class="mfb-dot" :class="row.admin_name === '未认领' ? 'dot-unclaimed' : `dot-${i % 4}`"></span>{{ row.admin_name }}</div>
                <div class="mfb-stats-nums">
                  <span>总量 <b>{{ row.total }}</b></span>
                  <span>处理中 <b class="c-processing">{{ row.processing }}</b></span>
                  <span>已解决 <b class="c-resolved">{{ row.resolved }}</b></span>
                  <span>已拒绝 <b class="c-rejected">{{ row.rejected }}</b></span>
                  <span>待处理 <b class="c-pending">{{ row.pending }}</b></span>
                </div>
              </div>
            </div>
          </template>
        </div>
        <div class="mobile-dialog-actions">
          <button class="mobile-dialog-btn cancel" @click="statsVisible = false">关闭</button>
        </div>
      </div>
    </div>

    <!-- 图片查看器 -->
    <div v-if="viewerVisible" class="mobile-dialog-overlay mfb-viewer">
      <button class="mfb-viewer-close" @click="viewerVisible = false">×</button>
      <button v-if="viewerList.length > 1" class="mfb-viewer-nav prev" @click="viewerPrev">‹</button>
      <img v-if="viewerList[viewerIndex]" :src="viewerList[viewerIndex]" class="mfb-viewer-img" />
      <button v-if="viewerList.length > 1" class="mfb-viewer-nav next" @click="viewerNext">›</button>
      <div v-if="viewerList.length > 1" class="mfb-viewer-counter">{{ viewerIndex + 1 }} / {{ viewerList.length }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref, computed } from 'vue'
import { adminApi, showToast } from '@/api/client'
import './MobilePage.css'

const loading = ref(false), status = ref(''), type = ref('')
const sortMode = ref('post_time_desc')
const list = ref<any[]>([])
const limitInput = ref(20)
const limitSaving = ref(false)

const statusMap: Record<string, string> = { pending: '待处理', processing: '处理中', resolved: '已解决', rejected: '已拒绝' }
const statusLabel = (s: string) => statusMap[s] || s
const badgeClass = (s: string) => (s === 'resolved' ? 'green' : s === 'rejected' ? 'red' : '')

const filteredList = computed(() => {
  let arr = list.value
  if (status.value !== '') arr = arr.filter(f => f.status === status.value)
  if (type.value !== '') arr = arr.filter(f => f.feedback_type === type.value)
  return arr
})

function imagesOf(f: any): string[] {
  if (!f.images) return []
  try {
    const a = JSON.parse(f.images)
    return Array.isArray(a) ? a.filter((u: string) => typeof u === 'string' && (u.startsWith('http') || u.startsWith('/'))) : []
  } catch { return [] }
}

function setStatus(s: string) { status.value = s }

// 时间戳格式化：2026年12月12日 21时25分（按数据库字面时间显示，兼容服务器/客户端）
function fmtTime(v: any): string {
  if (!v) return ''
  const m = String(v).match(/(\d{4})[-/](\d{1,2})[-/](\d{1,2})[T ]+(\d{1,2}):(\d{1,2})/)
  if (!m) return String(v)
  const [, y, mo, d, h, mi] = m.map(Number)
  return `${y}年${mo}月${d}日 ${h}时${String(mi).padStart(2, '0')}分`
}

async function loadList() {
  loading.value = true
  const res = await adminApi<any>('list_feedback', { status_filter: status.value, sort: sortMode.value })
  list.value = res.code === 200 && res.data ? (res.data.list || []) : []
  loading.value = false
}

async function loadLimit() {
  const res = await adminApi<any>('get_feedback_limit')
  if (res.code === 200 && res.data) limitInput.value = Number(res.data.feedback_daily_limit ?? 20)
}
async function saveLimit() {
  const v = Number(limitInput.value)
  if (!Number.isInteger(v) || v < 0 || v > 10000) return showToast('上限需为 0 到 10000 的整数')
  limitSaving.value = true
  const res = await adminApi('update_feedback_limit', { feedback_daily_limit: v })
  limitSaving.value = false
  if (res.code === 200) showToast('限制已保存', 'success'); else showToast(res.msg || '保存失败')
}

// 认领（不刷新页面）
async function claim(f: any) {
  const res = await adminApi('claim_feedback', { id: f.id })
  if (res.code === 200) {
    f.status = 'processing'
    f.assignee = res.data?.assignee || ''
    showToast('认领成功，已置为处理中', 'success')
  } else { showToast(res.msg || '认领失败') }
}

// 状态变更（本地更新）
async function setStatusOf(f: any, s: string) {
  const res = await adminApi('update_feedback_status', { id: f.id, status: s })
  if (res.code === 200) { f.status = s; showToast('状态已更新', 'success') } else { showToast(res.msg || '操作失败') }
}

// 完成弹窗
const resolveVisible = ref(false)
let resolveTarget: any = null
const resolveNote = ref('')
function openResolve(f: any) { resolveTarget = f; resolveNote.value = ''; resolveVisible.value = true }
function closeResolve() { resolveVisible.value = false; resolveTarget = null }
async function confirmResolve() {
  if (!resolveTarget || !resolveNote.value.trim()) return
  const res = await adminApi('resolve_feedback', { id: resolveTarget.id, note: resolveNote.value.trim() })
  if (res.code === 200) {
    resolveTarget.status = 'resolved'
    resolveTarget.resolve_note = resolveNote.value.trim()
    closeResolve()
    showToast('已标记为已完成', 'success')
  } else { showToast(res.msg || '操作失败') }
}

// 新建事项
const createVisible = ref(false)
const createType = ref<'problem' | 'suggestion'>('problem')
const createTitle = ref('')
const createContent = ref('')
const createImages = ref<string[]>([])
const createNotify = ref(false)
const createDragging = ref(false)
const createSaving = ref(false)
const fileInput = ref<HTMLInputElement | null>(null)

function openCreate() {
  createType.value = 'problem'; createTitle.value = ''; createContent.value = ''
  createImages.value = []; createNotify.value = false; createDragging.value = false; createSaving.value = false
  createVisible.value = true
}
function closeCreate() { if (!createSaving.value) createVisible.value = false }
function onFileChange(e: Event) {
  const input = e.target as HTMLInputElement
  if (input.files) handleFiles(Array.from(input.files))
  input.value = ''
}
function onDrop(e: DragEvent) {
  e.preventDefault(); createDragging.value = false
  const files = e.dataTransfer?.files
  if (files) handleFiles(Array.from(files))
}
function handleFiles(files: File[]) {
  const remaining = 6 - createImages.value.length
  if (remaining <= 0) return showToast('最多上传 6 张图片')
  for (const file of files.slice(0, remaining)) {
    if (!file.type.startsWith('image/')) continue
    if (file.size > 8 * 1024 * 1024) { showToast(`图片 ${file.name} 超过 8MB，已跳过`); continue }
    const reader = new FileReader()
    reader.onload = () => createImages.value.push(reader.result as string)
    reader.readAsDataURL(file)
  }
}
async function submitCreate() {
  if (!createTitle.value.trim() || !createContent.value.trim()) return showToast('请填写标题和内容')
  createSaving.value = true
  const res = await adminApi('create_feedback', {
    feedback_type: createType.value,
    title: createTitle.value.trim(),
    content: createContent.value.trim(),
    images: createImages.value,
    notify_external: createNotify.value ? 1 : 0,
  })
  createSaving.value = false
  if (res.code === 200) { showToast('创建成功', 'success'); closeCreate(); loadList() } else { showToast(res.msg || '创建失败') }
}

// 统计
const statsVisible = ref(false)
const statsLoading = ref(false)
const statsList = ref<any[]>([])
const statsGrand = ref(0)
async function openStats() {
  statsVisible.value = true
  statsLoading.value = true
  const res = await adminApi<any>('feedback_admin_stats')
  statsLoading.value = false
  if (res.code === 200 && res.data) {
    statsList.value = res.data.list || []
    statsGrand.value = Number(res.data.grand_total ?? 0)
  } else { statsList.value = []; statsGrand.value = 0 }
}

// 图片查看器
const viewerVisible = ref(false)
const viewerList = ref<string[]>([])
const viewerIndex = ref(0)
function openViewer(imgs: string[], i: number) { viewerList.value = imgs; viewerIndex.value = i; viewerVisible.value = true }
function viewerPrev() { viewerIndex.value = (viewerIndex.value - 1 + viewerList.value.length) % viewerList.value.length }
function viewerNext() { viewerIndex.value = (viewerIndex.value + 1) % viewerList.value.length }

onMounted(() => { loadLimit(); loadList() })
</script>

<style scoped>
.mfb-ops { display: flex; align-items: center; justify-content: space-between; gap: 10px; }
.mfb-sort { flex: 0 0 150px; }
.mfb-op-btns { display: flex; gap: 8px; }
.mfb-title-wrap { display: flex; align-items: center; gap: 6px; min-width: 0; }
.mfb-type {
  flex: 0 0 auto;
  font-size: 10px;
  font-weight: 850;
  padding: 2px 7px;
  border-radius: 999px;
}
.mfb-type.type-suggestion { background: #eef2ff; color: #4f46e5; }
.mfb-type.type-problem { background: #fef2f2; color: #dc2626; }
.mfb-sub { margin-top: 4px; font-size: 11px; color: var(--text-muted); }
.mfb-content { margin-top: 6px; }
.mfb-main { display: flex; gap: 12px; align-items: flex-start; margin-top: 6px; }
.mfb-left { flex: 1; min-width: 0; }
.mfb-right { display: flex; flex-direction: column; gap: 6px; flex-shrink: 0; }
.mfb-thumb {
  width: 72px; height: 72px;
  object-fit: cover;
  border-radius: 12px;
  border: 1px solid var(--border);
}
.mfb-timeline {
  margin-top: 10px;
  padding-top: 10px;
  border-top: 1px dashed var(--border);
  display: flex;
  flex-wrap: wrap;
  gap: 4px 0;
  font-size: 11px;
  color: var(--text-muted);
  line-height: 1.6;
}
.mfb-line { font-size: 12px; color: var(--text-light); word-break: break-word; }
.mfb-resolve { color: #16a34a; }

/* 新建弹窗 */
.mfb-create-body { padding: 10px 20px 12px; overflow-y: auto; display: flex; flex-direction: column; gap: 10px; }
.mfb-type-row { display: flex; gap: 8px; }
.mfb-type-btn {
  flex: 1;
  border: 1px solid var(--border);
  border-radius: 999px;
  padding: 8px 0;
  background: var(--control-bg);
  color: var(--text-light);
  font-size: 13px;
  font-weight: 800;
}
.mfb-type-btn.active { border-color: #EC4141; background: #EC4141; color: #fff; }
.mfb-dropzone {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  border: 1.5px dashed var(--border);
  border-radius: 14px;
  text-align: center;
  cursor: pointer;
  transition: border-color 0.2s, background 0.2s;
}
.mfb-dropzone.dragging, .mfb-dropzone.has { border-color: #EC4141; background: rgba(236, 65, 65, 0.05); }
.mfb-dropzone-text { display: flex; flex-direction: column; gap: 2px; }
.mfb-dropzone-text strong { font-size: 13px; color: var(--text); }
.mfb-dropzone-text span { font-size: 11px; color: var(--text-muted); }
.mfb-preview { display: flex; flex-wrap: wrap; gap: 8px; }
.mfb-preview-item { position: relative; }
.mfb-preview-img { width: 72px; height: 72px; object-fit: cover; border-radius: 10px; border: 1px solid var(--border); }
.mfb-preview-remove {
  position: absolute; top: -6px; right: -6px;
  width: 20px; height: 20px;
  border: none; border-radius: 50%;
  background: #dc2626; color: #fff;
  font-size: 14px; line-height: 1;
  display: flex; align-items: center; justify-content: center;
  cursor: pointer;
}
.mfb-notify { display: flex; align-items: flex-start; gap: 10px; padding: 10px 12px; border-radius: 12px; background: var(--control-bg); cursor: pointer; }
.mfb-notify input { display: none; }
.mfb-notify-box {
  width: 20px; height: 20px;
  border-radius: 6px;
  border: 1.5px solid var(--border);
  background: var(--card);
  color: transparent;
  display: flex; align-items: center; justify-content: center;
  font-size: 13px; font-weight: 900;
  flex-shrink: 0; margin-top: 1px;
}
.mfb-notify-box.checked { background: #EC4141; border-color: #EC4141; color: #fff; }
.mfb-notify-text { display: flex; flex-direction: column; gap: 2px; }
.mfb-notify-text strong { font-size: 13px; color: var(--text); }
.mfb-notify-text span { font-size: 11px; color: var(--text-muted); line-height: 1.5; }

/* 统计弹窗 */
.mfb-stats-body { padding: 10px 20px 12px; overflow-y: auto; display: flex; flex-direction: column; gap: 10px; }
.mfb-stats-total { display: flex; flex-direction: column; align-items: center; gap: 3px; padding: 14px; border-radius: 14px; background: var(--control-bg); }
.mfb-stats-total strong { font-size: 30px; line-height: 1; }
.mfb-stats-total span { font-size: 12px; color: var(--text-muted); }
.mfb-stats-list { display: flex; flex-direction: column; gap: 8px; }
.mfb-stats-row { display: flex; flex-direction: column; gap: 6px; padding: 10px 12px; border: 1px solid var(--border); border-radius: 12px; }
.mfb-stats-admin { display: flex; align-items: center; gap: 8px; font-size: 13px; font-weight: 850; }
.mfb-dot { width: 10px; height: 10px; border-radius: 50%; }
.dot-unclaimed { background: #a1a1aa; }
.dot-0 { background: #3b82f6; }
.dot-1 { background: #16a34a; }
.dot-2 { background: #f59e0b; }
.dot-3 { background: #8b5cf6; }
.mfb-stats-nums { display: flex; flex-wrap: wrap; gap: 6px 12px; }
.mfb-stats-nums span { font-size: 12px; color: var(--text-muted); }
.mfb-stats-nums b { font-size: 13px; }
.c-processing { color: #3b82f6; }
.c-resolved { color: #16a34a; }
.c-rejected { color: #dc2626; }
.c-pending { color: #f59e0b; }

/* 图片查看器 */
.mfb-viewer { background: rgba(0, 0, 0, 0.9) !important; padding: 0; }
.mfb-viewer-img { max-width: 92vw; max-height: 88vh; object-fit: contain; border-radius: 8px; }
.mfb-viewer-close {
  position: absolute; top: 18px; right: 18px;
  width: 40px; height: 40px;
  border: none; border-radius: 50%;
  background: rgba(255, 255, 255, 0.15); color: #fff;
  font-size: 22px; line-height: 1;
  display: flex; align-items: center; justify-content: center;
  cursor: pointer;
}
.mfb-viewer-nav {
  position: absolute; top: 50%; transform: translateY(-50%);
  width: 42px; height: 42px;
  border: none; border-radius: 50%;
  background: rgba(255, 255, 255, 0.15); color: #fff;
  font-size: 26px; line-height: 1;
  display: flex; align-items: center; justify-content: center;
  cursor: pointer;
}
.mfb-viewer-nav.prev { left: 14px; }
.mfb-viewer-nav.next { right: 14px; }
.mfb-viewer-counter {
  position: absolute; bottom: 20px; left: 50%; transform: translateX(-50%);
  padding: 5px 14px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.16); color: #fff;
  font-size: 13px; font-weight: 800;
}
</style>