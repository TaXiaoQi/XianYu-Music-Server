<template>
  <div class="about-config-page">
    <Transition name="fade-down" appear>
      <div class="page-header">
      <div>
        <h2 class="page-title">关于页配置</h2>
        <p class="page-desc">按客户端平台分别配置关于页的官网、检查更新、项目地址等入口，保存后由对应平台客户端从后台下发。</p>
      </div>
      <button class="btn-save" :disabled="saving" @click="save">
        <span v-if="saving" class="spinner"></span>
        {{ saving ? '保存中...' : '保存配置' }}
        </button>
      </div>
    </Transition>

    <div class="platform-tabs">
      <button
        v-for="p in PLATFORMS"
        :key="p.key"
        class="platform-tab"
        :class="{ active: platform === p.key }"
        @click="switchPlatform(p.key)"
      >{{ p.label }}</button>
    </div>

    <Transition name="fade-up" appear>
      <div v-if="loading" class="state-box">
        <span class="loader"></span>
        加载中...
      </div>
      <div v-else class="config-card">

      <div class="section-title">官网入口</div>
      <div class="field-grid">
        <label class="field">
          <span>官网链接</span>
          <input v-model="form.officialSiteUrl" type="text" placeholder="https://..." />
        </label>
      </div>

      <div class="section-title">加入群组入口</div>
      <div class="field-grid">
        <label class="field">
          <span>群组链接</span>
          <input v-model="form.joinGroupUrl" type="text" placeholder="https://..." />
        </label>
      </div>

      <div class="section-title">检查更新入口</div>
      <label class="switch-row">
        <input v-model="form.updateEnabled" type="checkbox" />
        <span>显示检查更新按钮</span>
      </label>

      <div class="section-title">项目地址</div>
      <div class="field-grid">
        <label class="field">
          <span>项目链接</span>
          <input v-model="form.projectUrl" type="text" placeholder="https://..." />
        </label>
      </div>

      <div class="section-title">参考项目</div>
      <div class="ack-list">
        <div v-for="(item, index) in form.referenceProjects" :key="index" class="ack-item">
          <label class="field">
            <span>项目名</span>
            <input v-model="item.name" type="text" placeholder="项目名" />
          </label>
          <label class="field">
            <span>项目链接</span>
            <input v-model="item.url" type="text" placeholder="https://..." />
          </label>
          <button type="button" class="btn-remove" @click="removeItem('referenceProjects', index)">删除</button>
        </div>
        <button type="button" class="btn-add" @click="addItem('referenceProjects')">＋ 添加参考项目</button>
      </div>

      <div class="section-title">致谢名单</div>
      <div class="ack-list">
        <div v-for="(item, index) in form.acknowledgements" :key="index" class="ack-item">
          <label class="field">
            <span>名字</span>
            <input v-model="item.name" type="text" placeholder="名字" />
          </label>
          <label class="field">
            <span>主页链接</span>
            <input v-model="item.url" type="text" placeholder="https://..." />
          </label>
          <button type="button" class="btn-remove" @click="removeItem('acknowledgements', index)">删除</button>
        </div>
        <button type="button" class="btn-add" @click="addItem('acknowledgements')">＋ 添加成员</button>
      </div>

      <p class="hint">各个入口只配置链接，按钮显示文字由客户端按语言本地化，不再由后台覆盖。参考项目为列表，客户端“参考项目”按钮点击后以弹窗展示全部条目；致谢名单展示在“参考项目”旁的“致谢名单”弹窗中。留空的名字会被忽略；三个平台的配置独立存储、互不影响。</p>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'

interface AcknowledgementsItem {
  name: string
  url: string
}

interface AboutConfig {
  officialSiteUrl: string
  updateEnabled: boolean
  projectUrl: string
  joinGroupUrl: string
  referenceProjects: AcknowledgementsItem[]
  acknowledgements: AcknowledgementsItem[]
}

const desktopDefaults: AboutConfig = {
  officialSiteUrl: 'https://xianyumusic.cn',
  updateEnabled: true,
  projectUrl: 'https://github.com/TaXiaoQi/XianYu-Music-Desktop',
  joinGroupUrl: 'https://qm.qq.com/q/kvteWSD8yY',
  referenceProjects: [
    { name: 'Lycia Player', url: 'https://github.com/Billy636/LyciaMusic' },
    { name: 'BakaMusic', url: 'https://github.com/Zencok/BakaMusic' },
  ],
  acknowledgements: [
    { name: '@Billy636', url: 'https://github.com/Billy636' },
    { name: '@Zencok', url: 'https://github.com/Zencok' },
    { name: '@kiomosu', url: 'https://github.com/kiomosu' },
  ],
}

const mobileDefaults: AboutConfig = {
  ...desktopDefaults,
  projectUrl: 'https://github.com/TaXiaoQi/XianYu-Music-Mobile',
  referenceProjects: [
    { name: '弦予音乐桌面端', url: 'https://github.com/TaXiaoQi/XianYu-Music-Desktop' },
    { name: 'BakaMusic', url: 'https://github.com/Zencok/BakaMusic' },
    { name: 'BiliPai', url: 'https://github.com/jay3-yy/BiliPai/releases' },
    { name: 'RawS', url: 'https://github.com/QFDY-GZC/RawS-Music' },
  ],
  acknowledgements: [
    { name: '@Zencok', url: 'https://github.com/Zencok' },
    { name: '@jay3-yy', url: 'https://github.com/jay3-yy' },
    { name: '@QFDY-GZC', url: 'https://github.com/QFDY-GZC' },
  ],
}

const watchDefaults: AboutConfig = {
  ...desktopDefaults,
  referenceProjects: [
    { name: '弦予音乐移动端', url: 'https://github.com/TaXiaoQi/XianYu-Music-Mobile' },
  ],
}

const loading = ref(true)
const saving = ref(false)
const platform = ref<'desktop' | 'mobile' | 'watch'>('desktop')
const defaultConfig = computed(() =>
  platform.value === 'mobile' ? mobileDefaults : platform.value === 'watch' ? watchDefaults : desktopDefaults,
)
const form = ref<AboutConfig>({ ...desktopDefaults })

const PLATFORMS = [
  { key: 'desktop' as const, label: '桌面端' },
  { key: 'mobile' as const, label: '移动端' },
  { key: 'watch' as const, label: '腕上端' },
]

function switchPlatform(key: 'desktop' | 'mobile' | 'watch') {
  if (platform.value === key || loading.value || saving.value) return
  platform.value = key
  loadConfig()
}

async function loadConfig() {
  loading.value = true
  const res = await adminApi<Partial<AboutConfig>>('get_about_config_admin', { platform: platform.value })
  if (res.code === 200 && res.data) {
    form.value = { ...defaultConfig.value, ...res.data }
    if (!Array.isArray(form.value.referenceProjects)) {
      form.value.referenceProjects = [...defaultConfig.value.referenceProjects]
    }
    if (!Array.isArray(form.value.acknowledgements)) {
      form.value.acknowledgements = [...defaultConfig.value.acknowledgements]
    }
  } else {
    showToast(res.msg || '加载配置失败')
  }
  loading.value = false
}

function addItem(list: 'referenceProjects' | 'acknowledgements') {
  form.value[list].push({ name: '', url: '' })
}

function removeItem(list: 'referenceProjects' | 'acknowledgements', index: number) {
  form.value[list].splice(index, 1)
}

async function save() {
  saving.value = true
  const res = await adminApi('save_about_config', {
    platform: platform.value,
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
.platform-tabs {
  display: inline-flex;
  gap: 4px;
  padding: 4px;
  margin-bottom: 16px;
  background: var(--control-bg);
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
.platform-tab:hover {
  color: var(--text);
}
.platform-tab.active {
  background: var(--card-solid);
  color: var(--accent);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
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
  background: var(--control-bg);
  font-size: 14px;
  transition: border-color 0.2s, background 0.2s;
}
.field input:focus {
  border-color: var(--accent);
  background: var(--card-solid);
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
.ack-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.ack-item {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1.4fr) auto;
  gap: 12px;
  align-items: end;
  padding: 12px;
  border: 1px solid var(--border);
  border-radius: 12px;
  background: var(--control-bg);
}
.btn-add,
.btn-remove {
  border: 1.5px dashed var(--border);
  border-radius: 10px;
  background: transparent;
  color: var(--text-light);
  padding: 9px 14px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}
.btn-add:hover {
  color: var(--accent);
  border-color: var(--accent);
}
.btn-remove {
  border-style: solid;
  color: var(--danger, #e02020);
  white-space: nowrap;
  align-self: end;
}
.btn-remove:hover {
  background: rgba(224, 32, 32, 0.08);
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
  .ack-item {
    grid-template-columns: 1fr;
  }
}

/* ===== 过渡动画 ===== */
.fade-down-enter-active, .fade-down-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-down-enter-from { opacity: 0; transform: translateY(-12px); }

.fade-up-enter-active, .fade-up-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-up-enter-from { opacity: 0; transform: translateY(12px); }
</style>
