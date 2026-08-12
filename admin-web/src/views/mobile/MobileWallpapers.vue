<template>
  <div class="mobile-page">
    <section class="mobile-card mobile-form">
      <h3 class="mobile-card-title">上传壁纸</h3>
      <input v-model="form.title" class="mobile-input" placeholder="壁纸标题" />
      <input v-model="form.category" class="mobile-input" placeholder="分类，默认可留空" />
      <textarea v-model="form.description" class="mobile-textarea" placeholder="壁纸说明（选填）"></textarea>
      <label class="mobile-upload">
        <input type="file" accept="image/jpeg,image/png,image/webp" @change="onImageChange" />
        <span>{{ imageFile ? imageFile.name : '选择 JPG / PNG / WEBP 图片上传到服务器' }}</span>
      </label>
      <img v-if="previewUrl" :src="previewUrl" class="wallpaper-img preview" />
      <button class="mobile-btn primary" :disabled="uploading" @click="uploadWallpaper">
        {{ uploading ? '上传中...' : '上传壁纸' }}
      </button>
    </section>
    <section class="mobile-card mobile-form">
      <h3 class="mobile-card-title">上传限制</h3>
      <input v-model="globalLimit" class="mobile-input" type="number" placeholder="全局上传上限，0 表示不限" />
      <button class="mobile-btn primary" @click="saveGlobalLimit">保存全局上限</button>
      <input v-model="accountLimit.ciyuanxi_id" class="mobile-input" placeholder="指定弦予号" />
      <input v-model="accountLimit.upload_limit" class="mobile-input" type="number" placeholder="账号上传上限，0 表示不限" />
      <input v-model="accountLimit.remark" class="mobile-input" placeholder="备注" />
      <button class="mobile-btn" @click="saveAccountLimit">保存账号上限</button>
      <div v-for="item in accountLimits" :key="item.ciyuanxi_id" class="limit-row">
        <span>{{ item.ciyuanxi_id }} · {{ item.upload_limit }} · 已传 {{ item.uploaded_count || 0 }}</span>
        <button class="mobile-btn danger" @click="deleteAccountLimit(item)">删</button>
      </div>
    </section>
    <div v-if="loading" class="mobile-empty">加载中...</div>
    <div v-else-if="list.length === 0" class="mobile-empty">暂无壁纸</div>
    <div v-else class="mobile-list">
      <div v-for="w in list" :key="w.id" class="mobile-item">
        <img v-if="w.image_url || w.url" :src="w.image_url || w.url" class="wallpaper-img" />
        <div class="mobile-item-head"><div class="mobile-item-title">{{ w.title || '壁纸 #' + w.id }}</div><span class="mobile-badge">{{ w.status || '-' }}</span></div>
        <div class="mobile-item-sub">{{ w.username || w.author || '-' }} · {{ w.created_at || '-' }}</div>
        <div class="mobile-actions">
          <button class="mobile-btn primary" @click="change(w, 'normal')">通过</button>
          <button class="mobile-btn" @click="change(w, 'rejected')">拒绝</button>
          <button class="mobile-btn" @click="change(w, 'disabled')">禁用</button>
          <button class="mobile-btn danger" @click="remove(w)">删除</button>
        </div>
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { onMounted, ref, onUnmounted } from 'vue'
import { adminApi, showToast } from '@/api/client'
import './MobilePage.css'
import { mobileConfirm } from '@/utils/mobileDialog'
const loading = ref(false), uploading = ref(false), list = ref<any[]>([])
const imageFile = ref<File | null>(null)
const previewUrl = ref('')
const form = ref({ title: '', description: '', category: '' })
const globalLimit = ref(20)
const accountLimit = ref({ ciyuanxi_id: '', upload_limit: 20, remark: '' })
const accountLimits = ref<any[]>([])
async function load(silent = false) { if (!silent) loading.value = true; const res = await adminApi<any[]>('list_wallpapers'); list.value = res.code === 200 && res.data ? res.data : []; if (!silent) loading.value = false }
async function loadLimits() {
  const [g, a] = await Promise.all([adminApi<any>('get_wallpaper_upload_limit'), adminApi<any[]>('list_wallpaper_account_limits')])
  if (g.code === 200 && g.data) globalLimit.value = Number(g.data.wallpaper_upload_limit ?? 20)
  accountLimits.value = a.code === 200 && a.data ? a.data : []
}
async function saveGlobalLimit() {
  const res = await adminApi('update_wallpaper_upload_limit', { wallpaper_upload_limit: Number(globalLimit.value) })
  if (res.code === 200) showToast('全局上限已保存', 'success'); else showToast(res.msg || '保存失败')
}
async function saveAccountLimit() {
  if (!accountLimit.value.ciyuanxi_id.trim()) return showToast('请填写弦予号')
  const res = await adminApi('save_wallpaper_account_limit', {
    ciyuanxi_id: accountLimit.value.ciyuanxi_id.trim(),
    upload_limit: Number(accountLimit.value.upload_limit),
    remark: accountLimit.value.remark.trim(),
  })
  if (res.code === 200) { showToast('账号上限已保存', 'success'); accountLimit.value = { ciyuanxi_id: '', upload_limit: 20, remark: '' }; loadLimits() } else showToast(res.msg || '保存失败')
}
async function deleteAccountLimit(item: any) {
  const res = await adminApi('delete_wallpaper_account_limit', { ciyuanxi_id: item.ciyuanxi_id })
  if (res.code === 200) { showToast('已删除限制', 'success'); loadLimits() } else showToast(res.msg || '删除失败')
}
function onImageChange(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0] || null
  if (!file) return
  if (!['image/jpeg', 'image/png', 'image/webp'].includes(file.type)) {
    input.value = ''
    return showToast('只允许上传 JPG / PNG / WEBP 图片')
  }
  imageFile.value = file
  if (previewUrl.value) URL.revokeObjectURL(previewUrl.value)
  previewUrl.value = URL.createObjectURL(file)
}
function readFileAsBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => resolve(String(reader.result || '').split(',')[1] || '')
    reader.onerror = () => reject(new Error('图片读取失败'))
    reader.readAsDataURL(file)
  })
}
async function uploadWallpaper() {
  if (!form.value.title.trim()) return showToast('请填写壁纸标题')
  if (!imageFile.value) return showToast('请选择壁纸图片')
  uploading.value = true
  let image = ''
  try {
    image = await readFileAsBase64(imageFile.value)
  } catch {
    uploading.value = false
    return showToast('图片读取失败')
  }
  const res = await adminApi('add_wallpaper', {
    title: form.value.title.trim(),
    description: form.value.description.trim(),
    category: form.value.category.trim() || '默认',
    image,
  })
  uploading.value = false
  if (res.code === 200) {
    showToast('壁纸已上传', 'success')
    form.value = { title: '', description: '', category: '' }
    imageFile.value = null
    if (previewUrl.value) URL.revokeObjectURL(previewUrl.value)
    previewUrl.value = ''
    load()
  } else showToast(res.msg || '上传失败')
}
async function change(w: any, status: string) { const res = await adminApi('change_wallpaper_status', { id: w.id, status }); if (res.code === 200) { w.status = status; showToast('已处理', 'success') } else showToast(res.msg || '操作失败') }
async function remove(w: any) { if (!(await mobileConfirm('确认删除壁纸？'))) return; const res = await adminApi('delete_wallpaper', { id: w.id }); if (res.code === 200) { showToast('已删除', 'success'); load() } else showToast(res.msg || '删除失败') }
onMounted(() => { load(); loadLimits(); startPolling() })
onUnmounted(() => stopPolling())
let pollTimer: ReturnType<typeof setInterval> | null = null
function startPolling() { stopPolling(); pollTimer = setInterval(() => load(true), 30000) }
function stopPolling() { if (pollTimer) { clearInterval(pollTimer); pollTimer = null } }
</script>
<style scoped>
.wallpaper-img{width:100%;max-height:220px;object-fit:cover;border-radius:14px;margin-bottom:10px;background:var(--control-bg)}
.wallpaper-img.preview{max-height:180px;margin:0}
.mobile-upload {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 96px;
  border: 1.5px dashed var(--border);
  border-radius: 16px;
  background: var(--control-bg);
  color: var(--text-muted);
  padding: 12px;
  text-align: center;
  font-size: 13px;
  font-weight: 800;
}
.mobile-upload input {
  display: none;
}
.limit-row{display:flex;justify-content:space-between;align-items:center;gap:8px;padding:9px 0;border-top:1px solid var(--border);font-size:12px;word-break:break-all}
</style>
