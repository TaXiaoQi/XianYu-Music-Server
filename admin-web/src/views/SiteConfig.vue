<template>
  <div class="site-config-page">
    <!-- 页面头部动效 -->
    <Transition name="fade-down" appear>
      <div class="page-header">
      <div>
        <h2 class="page-title">Logo 配置</h2>
        <p class="page-desc">上传站点 Logo，用于后台登录页、侧边栏及浏览器标签页图标，保存后即时生效。</p>
      </div>
    </div>
    </Transition>

    <Transition name="fade-up" appear>
      <div v-if="loading" class="state-box">
      <span class="loader"></span>
      加载中...
    </div>
      <div v-else class="config-card">
      <div class="section-title">当前 Logo</div>

      <div class="preview-row">
        <div class="preview-box">
          <img :src="previewUrl || siteLogoUrl" alt="站点 Logo" class="preview-img" />
        </div>
        <div class="preview-info">
          <p class="preview-label">浏览器标签页</p>
          <div class="favicon-chip">
            <img :src="previewUrl || siteLogoUrl" alt="favicon" class="favicon-img" />
          </div>
          <p class="preview-hint">建议使用 512×512 以内、透明底的 PNG 图片，正方形的效果最佳。</p>
        </div>
      </div>

      <div class="upload-row">
        <input ref="fileInput" type="file" accept="image/png,image/jpeg,image/webp,image/gif" class="file-input" @change="onFileChange" />
        <button class="btn-pick" @click="fileInput?.click()">选择图片</button>
        <span class="file-name">{{ fileName || '未选择图片' }}</span>
      </div>

      <div v-if="previewUrl" class="preview-new">
        <img :src="previewUrl" alt="新 Logo 预览" class="preview-new-img" />
        <span>新 Logo 预览</span>
      </div>

      <div class="actions">
        <button class="btn-save" :disabled="saving || !hasNewImage" @click="upload">
          <span v-if="saving" class="spinner"></span>
          {{ saving ? '上传中...' : '上传并保存' }}
        </button>
        <button v-if="hasNewImage" class="btn-cancel" :disabled="saving" @click="reset">取消</button>
      </div>
    </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'
import { loadSiteLogo, setSiteLogo, siteLogoUrl } from '@/utils/siteLogo'

const loading = ref(true)
const saving = ref(false)
const fileInput = ref<HTMLInputElement | null>(null)
const fileName = ref('')
const selectedImage = ref<string>('')

const previewUrl = computed(() => selectedImage.value)
const hasNewImage = computed(() => !!selectedImage.value)

function onFileChange(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  if (!/^image\/(png|jpeg|webp|gif)$/.test(file.type)) {
    showToast('只支持 PNG/JPG/WEBP/GIF 图片')
    return
  }
  fileName.value = file.name
  const reader = new FileReader()
  reader.onload = () => {
    selectedImage.value = String(reader.result || '')
  }
  reader.readAsDataURL(file)
}

function reset() {
  fileName.value = ''
  selectedImage.value = ''
  if (fileInput.value) fileInput.value.value = ''
}

async function upload() {
  if (!hasNewImage.value) return
  saving.value = true
  const res = await adminApi<{ logo_url: string }>('upload_site_logo', { image: selectedImage.value })
  saving.value = false
  if (res.code === 200 && res.data?.logo_url) {
    setSiteLogo(res.data.logo_url)
    showToast('站点 Logo 已更新', 'success')
    reset()
  } else {
    showToast(res.msg || '上传失败')
  }
}

onMounted(async () => {
  await loadSiteLogo()
  loading.value = false
})
</script>

<style scoped>
.site-config-page {
  max-width: 760px;
  margin: 0 auto;
}
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
  margin: 0 0 6px;
}
.page-desc {
  font-size: 13px;
  color: var(--text-muted);
  margin: 0;
  line-height: 1.6;
}
.btn-save {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  border: none;
  border-radius: 10px;
  background: var(--accent);
  color: var(--white);
  padding: 10px 20px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}
.btn-save:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 4px 14px rgba(0, 0, 0, 0.14);
}
.btn-save:disabled {
  opacity: 0.65;
  cursor: not-allowed;
}
.btn-cancel {
  border: 1px solid var(--border);
  background: transparent;
  color: var(--text-light);
  border-radius: 10px;
  padding: 10px 20px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}
.btn-cancel:hover:not(:disabled) {
  border-color: var(--text-light);
  color: var(--text);
}
.spinner,
.loader {
  width: 14px;
  height: 14px;
  border: 2px solid rgba(255, 255, 255, 0.35);
  border-top-color: var(--white);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}
.loader {
  width: 24px;
  height: 24px;
  border-color: var(--border);
  border-top-color: var(--accent);
}
.state-box {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  min-height: 220px;
  color: var(--text-muted);
}
.config-card {
  background: var(--card-solid);
  border: 1px solid var(--border);
  border-radius: 16px;
  padding: 24px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.04);
}
.section-title {
  font-size: 15px;
  font-weight: 800;
  color: var(--text);
  margin: 0 0 16px;
}
.preview-row {
  display: flex;
  align-items: center;
  gap: 20px;
  margin-bottom: 20px;
}
.preview-box {
  width: 120px;
  height: 120px;
  border-radius: 16px;
  border: 1px solid var(--border);
  background: var(--control-bg);
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  flex-shrink: 0;
}
.preview-img {
  width: 100%;
  height: 100%;
  object-fit: contain;
}
.preview-info {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.preview-label {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-light);
  margin: 0;
}
.favicon-chip {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  border-radius: 9px;
  border: 1px solid var(--border);
  background: var(--control-bg);
}
.favicon-img {
  width: 24px;
  height: 24px;
  object-fit: contain;
}
.preview-hint {
  font-size: 12px;
  color: var(--text-muted);
  margin: 0;
  max-width: 300px;
  line-height: 1.6;
}
.upload-row {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
}
.file-input {
  display: none;
}
.btn-pick {
  border: 1.5px solid var(--border);
  background: var(--control-bg);
  color: var(--text);
  border-radius: 10px;
  padding: 9px 18px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}
.btn-pick:hover {
  border-color: var(--accent);
  color: var(--accent);
  background: var(--card-solid);
}
.file-name {
  font-size: 13px;
  color: var(--text-muted);
}
.preview-new {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 20px;
  padding: 12px;
  border: 1px dashed var(--border);
  border-radius: 12px;
  background: var(--control-bg);
}
.preview-new-img {
  width: 48px;
  height: 48px;
  border-radius: 10px;
  object-fit: contain;
  background: #fff;
}
.preview-new span {
  font-size: 13px;
  color: var(--text-light);
  font-weight: 600;
}
.actions {
  display: flex;
  gap: 12px;
}
@keyframes spin {
  to { transform: rotate(360deg); }
}
@media (max-width: 600px) {
  .preview-row {
    flex-direction: column;
    align-items: flex-start;
  }
}

/* ===== 过渡动画 ===== */
.fade-down-enter-active, .fade-down-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-down-enter-from { opacity: 0; transform: translateY(-12px); }
.fade-up-enter-active, .fade-up-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-up-enter-from { opacity: 0; transform: translateY(12px); }
.config-card { animation: cardIn 0.4s cubic-bezier(0.16, 1, 0.3, 1) both; }
@keyframes cardIn { from { opacity: 0; transform: translateY(8px); } to { opacity: 1; transform: translateY(0); } }
</style>