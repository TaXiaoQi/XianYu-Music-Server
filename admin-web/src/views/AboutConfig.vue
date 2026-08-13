<template>
  <div class="about-config-page">
    <Transition name="fade-down" appear>
      <div class="page-header">
      <div>
        <h2 class="page-title">关于页配置</h2>
        <p class="page-desc">配置桌面端关于页的官网、检查更新、项目地址等入口，保存后由客户端从后台下发。</p>
      </div>
      <button class="btn-save" :disabled="saving" @click="save">
        <span v-if="saving" class="spinner"></span>
        {{ saving ? '保存中...' : '保存配置' }}
        </button>
      </div>
    </Transition>

    <Transition name="fade-up" appear>
      <div v-if="loading" class="state-box">
        <span class="loader"></span>
        加载中...
      </div>
      <div v-else class="config-card">
      <div class="section-title">官网入口</div>
      <div class="field-grid">
        <label class="field">
          <span>按钮文字</span>
          <input v-model="form.officialSiteText" type="text" placeholder="前往官网" />
        </label>
        <label class="field">
          <span>官网链接</span>
          <input v-model="form.officialSiteUrl" type="text" placeholder="https://..." />
        </label>
      </div>

      <div class="section-title">加入群组入口</div>
      <div class="field-grid">
        <label class="field">
          <span>按钮文字</span>
          <input v-model="form.joinGroupText" type="text" placeholder="加入群组" />
        </label>
        <label class="field">
          <span>群组链接</span>
          <input v-model="form.joinGroupUrl" type="text" placeholder="https://..." />
        </label>
      </div>

      <div class="section-title">检查更新入口</div>
      <div class="field-grid">
        <label class="field">
          <span>按钮文字</span>
          <input v-model="form.updateText" type="text" placeholder="检查更新" />
        </label>
        <label class="switch-row">
          <input v-model="form.updateEnabled" type="checkbox" />
          <span>显示检查更新按钮</span>
        </label>
      </div>

      <div class="section-title">项目地址</div>
      <div class="field-grid">
        <label class="field">
          <span>按钮文字</span>
          <input v-model="form.projectText" type="text" placeholder="开源地址" />
        </label>
        <label class="field">
          <span>项目链接</span>
          <input v-model="form.projectUrl" type="text" placeholder="https://..." />
        </label>
      </div>

      <div class="section-title">参考项目</div>
      <div class="field-grid">
        <label class="field">
          <span>按钮文字</span>
          <input v-model="form.referenceProjectText" type="text" placeholder="参考项目" />
        </label>
        <label class="field">
          <span>参考项目链接</span>
          <input v-model="form.referenceProjectUrl" type="text" placeholder="https://..." />
        </label>
      </div>

      <p class="hint">链接留空后，桌面端会隐藏对应外链按钮；检查更新入口可通过开关隐藏。</p>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'

interface AboutConfig {
  officialSiteUrl: string
  officialSiteText: string
  updateEnabled: boolean
  updateText: string
  projectUrl: string
  projectText: string
  referenceProjectUrl: string
  referenceProjectText: string
  joinGroupUrl: string
  joinGroupText: string
}

const defaultConfig: AboutConfig = {
  officialSiteUrl: 'https://xymusic.cc',
  officialSiteText: '前往官网',
  updateEnabled: true,
  updateText: '检查更新',
  projectUrl: 'https://github.com/TaXiaoQi/XianYu-Music-Desktop',
  projectText: '开源地址',
  referenceProjectUrl: 'https://github.com/Billy636/XianYuMusic',
  referenceProjectText: '参考项目',
  joinGroupUrl: 'https://qm.qq.com/q/kvteWSD8yY',
  joinGroupText: '加入群组',
}

const loading = ref(true)
const saving = ref(false)
const form = ref<AboutConfig>({ ...defaultConfig })

async function loadConfig() {
  loading.value = true
  const res = await adminApi<Partial<AboutConfig>>('get_about_config_admin')
  if (res.code === 200 && res.data) {
    form.value = { ...defaultConfig, ...res.data }
  } else {
    showToast(res.msg || '加载配置失败')
  }
  loading.value = false
}

async function save() {
  saving.value = true
  const res = await adminApi('save_about_config', {
    ...form.value,
    updateEnabled: form.value.updateEnabled ? 1 : 0,
  })
  saving.value = false

  if (res.code === 200) {
    showToast('保存成功', 'success')
  } else {
    showToast(res.msg || '保存失败')
  }
}

onMounted(loadConfig)
</script>

<style scoped>
.about-config-page {
  max-width: 980px;
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
  background: var(--white);
  border: 1px solid var(--border);
  border-radius: 16px;
  padding: 24px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.04);
}
.section-title {
  font-size: 15px;
  font-weight: 800;
  color: var(--text);
  margin: 22px 0 12px;
}
.section-title:first-child {
  margin-top: 0;
}
.field-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px;
}
.field {
  display: flex;
  flex-direction: column;
  gap: 7px;
}
.field span,
.switch-row span {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-light);
}
.field input {
  border: 1.5px solid var(--border);
  border-radius: 10px;
  padding: 10px 12px;
  outline: none;
  background: #fafafa;
  font-size: 14px;
  transition: border-color 0.2s, background 0.2s;
}
.field input:focus {
  border-color: var(--accent);
  background: var(--white);
}
.switch-row {
  display: flex;
  align-items: center;
  gap: 10px;
  min-height: 66px;
}
.switch-row input {
  width: 18px;
  height: 18px;
  accent-color: var(--accent);
}
.hint {
  margin: 22px 0 0;
  font-size: 12px;
  color: var(--text-muted);
}
@keyframes spin {
  to { transform: rotate(360deg); }
}
@media (max-width: 760px) {
  .page-header,
  .field-grid {
    grid-template-columns: 1fr;
    flex-direction: column;
  }
}

/* ===== 过渡动画 ===== */
.fade-down-enter-active, .fade-down-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-down-enter-from { opacity: 0; transform: translateY(-12px); }

.fade-up-enter-active, .fade-up-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-up-enter-from { opacity: 0; transform: translateY(12px); }
</style>
