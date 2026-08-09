<template>
  <div class="wp-page">
    <!-- 页面头部 -->
    <Transition name="fade-down" appear>
      <div class="page-header">
        <div class="header-info">
          <h2 class="page-title">
            壁纸管理
            <span v-if="pendingCount > 0" class="pending-badge">{{ pendingCount }} 项待审核</span>
          </h2>
          <p class="page-desc">
            管理员上传的壁纸直接启用；用户在桌面端上传的壁纸状态为「待审核」，需审核通过后才会展示给所有用户。
          </p>
        </div>
        <button class="btn-add" @click="openAddModal">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
          </svg>
          新增壁纸
        </button>
      </div>
    </Transition>

    <!-- 上传限制配置 -->
    <Transition name="fade-up" appear>
      <div class="limit-card">
        <div>
          <div class="limit-title">用户上传上限</div>
          <div class="limit-desc">
            每个桌面端用户最多可上传的壁纸数量；填 0 表示不限制。
            <span v-if="wallpaperUploadLimit === 0">当前：无限制</span>
            <span v-else>当前：{{ wallpaperUploadLimit }} 张</span>
          </div>
        </div>
        <div class="limit-controls">
          <input
            v-model.number="wallpaperUploadLimitInput"
            type="number"
            min="0"
            max="10000"
            class="limit-input"
            :disabled="limitLoading || limitSaving"
          />
          <button class="btn-limit-save" :disabled="limitLoading || limitSaving" @click="saveWallpaperUploadLimit">
            {{ limitSaving ? '保存中...' : '保存限制' }}
          </button>
        </div>
      </div>
    </Transition>

    <!-- 账号独立上传限制 -->
    <Transition name="fade-up" appear>
      <div class="account-limit-card">
        <div class="account-limit-head">
          <div>
            <div class="limit-title">账号独立上限</div>
            <div class="limit-desc">指定某个弦予号使用独立上传上限；填 0 表示该账号无限制，未配置账号继续使用全局上限。</div>
          </div>
          <button class="btn-limit-refresh" :disabled="accountLimitLoading" @click="loadWallpaperAccountLimits">刷新</button>
        </div>
        <div class="account-limit-form">
          <input v-model.trim="accountLimitForm.ciyuanxi_id" class="limit-input account-id-input" placeholder="弦予号，例如 XY123456" />
          <input v-model.number="accountLimitForm.upload_limit" type="number" min="0" max="10000" class="limit-input" placeholder="上限" />
          <input v-model.trim="accountLimitForm.remark" class="limit-input account-remark-input" placeholder="备注（可选）" />
          <button class="btn-limit-save" :disabled="accountLimitSaving" @click="saveWallpaperAccountLimit">
            {{ accountLimitSaving ? '保存中...' : '保存账号限制' }}
          </button>
        </div>
        <div v-if="accountLimits.length === 0" class="account-limit-empty">
          暂无账号独立限制
        </div>
        <div v-else class="account-limit-list">
          <div v-for="item in accountLimits" :key="item.ciyuanxi_id" class="account-limit-item">
            <div class="account-limit-main">
              <div class="account-id">{{ item.ciyuanxi_id }}</div>
              <div class="account-meta">
                {{ item.username || '未命名账号' }} · 已上传 {{ Number(item.uploaded_count || 0) }} 张 ·
                <template v-if="Number(item.upload_limit) === 0">无限制</template>
                <template v-else>最多 {{ item.upload_limit }} 张</template>
              </div>
              <div v-if="item.remark" class="account-remark">{{ item.remark }}</div>
            </div>
            <div class="account-limit-actions">
              <button class="text-btn" @click="fillAccountLimitForm(item)">编辑</button>
              <button class="text-btn danger" @click="deleteWallpaperAccountLimit(item.ciyuanxi_id)">恢复默认</button>
            </div>
          </div>
        </div>
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
            <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>
            <circle cx="8.5" cy="8.5" r="1.5"/>
            <polyline points="21 15 16 10 5 21"/>
          </svg>
        </div>
        <p class="empty-title">{{ activeFilter === 'all' ? '暂无壁纸' : '该分类下暂无壁纸' }}</p>
        <p class="empty-sub">点击右上角「新增壁纸」上传第一张壁纸</p>
      </div>
    </Transition>

    <!-- 壁纸画廊 -->
    <div v-else class="gallery-grid">
      <TransitionGroup name="card">
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
              v-if="item.thumbnail_url"
              :src="item.thumbnail_url"
              :alt="item.title"
              class="thumb-img"
              loading="lazy"
              @error="onImgError"
            />
            <div v-else class="thumb-placeholder">
              <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>
            </div>

            <!-- hover 遮罩 -->
            <div class="thumb-overlay">
              <a v-if="item.image_url" :href="item.image_url" target="_blank" class="overlay-btn" title="查看原图">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 3h6v6"/><path d="M10 14L21 3"/><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/></svg>
              </a>
            </div>

            <!-- 状态徽章 -->
            <span class="status-tag" :class="`tag-${item.status}`">{{ statusLabel(item.status) }}</span>
          </div>

          <!-- 信息区 -->
          <div class="card-info">
            <div class="info-top">
              <h3 class="wp-title">{{ item.title }}</h3>
              <span class="wp-category">{{ item.category || '默认' }}</span>
            </div>
            <p v-if="item.description" class="wp-desc">{{ item.description }}</p>

            <div class="info-meta">
              <span class="meta-uploader">
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
                {{ uploaderText(item) }}
              </span>
              <span class="meta-date">{{ item.created_at || '-' }}</span>
            </div>

            <!-- 审核信息 -->
            <div v-if="item.reviewed_by && item.reviewed_at" class="review-info">
              审核人：{{ item.reviewed_by }} · {{ item.reviewed_at }}
            </div>
            <div v-else-if="item.status === 'pending'" class="review-info review-pending">
              等待审核
            </div>

            <!-- 操作按钮 -->
            <div class="card-actions">
              <template v-if="item.status === 'pending'">
                <button class="act-btn act-approve" @click="changeStatus(item.id, 'normal')">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12"/></svg>
                  通过
                </button>
                <button class="act-btn act-reject" @click="changeStatus(item.id, 'rejected')">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
                  拒绝
                </button>
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
      </TransitionGroup>
    </div>

    <!-- 新增壁纸弹窗 -->
    <Transition name="modal">
      <div v-if="addModalVisible" class="modal-backdrop" @click.self="closeAddModal">
        <div class="modal-dialog">
          <div class="modal-head">
            <h3>新增壁纸</h3>
            <button class="modal-close" @click="closeAddModal">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
          <div class="modal-form">
            <div class="field">
              <label class="required">标题</label>
              <input v-model="addForm.title" type="text" placeholder="请输入壁纸标题" />
            </div>
            <div class="field">
              <label>描述 <span class="field-optional">（可选）</span></label>
              <textarea v-model="addForm.description" rows="2" placeholder="简短描述"></textarea>
            </div>
            <div class="field">
              <label>分类</label>
              <input v-model="addForm.category" type="text" placeholder="默认" />
            </div>
            <div class="field">
              <label><span class="required">图片</span> <span class="field-optional">（JPG/PNG/WEBP）</span></label>
              <div
                class="upload-zone"
                :class="{ dragging: isDragging, 'has-preview': !!imagePreview }"
                @click="triggerFileInput"
                @dragover.prevent="isDragging = true"
                @dragleave.prevent="isDragging = false"
                @drop.prevent="onDrop"
              >
                <img v-if="imagePreview" :src="imagePreview" class="upload-preview" />
                <div v-else class="upload-placeholder">
                  <svg width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                    <polyline points="17 8 12 3 7 8"/>
                    <line x1="12" y1="3" x2="12" y2="15"/>
                  </svg>
                  <span>点击或拖拽图片到此处</span>
                </div>
                <input ref="fileInputRef" type="file" accept=".jpg,.jpeg,.png,.webp" class="file-hidden" @change="onFileChange" />
              </div>
              <div v-if="addForm.fileName" class="file-name">
                {{ addForm.fileName }}（{{ formatFileSize(addForm.fileSize) }}）
              </div>
            </div>
            <div v-if="uploadProgress" class="upload-status">
              <div class="upload-bar-track">
                <div class="upload-bar-fill" :style="{ width: uploadProgress + '%' }"></div>
              </div>
              <span class="upload-pct">{{ uploadProgress }}%</span>
            </div>
          </div>
          <div class="modal-foot">
            <button class="btn-cancel" @click="closeAddModal">取消</button>
            <button class="btn-save" :disabled="uploading" @click="doAddWallpaper">
              <span v-if="uploading" class="btn-spinner"></span>
              {{ uploading ? '上传中...' : '上传' }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { adminApi, showToast } from '@/api/client'

interface Wallpaper {
  id: number
  title: string
  description: string
  image_url: string
  thumbnail_url: string
  category: string
  sort_order: number
  status: string
  uploaded_by: string
  uploaded_by_nickname: string
  reviewed_at: string | null
  reviewed_by: string
  created_at: string
  [key: string]: any
}

interface WallpaperUploadLimit {
  wallpaper_upload_limit: number
}

interface WallpaperAccountLimit {
  ciyuanxi_id: string
  upload_limit: number
  remark: string
  updated_by: string
  updated_at: string
  username: string
  email: string
  uploaded_count: number
}

const filters = [
  { value: 'all', label: '全部' },
  { value: 'pending', label: '待审核' },
  { value: 'normal', label: '已启用' },
  { value: 'rejected', label: '已拒绝' },
  { value: 'disabled', label: '已禁用' },
]

const statusMap: Record<string, { label: string; cls: string }> = {
  normal: { label: '已启用', cls: 'tag-normal' },
  disabled: { label: '已禁用', cls: 'tag-disabled' },
  pending: { label: '待审核', cls: 'tag-pending' },
  rejected: { label: '已拒绝', cls: 'tag-rejected' },
}

function statusLabel(s: string): string {
  return statusMap[s]?.label || s
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
const limitLoading = ref(false)
const limitSaving = ref(false)
const accountLimits = ref<WallpaperAccountLimit[]>([])
const accountLimitLoading = ref(false)
const accountLimitSaving = ref(false)
const accountLimitForm = ref({
  ciyuanxi_id: '',
  upload_limit: 20,
  remark: '',
})

const pendingCount = computed(() => wallpapers.value.filter(w => w.status === 'pending').length)
const filteredList = computed(() => {
  if (activeFilter.value === 'all') return wallpapers.value
  return wallpapers.value.filter(w => w.status === activeFilter.value)
})

async function loadList() {
  loading.value = true
  const res = await adminApi<Wallpaper[]>('list_wallpapers')
  if (res.code === 200 && res.data) {
    wallpapers.value = Array.isArray(res.data) ? res.data : []
  } else {
    wallpapers.value = []
  }
  loading.value = false
}

async function loadWallpaperUploadLimit() {
  limitLoading.value = true
  const res = await adminApi<WallpaperUploadLimit>('get_wallpaper_upload_limit')
  if (res.code === 200 && res.data) {
    const limit = Number(res.data.wallpaper_upload_limit ?? 20)
    wallpaperUploadLimit.value = Number.isFinite(limit) ? limit : 20
    wallpaperUploadLimitInput.value = wallpaperUploadLimit.value
  } else {
    showToast(res.msg || '壁纸上传上限加载失败')
  }
  limitLoading.value = false
}

async function saveWallpaperUploadLimit() {
  const limit = Number(wallpaperUploadLimitInput.value)
  if (!Number.isInteger(limit) || limit < 0 || limit > 10000) {
    showToast('上传上限需为 0 到 10000 的整数')
    return
  }
  limitSaving.value = true
  const res = await adminApi<WallpaperUploadLimit>('update_wallpaper_upload_limit', {
    wallpaper_upload_limit: limit,
  })
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
  if (res.code === 200 && Array.isArray(res.data)) {
    accountLimits.value = res.data
  } else {
    accountLimits.value = []
    showToast(res.msg || '账号独立上限加载失败')
  }
  accountLimitLoading.value = false
}

async function saveWallpaperAccountLimit() {
  const ciyuanxiId = accountLimitForm.value.ciyuanxi_id.trim()
  const limit = Number(accountLimitForm.value.upload_limit)
  if (!ciyuanxiId) {
    showToast('请填写弦予号')
    return
  }
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
}

async function deleteWallpaperAccountLimit(ciyuanxiId: string) {
  if (!confirm(`确定恢复 ${ciyuanxiId} 使用全局上传上限吗？`)) return
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
  if (tips[status] && !confirm(tips[status])) return
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
  if (!confirm('确定要删除此壁纸吗？图片文件也会一并删除。')) return
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
const isDragging = ref(false)
const imagePreview = ref('')
const fileInputRef = ref<HTMLInputElement | null>(null)
const addForm = ref({
  title: '',
  description: '',
  category: '默认',
  fileName: '',
  fileSize: 0,
  fileBase64: '',
})

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
  handleFile(input.files[0])
}

function onDrop(e: DragEvent) {
  isDragging.value = false
  if (!e.dataTransfer?.files || e.dataTransfer.files.length === 0) return
  handleFile(e.dataTransfer.files[0])
}

function handleFile(file: File) {
  const validTypes = ['image/jpeg', 'image/png', 'image/webp']
  if (!validTypes.includes(file.type)) {
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
  if (!addForm.value.title.trim()) {
    showToast('请填写壁纸标题')
    return
  }
  if (!addForm.value.fileBase64) {
    showToast('请选择图片')
    return
  }

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
})
</script>

<style scoped>
.wp-page {
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
}
.page-desc {
  font-size: 13px;
  color: var(--text-muted);
  line-height: 1.6;
  margin: 0;
  max-width: 620px;
}

.btn-add {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 10px 20px;
  border-radius: 10px;
  border: none;
  background: var(--accent);
  color: var(--white);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.12);
}
.btn-add:hover { transform: translateY(-2px); box-shadow: 0 6px 20px rgba(0, 0, 0, 0.2); }
.btn-add:active { transform: scale(0.96); }

.limit-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 18px;
  padding: 14px 16px;
  border: 1px solid var(--border);
  border-radius: 14px;
  background: var(--white);
  box-shadow: 0 2px 10px rgba(15, 23, 42, 0.04);
}
.limit-title {
  font-size: 14px;
  font-weight: 700;
  color: var(--text);
}
.limit-desc {
  margin-top: 4px;
  font-size: 12px;
  color: var(--text-muted);
}
.limit-controls {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}
.limit-input {
  width: 110px;
  padding: 9px 10px;
  border: 1px solid var(--border);
  border-radius: 10px;
  font-size: 13px;
  outline: none;
}
.limit-input:focus {
  border-color: var(--accent);
}
.btn-limit-save {
  padding: 9px 14px;
  border: none;
  border-radius: 10px;
  background: var(--accent);
  color: var(--white);
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
}
.btn-limit-save:disabled,
.limit-input:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}
.account-limit-card {
  margin-bottom: 18px;
  padding: 14px 16px;
  border: 1px solid var(--border);
  border-radius: 14px;
  background: var(--white);
  box-shadow: 0 2px 10px rgba(15, 23, 42, 0.04);
}
.account-limit-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 12px;
}
.account-limit-form {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  margin-bottom: 12px;
}
.account-id-input {
  width: 180px;
}
.account-remark-input {
  width: 240px;
}
.btn-limit-refresh {
  padding: 8px 12px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--white);
  color: var(--text-light);
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
}
.btn-limit-refresh:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}
.account-limit-empty {
  padding: 12px;
  border-radius: 10px;
  background: rgba(148, 163, 184, 0.08);
  color: var(--text-muted);
  font-size: 12px;
}
.account-limit-list {
  display: grid;
  gap: 8px;
}
.account-limit-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border: 1px solid var(--border);
  border-radius: 12px;
  background: rgba(248, 250, 252, 0.7);
}
.account-limit-main {
  min-width: 0;
}
.account-id {
  font-size: 13px;
  font-weight: 700;
  color: var(--text);
}
.account-meta,
.account-remark {
  margin-top: 3px;
  font-size: 12px;
  color: var(--text-muted);
}
.account-limit-actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}
.text-btn {
  border: none;
  background: transparent;
  color: var(--accent);
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
}
.text-btn.danger {
  color: #ef4444;
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
  background: var(--white);
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

.wp-card {
  background: var(--white);
  border-radius: 14px;
  border: 1px solid var(--border);
  overflow: hidden;
  transition: transform 0.3s cubic-bezier(0.16, 1, 0.3, 1), box-shadow 0.3s ease, border-color 0.2s;
  animation: cardEnter 0.5s cubic-bezier(0.16, 1, 0.3, 1) backwards;
  display: flex;
  flex-direction: column;
}
.wp-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.08);
  border-color: transparent;
}
@keyframes cardEnter {
  from { opacity: 0; transform: translateY(20px); }
  to { opacity: 1; transform: translateY(0); }
}

/* 缩略图 */
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
.wp-card:hover .thumb-img { transform: scale(1.06); }
.thumb-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #d1d5db;
}

/* hover 遮罩 */
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
.wp-card:hover .thumb-overlay { opacity: 1; }
.overlay-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.9);
  color: var(--text);
  text-decoration: none;
  backdrop-filter: blur(4px);
  transition: background 0.2s;
}
.overlay-btn:hover { background: var(--white); }

/* 状态标签 */
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

/* 信息区 */
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
.wp-title {
  font-size: 14px;
  font-weight: 700;
  margin: 0;
  color: var(--text);
  line-height: 1.4;
}
.wp-category {
  font-size: 11px;
  color: var(--text-muted);
  background: #f5f5f5;
  padding: 2px 8px;
  border-radius: 10px;
  white-space: nowrap;
  flex-shrink: 0;
}
.wp-desc {
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
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 2px;
}
.meta-uploader {
  display: flex;
  align-items: center;
  gap: 4px;
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

/* 操作按钮 */
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
.act-approve { background: #ecfdf5; color: #10b981; }
.act-approve:hover { background: #d1fae5; }
.act-reject { background: #fef2f2; color: #ef4444; }
.act-reject:hover { background: #fee2e2; }
.act-disable { background: #fffbeb; color: #f59e0b; }
.act-disable:hover { background: #fef3c7; }
.act-delete { background: #f5f5f5; color: #888; }
.act-delete:hover { background: #fef2f2; color: #ef4444; }

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
  background: var(--white);
  border-radius: 18px;
  width: 100%;
  max-width: 480px;
  max-height: 90vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.16);
}
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

.modal-form {
  padding: 0 24px;
  overflow-y: auto;
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.field { display: flex; flex-direction: column; gap: 6px; }
.field label { font-size: 13px; font-weight: 600; color: var(--text-light); }
.field-optional { font-weight: 400; color: var(--text-muted); font-size: 12px; }
.field input,
.field textarea {
  padding: 10px 14px;
  border: 1.5px solid var(--border);
  border-radius: 10px;
  font-size: 14px;
  outline: none;
  background: #fafafa;
  font-family: inherit;
  resize: vertical;
  transition: border-color 0.2s, background 0.2s;
}
.field input:focus, .field textarea:focus {
  border-color: var(--accent);
  background: var(--white);
}

/* 上传区域 */
.upload-zone {
  position: relative;
  border: 2px dashed var(--border);
  border-radius: 12px;
  min-height: 140px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s;
  overflow: hidden;
  background: #fafafa;
}
.upload-zone:hover { border-color: var(--text-muted); background: #f5f5f5; }
.upload-zone.dragging { border-color: var(--accent); background: #f0f0f0; }
.upload-zone.has-preview { border-style: solid; border-color: var(--border); }
.upload-preview {
  max-width: 100%;
  max-height: 200px;
  object-fit: contain;
  border-radius: 8px;
}
.upload-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  color: var(--text-muted);
  font-size: 13px;
}
.file-hidden { display: none; }
.file-name {
  font-size: 12px;
  color: var(--text-muted);
  margin-top: -4px;
}

.upload-status {
  display: flex;
  align-items: center;
  gap: 8px;
}
.upload-bar-track {
  flex: 1;
  height: 6px;
  background: var(--border);
  border-radius: 3px;
  overflow: hidden;
}
.upload-bar-fill {
  height: 100%;
  background: var(--accent);
  border-radius: 3px;
  transition: width 0.3s;
}
.upload-pct { font-size: 12px; color: var(--text-muted); min-width: 36px; text-align: right; }

/* 弹窗底部 */
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
  background: var(--white);
  font-size: 14px;
  font-weight: 500;
  color: var(--text-light);
  cursor: pointer;
  transition: all 0.2s;
}
.btn-cancel:hover { background: #f5f5f5; }
.btn-save {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 10px 24px;
  border-radius: 10px;
  border: none;
  background: var(--accent);
  color: var(--white);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}
.btn-save:hover:not(:disabled) { transform: translateY(-1px); box-shadow: 0 4px 12px rgba(0,0,0,0.15); }
.btn-save:disabled { opacity: 0.6; cursor: not-allowed; }
.btn-spinner {
  width: 14px; height: 14px;
  border: 2px solid rgba(255,255,255,0.3);
  border-top-color: var(--white);
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

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
