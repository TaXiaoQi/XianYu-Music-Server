<template>
  <div class="mobile-page">
    <!-- 页头 -->
    <div class="wp-header">
      <div class="wp-header-info">
        <h2 class="wp-title">
          壁纸管理
          <span v-if="pendingCount > 0" class="pending-badge">{{ pendingCount }} 项待审核</span>
        </h2>
        <p class="wp-desc">管理员上传的壁纸直接启用；用户在桌面端上传的壁纸状态为「待审核」，需审核通过后才会展示给所有用户。</p>
      </div>
      <button class="mobile-btn primary wp-add-btn" @click="openAddModal">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.8" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
        新增壁纸
      </button>
    </div>

    <!-- 上传限制配置 -->
    <section class="mobile-card">
      <div class="limit-head">
        <div>
          <h3 class="limit-title">用户上传上限</h3>
          <p class="limit-desc">每个桌面端用户最多可上传的壁纸数量；填 0 表示不限制。</p>
        </div>
        <span class="limit-current">{{ wallpaperUploadLimit === 0 ? '无限制' : wallpaperUploadLimit + ' 张' }}</span>
      </div>
      <div class="limit-row">
        <input v-model.number="wallpaperUploadLimitInput" class="mobile-input" type="number" min="0" max="10000" :disabled="limitSaving" />
        <button class="mobile-btn primary" :disabled="limitSaving" @click="saveWallpaperUploadLimit">{{ limitSaving ? '保存中...' : '保存限制' }}</button>
      </div>
    </section>

    <!-- 账号独立上传限制 -->
    <section class="mobile-card">
      <div class="limit-head">
        <div>
          <h3 class="limit-title">账号独立上限</h3>
          <p class="limit-desc">指定某个弦予号使用独立上传上限；填 0 表示该账号无限制。</p>
        </div>
        <button class="mobile-btn" :disabled="accountLimitLoading" @click="loadWallpaperAccountLimits">刷新</button>
      </div>

      <div class="account-form">
        <input v-model.trim="accountLimitForm.ciyuanxi_id" class="mobile-input" placeholder="弦予号，例如 XY123456" />
        <div class="account-form-row">
          <input v-model.number="accountLimitForm.upload_limit" class="mobile-input" type="number" min="0" max="10000" placeholder="上限" />
          <input v-model.trim="accountLimitForm.remark" class="mobile-input" placeholder="备注（可选）" />
        </div>
        <button class="mobile-btn primary" :disabled="accountLimitSaving" @click="saveWallpaperAccountLimit">{{ accountLimitSaving ? '保存中...' : '保存账号限制' }}</button>
      </div>

      <div v-if="accountLimits.length === 0" class="account-empty">暂无账号独立限制</div>
      <div v-else class="account-list">
        <div v-for="item in accountLimits" :key="item.ciyuanxi_id" class="account-item">
          <div class="account-item-main">
            <div class="account-id">{{ item.ciyuanxi_id }}</div>
            <div class="account-meta">
              {{ item.username || '未命名账号' }} · 已上传 {{ Number(item.uploaded_count || 0) }} 张 ·
              <template v-if="Number(item.upload_limit) === 0">无限制</template>
              <template v-else>最多 {{ item.upload_limit }} 张</template>
            </div>
            <div v-if="item.remark" class="account-remark">{{ item.remark }}</div>
          </div>
          <div class="account-actions">
            <button class="text-btn" @click="fillAccountLimitForm(item)">编辑</button>
            <button class="text-btn danger" @click="deleteWallpaperAccountLimit(item.ciyuanxi_id)">恢复默认</button>
          </div>
        </div>
      </div>
    </section>

    <!-- 加载中 -->
    <div v-if="loading" class="mobile-empty">加载中...</div>

    <!-- 状态筛选 -->
    <div v-else class="wp-tabs">
      <button
        v-for="f in filters"
        :key="f.value"
        class="wp-tab"
        :class="{ active: activeFilter === f.value }"
        @click="activeFilter = f.value"
      >
        {{ f.label }}
        <span v-if="countByStatus(f.value) > 0" class="tab-count">{{ countByStatus(f.value) }}</span>
      </button>
    </div>

    <!-- 空状态 -->
    <div v-if="!loading && filteredList.length === 0" class="mobile-empty">
      <div class="empty-title">{{ activeFilter === 'all' ? '暂无壁纸' : '该分类下暂无壁纸' }}</div>
      <div class="empty-sub">点击上方「新增壁纸」上传第一张壁纸</div>
    </div>

    <!-- 壁纸画廊 -->
    <div v-if="!loading && filteredList.length > 0" class="wp-gallery">
      <div
        v-for="(item, idx) in filteredList"
        :key="item.id"
        class="wp-card"
        :class="[`st-${item.status}`, { 'is-user': isUserUpload(item) }]"
        :style="{ animationDelay: `${idx * 50}ms` }"
      >
        <!-- 缩略图 -->
        <div class="thumb-wrap">
          <img
            v-if="item.thumbnail_url || item.image_url || item.url"
            :src="item.thumbnail_url || item.image_url || item.url"
            :alt="item.title"
            class="thumb-img"
            loading="lazy"
            @error="onImgError"
          />
          <div v-else class="thumb-placeholder">
            <svg width="30" height="30" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>
          </div>
          <span class="status-tag" :class="`tag-${item.status}`">{{ statusLabel(item.status) }}</span>
        </div>

        <!-- 信息区 -->
        <div class="card-info">
          <div class="info-top">
            <h3 class="card-title">{{ item.title }}</h3>
            <span class="card-category">{{ item.category || '默认' }}</span>
          </div>
          <p v-if="item.description" class="card-desc">{{ item.description }}</p>
          <div class="info-meta">
            <span class="meta-uploader">
              <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
              {{ uploaderText(item) }}
            </span>
            <span class="meta-date">{{ item.created_at || '-' }}</span>
          </div>
          <div v-if="item.reviewed_by && item.reviewed_at" class="review-info">审核人：{{ item.reviewed_by }} · {{ item.reviewed_at }}</div>
          <div v-else-if="item.status === 'pending'" class="review-info review-pending">等待审核</div>

          <!-- 操作按钮 -->
          <div class="card-actions">
            <template v-if="item.status === 'pending'">
              <button class="act-btn act-approve" @click="changeStatus(item.id, 'normal')">通过</button>
              <button class="act-btn act-reject" @click="changeStatus(item.id, 'rejected')">拒绝</button>
            </template>
            <template v-else-if="item.status === 'rejected'">
              <button class="act-btn act-approve" @click="changeStatus(item.id, 'normal')">通过</button>
              <button class="act-btn act-delete" @click="deleteWallpaper(item.id)">删除</button>
            </template>
            <template v-else-if="item.status === 'normal'">
              <button class="act-btn act-disable" @click="changeStatus(item.id, 'disabled')">禁用</button>
              <button class="act-btn act-delete" @click="deleteWallpaper(item.id)">删除</button>
            </template>
            <template v-else-if="item.status === 'disabled'">
              <button class="act-btn act-approve" @click="changeStatus(item.id, 'normal')">启用</button>
              <button class="act-btn act-delete" @click="deleteWallpaper(item.id)">删除</button>
            </template>
          </div>
        </div>
      </div>
    </div>

    <!-- 新增壁纸弹窗 -->
    <Transition name="modal">
      <div v-if="addModalVisible" class="modal-backdrop">
        <div class="modal-dialog">
          <div class="modal-head">
            <h3>新增壁纸</h3>
            <button class="modal-close" @click="closeAddModal">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
          <div class="modal-body">
            <label class="modal-field">
              <span class="required">标题</span>
              <input v-model="addForm.title" type="text" placeholder="请输入壁纸标题" />
            </label>
            <label class="modal-field">
              <span>描述 <i>（可选）</i></span>
              <textarea v-model="addForm.description" rows="2" placeholder="简短描述"></textarea>
            </label>
            <label class="modal-field">
              <span>分类</span>
              <input v-model="addForm.category" type="text" placeholder="默认" />
            </label>
            <div class="modal-field">
              <span class="required">图片 <i>（JPG/PNG/WEBP）</i></span>
              <div class="upload-zone" :class="{ 'has-preview': !!imagePreview }" @click="triggerFileInput">
                <img v-if="imagePreview" :src="imagePreview" class="upload-preview" />
                <div v-else class="upload-placeholder">
                  <svg width="30" height="30" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>
                  <span>点击选择图片</span>
                </div>
                <input ref="fileInputRef" type="file" accept=".jpg,.jpeg,.png,.webp" class="file-hidden" @change="onFileChange" />
              </div>
              <div v-if="addForm.fileName" class="file-name">{{ addForm.fileName }}（{{ formatFileSize(addForm.fileSize) }}）</div>
            </div>
            <div v-if="uploading" class="upload-status">
              <div class="upload-bar-track"><div class="upload-bar-fill" :style="{ width: uploadProgress + '%' }"></div></div>
              <span class="upload-pct">{{ uploadProgress }}%</span>
            </div>
            <p class="modal-tip">管理员上传的壁纸将直接启用并展示给所有用户。</p>
          </div>
          <div class="modal-foot">
            <button class="modal-btn cancel" @click="closeAddModal">取消</button>
            <button class="modal-btn save" :disabled="uploading" @click="doAddWallpaper">{{ uploading ? '上传中...' : '上传' }}</button>
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
import { mobileConfirm } from '@/utils/mobileDialog'

interface Wallpaper {
  id: number
  title: string
  description: string
  image_url: string
  thumbnail_url: string
  url?: string
  category: string
  status: string
  uploaded_by: string
  uploaded_by_nickname: string
  reviewed_at: string | null
  reviewed_by: string
  created_at: string
  [key: string]: any
}

interface WallpaperAccountLimit {
  ciyuanxi_id: string
  upload_limit: number
  remark: string
  username: string
  uploaded_count: number
}

const filters = [
  { value: 'all', label: '全部' },
  { value: 'pending', label: '待审核' },
  { value: 'normal', label: '已启用' },
  { value: 'rejected', label: '已拒绝' },
  { value: 'disabled', label: '已禁用' },
]

const statusMap: Record<string, string> = {
  normal: '已启用',
  disabled: '已禁用',
  pending: '待审核',
  rejected: '已拒绝',
}

function statusLabel(s: string): string {
  return statusMap[s] || s
}

function isUserUpload(item: Wallpaper): boolean {
  return !!item.uploaded_by && item.uploaded_by !== 'admin'
}

function uploaderText(item: Wallpaper): string {
  if (isUserUpload(item)) {
    const nick = (item.uploaded_by_nickname || '').trim()
    return nick ? `${nick}（${item.uploaded_by}）` : item.uploaded_by
  }
  return '管理员'
}

function countByStatus(status: string): number {
  if (status === 'all') return wallpapers.value.length
  return wallpapers.value.filter(w => w.status === status).length
}

// ===== 列表 =====
const wallpapers = ref<Wallpaper[]>([])
const loading = ref(true)
const activeFilter = ref('all')
const wallpaperUploadLimit = ref(20)
const wallpaperUploadLimitInput = ref(20)
const limitSaving = ref(false)
const accountLimits = ref<WallpaperAccountLimit[]>([])
const accountLimitLoading = ref(false)
const accountLimitSaving = ref(false)
const accountLimitForm = ref({ ciyuanxi_id: '', upload_limit: 20, remark: '' })

const pendingCount = computed(() => wallpapers.value.filter(w => w.status === 'pending').length)
const filteredList = computed(() => {
  if (activeFilter.value === 'all') return wallpapers.value
  return wallpapers.value.filter(w => w.status === activeFilter.value)
})

async function loadList(silent = false) {
  if (!silent) loading.value = true
  const res = await adminApi<Wallpaper[]>('list_wallpapers')
  wallpapers.value = res.code === 200 && res.data ? (Array.isArray(res.data) ? res.data : []) : []
  if (!silent) loading.value = false
}

async function loadWallpaperUploadLimit() {
  const res = await adminApi<{ wallpaper_upload_limit: number }>('get_wallpaper_upload_limit')
  if (res.code === 200 && res.data) {
    const limit = Number(res.data.wallpaper_upload_limit ?? 20)
    wallpaperUploadLimit.value = Number.isFinite(limit) ? limit : 20
    wallpaperUploadLimitInput.value = wallpaperUploadLimit.value
  } else {
    showToast(res.msg || '壁纸上传上限加载失败')
  }
}

async function saveWallpaperUploadLimit() {
  const limit = Number(wallpaperUploadLimitInput.value)
  if (!Number.isInteger(limit) || limit < 0 || limit > 10000) {
    showToast('上传上限需为 0 到 10000 的整数')
    return
  }
  limitSaving.value = true
  const res = await adminApi<{ wallpaper_upload_limit: number }>('update_wallpaper_upload_limit', { wallpaper_upload_limit: limit })
  limitSaving.value = false
  if (res.code === 200) {
    wallpaperUploadLimit.value = Number(res.data?.wallpaper_upload_limit ?? limit)
    wallpaperUploadLimitInput.value = wallpaperUploadLimit.value
    showToast('壁纸上传上限已保存', 'success')
  } else {
    showToast(res.msg || '保存失败')
  }
}

async function loadWallpaperAccountLimits() {
  accountLimitLoading.value = true
  const res = await adminApi<WallpaperAccountLimit[]>('list_wallpaper_account_limits')
  accountLimits.value = res.code === 200 && Array.isArray(res.data) ? res.data : []
  if (res.code !== 200) showToast(res.msg || '账号独立上限加载失败')
  accountLimitLoading.value = false
}

async function saveWallpaperAccountLimit() {
  const ciyuanxiId = accountLimitForm.value.ciyuanxi_id.trim()
  const limit = Number(accountLimitForm.value.upload_limit)
  if (!ciyuanxiId) { showToast('请填写弦予号'); return }
  if (!Number.isInteger(limit) || limit < 0 || limit > 10000) {
    showToast('账号上传上限需为 0 到 10000 的整数')
    return
  }
  accountLimitSaving.value = true
  const res = await adminApi('save_wallpaper_account_limit', {
    ciyuanxi_id: ciyuanxiId,
    upload_limit: limit,
    remark: accountLimitForm.value.remark.trim(),
  })
  accountLimitSaving.value = false
  if (res.code === 200) {
    showToast('账号上传上限已保存', 'success')
    accountLimitForm.value = { ciyuanxi_id: '', upload_limit: 20, remark: '' }
    loadWallpaperAccountLimits()
  } else {
    showToast(res.msg || '保存失败')
  }
}

function fillAccountLimitForm(item: WallpaperAccountLimit) {
  accountLimitForm.value = {
    ciyuanxi_id: item.ciyuanxi_id,
    upload_limit: Number(item.upload_limit || 0),
    remark: item.remark || '',
  }
  window.scrollTo({ top: 0, behavior: 'smooth' })
}

async function deleteWallpaperAccountLimit(ciyuanxiId: string) {
  const ok = await mobileConfirm(`确定恢复 ${ciyuanxiId} 使用全局上传上限吗？`, { title: '恢复全局上限', confirmText: '确认恢复' })
  if (!ok) return
  const res = await adminApi('delete_wallpaper_account_limit', { ciyuanxi_id: ciyuanxiId })
  if (res.code === 200) {
    showToast('已恢复全局默认', 'success')
    if (accountLimitForm.value.ciyuanxi_id === ciyuanxiId) {
      accountLimitForm.value = { ciyuanxi_id: '', upload_limit: 20, remark: '' }
    }
    loadWallpaperAccountLimits()
  } else {
    showToast(res.msg || '操作失败')
  }
}

function onImgError(e: Event) {
  const img = e.target as HTMLImageElement
  img.style.display = 'none'
}

// ===== 状态变更 =====
async function changeStatus(id: number, status: string) {
  const tips: Record<string, string> = {
    normal: '确定通过审核并启用此壁纸吗？',
    rejected: '确定拒绝此壁纸吗？',
    disabled: '确定禁用此壁纸吗？',
  }
  if (tips[status]) {
    const ok = await mobileConfirm(tips[status], { title: '壁纸状态变更', confirmText: '确认' })
    if (!ok) return
  }
  const res = await adminApi('change_wallpaper_status', { id, status })
  if (res.code === 200) {
    showToast('操作成功', 'success')
    loadList()
  } else {
    showToast(res.msg || '操作失败')
  }
}

// ===== 删除 =====
async function deleteWallpaper(id: number) {
  const ok = await mobileConfirm('确定要删除此壁纸吗？图片文件也会一并删除。', { title: '删除壁纸', confirmText: '确认删除', danger: true })
  if (!ok) return
  const res = await adminApi('delete_wallpaper', { id })
  if (res.code === 200) {
    showToast('删除成功', 'success')
    loadList()
  } else {
    showToast(res.msg || '删除失败')
  }
}

// ===== 新增弹窗 =====
const addModalVisible = ref(false)
const uploading = ref(false)
const uploadProgress = ref(0)
const imagePreview = ref('')
const fileInputRef = ref<HTMLInputElement | null>(null)
const addForm = ref({ title: '', description: '', category: '默认', fileName: '', fileSize: 0, fileBase64: '' })

function openAddModal() {
  addForm.value = { title: '', description: '', category: '默认', fileName: '', fileSize: 0, fileBase64: '' }
  imagePreview.value = ''
  uploadProgress.value = 0
  addModalVisible.value = true
}

function closeAddModal() {
  if (uploading.value) return
  addModalVisible.value = false
}

function triggerFileInput() {
  fileInputRef.value?.click()
}

function onFileChange(e: Event) {
  const input = e.target as HTMLInputElement
  if (!input.files || input.files.length === 0) return
  const file = input.files[0]
  if (!['image/jpeg', 'image/png', 'image/webp'].includes(file.type)) {
    showToast('只允许上传 JPG/PNG/WEBP 图片')
    return
  }
  addForm.value.fileName = file.name
  addForm.value.fileSize = file.size
  const reader = new FileReader()
  reader.onload = () => {
    const result = reader.result as string
    imagePreview.value = result
    addForm.value.fileBase64 = result.split(',')[1] || ''
  }
  reader.readAsDataURL(file)
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
}

async function doAddWallpaper() {
  if (!addForm.value.title.trim()) { showToast('请填写壁纸标题'); return }
  if (!addForm.value.fileBase64) { showToast('请选择图片'); return }
  uploading.value = true
  uploadProgress.value = 20
  const res = await adminApi('add_wallpaper', {
    title: addForm.value.title.trim(),
    description: addForm.value.description.trim(),
    category: addForm.value.category.trim() || '默认',
    image: addForm.value.fileBase64,
  })
  uploadProgress.value = 100
  setTimeout(() => { uploadProgress.value = 0 }, 400)
  uploading.value = false
  if (res.code === 200) {
    showToast('上传成功', 'success')
    closeAddModal()
    loadList()
  } else {
    showToast(res.msg || '上传失败')
  }
}

onMounted(() => {
  loadList()
  loadWallpaperUploadLimit()
  loadWallpaperAccountLimits()
  startPolling()
})
onUnmounted(() => stopPolling())

let pollTimer: ReturnType<typeof setInterval> | null = null
function startPolling() { stopPolling(); pollTimer = setInterval(() => loadList(true), 30000) }
function stopPolling() { if (pollTimer) { clearInterval(pollTimer); pollTimer = null } }
</script>
<style scoped>
.wp-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}
.wp-header-info { min-width: 0; flex: 1; }
.wp-title { font-size: 18px; font-weight: 850; margin: 0 0 4px; color: var(--text); display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
.pending-badge { font-size: 11px; font-weight: 700; padding: 3px 10px; border-radius: 20px; background: #fffbeb; color: #f59e0b; }
.wp-desc { font-size: 12px; color: var(--text-light); line-height: 1.6; margin: 0; }
.wp-add-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  flex-shrink: 0;
  padding: 9px 16px;
  white-space: nowrap;
}

/* 状态筛选 */
.wp-tabs {
  display: flex;
  gap: 8px;
  overflow-x: auto;
  padding-bottom: 2px;
}
.wp-tab {
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
.wp-tab.active {
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
.wp-tab.active .tab-count { background: rgba(255, 255, 255, 0.25); }

/* 上传限制 */
.limit-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
  margin-bottom: 12px;
}
.limit-title { font-size: 14px; font-weight: 750; margin: 0; color: var(--text); }
.limit-desc { font-size: 11px; color: var(--text-muted); line-height: 1.5; margin: 3px 0 0; }
.limit-current { flex-shrink: 0; font-size: 12px; font-weight: 800; color: var(--accent); background: var(--accent-soft); padding: 4px 10px; border-radius: 999px; }
.limit-row { display: flex; gap: 8px; }
.limit-row .mobile-input { flex: 1; }
.limit-row .mobile-btn { flex-shrink: 0; }

/* 账号独立上限 */
.account-form { display: flex; flex-direction: column; gap: 8px; margin-bottom: 12px; }
.account-form-row { display: flex; gap: 8px; }
.account-form-row .mobile-input:first-child { width: 40%; flex: none; }
.account-form-row .mobile-input:last-child { flex: 1; }
.account-empty { padding: 12px; border-radius: 12px; background: var(--control-bg); color: var(--text-muted); font-size: 12px; }
.account-list { display: flex; flex-direction: column; gap: 8px; }
.account-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border: 1px solid var(--border);
  border-radius: 12px;
  background: var(--control-bg);
}
.account-item-main { min-width: 0; }
.account-id { font-size: 13px; font-weight: 750; color: var(--text); }
.account-meta, .account-remark { margin-top: 3px; font-size: 11px; color: var(--text-muted); }
.account-actions { display: flex; gap: 10px; flex-shrink: 0; }
.text-btn { border: none; background: transparent; color: var(--accent); font-size: 12px; font-weight: 700; cursor: pointer; padding: 4px; }
.text-btn.danger { color: #ef4444; }

/* 画廊 */
.wp-gallery {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}
.wp-card {
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
.card-title { font-size: 13px; font-weight: 750; margin: 0; color: var(--text); line-height: 1.35; word-break: break-word; }
.card-category { font-size: 10px; color: var(--text-muted); background: var(--control-bg); padding: 2px 7px; border-radius: 999px; white-space: nowrap; flex-shrink: 0; }
.card-desc {
  font-size: 11px; color: var(--text-muted); margin: 0; line-height: 1.4;
  display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden;
}
.info-meta { display: flex; justify-content: space-between; align-items: center; font-size: 10px; color: var(--text-muted); }
.meta-uploader { display: flex; align-items: center; gap: 3px; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
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
.act-approve { background: rgba(16, 185, 129, 0.12); color: #10b981; }
.act-reject { background: rgba(239, 68, 68, 0.1); color: #ef4444; }
.act-disable { background: rgba(245, 158, 11, 0.12); color: #f59e0b; }
.act-delete { background: var(--control-bg); color: var(--text-muted); }

.empty-title { font-size: 14px; font-weight: 700; color: var(--text-light); margin-bottom: 4px; }
.empty-sub { font-size: 12px; color: var(--text-muted); }

/* 弹窗 */
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
  max-width: 340px;
  max-height: 88vh;
  border-radius: 22px;
  background: var(--card-solid, var(--card));
  box-shadow: 0 24px 60px rgba(15, 23, 42, 0.22);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
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
.modal-field { display: flex; flex-direction: column; gap: 6px; }
.modal-field span { font-size: 12px; font-weight: 700; color: var(--text-light); }
.modal-field span.required::after { content: '*'; color: #ef4444; margin-left: 2px; }
.modal-field span i { font-weight: 400; color: var(--text-muted); font-style: normal; font-size: 11px; }
.modal-field input, .modal-field textarea {
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 11px 12px;
  font-size: 14px;
  font-family: inherit;
  outline: none;
  background: var(--control-bg);
  color: var(--text);
  transition: border-color 0.18s, box-shadow 0.18s;
  width: 100%;
  box-sizing: border-box;
  resize: vertical;
}
.modal-field input:focus, .modal-field textarea:focus { border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-soft); }

.upload-zone {
  border: 1.5px dashed var(--border);
  border-radius: 14px;
  min-height: 120px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  overflow: hidden;
  background: var(--control-bg);
}
.upload-zone.has-preview { border-style: solid; border-color: var(--border); }
.upload-preview { max-width: 100%; max-height: 180px; object-fit: contain; }
.upload-placeholder { display: flex; flex-direction: column; align-items: center; gap: 8px; color: var(--text-muted); font-size: 12px; }
.file-hidden { display: none; }
.file-name { font-size: 11px; color: var(--text-muted); }

.upload-status { display: flex; align-items: center; gap: 8px; }
.upload-bar-track { flex: 1; height: 6px; background: var(--border); border-radius: 3px; overflow: hidden; }
.upload-bar-fill { height: 100%; background: var(--accent); border-radius: 3px; transition: width 0.3s; }
.upload-pct { font-size: 11px; color: var(--text-muted); min-width: 36px; text-align: right; }

.modal-tip { margin: 0; font-size: 11px; color: var(--text-muted); line-height: 1.5; }
.modal-foot { display: flex; gap: 10px; padding: 14px 20px 18px; border-top: 1px solid var(--border); }
.modal-btn { flex: 1; padding: 11px; border-radius: 12px; font-size: 14px; font-weight: 750; cursor: pointer; display: inline-flex; align-items: center; justify-content: center; transition: all 0.18s; }
.modal-btn.cancel { border: 1px solid var(--border); background: transparent; color: var(--text-muted); }
.modal-btn.cancel:active { background: var(--control-bg); }
.modal-btn.save { border: none; background: #EC4141; color: #fff; }
.modal-btn.save:disabled { opacity: 0.55; }

/* 过渡动画 */
.modal-enter-active, .modal-leave-active { transition: opacity 0.24s var(--motion, cubic-bezier(0.16, 1, 0.3, 1)); }
.modal-enter-from, .modal-leave-to { opacity: 0; }
.modal-enter-active .modal-dialog { animation: modalIn 0.24s cubic-bezier(0.16, 1, 0.3, 1) forwards; }
.modal-leave-active .modal-dialog { animation: modalOut 0.2s ease forwards; }
@keyframes modalIn { from { opacity: 0; transform: scale(0.94); } to { opacity: 1; transform: scale(1); } }
@keyframes modalOut { from { opacity: 1; transform: scale(1); } to { opacity: 0; transform: scale(0.96); } }
</style>