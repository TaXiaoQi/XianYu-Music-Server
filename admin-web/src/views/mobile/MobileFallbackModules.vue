<template>
  <div class="mobile-page">
    <!-- 页头 -->
    <div class="fb-header">
      <div class="fb-header-info">
        <h2 class="fb-title">兜底管理</h2>
        <p class="fb-desc">桌面端内置各兜底行为的默认实现；在此下发的新代码在客户端下次拉取后缓存、下一次调用生效，异常时自动回退内置实现。</p>
      </div>
    </div>

    <!-- 统计 -->
    <div class="mobile-grid" style="grid-template-columns: repeat(3, minmax(0, 1fr));">
      <div class="mobile-stat">
        <div class="stat-icon-row">
          <span class="stat-icon stat-icon-total">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg>
          </span>
          <strong>{{ modules.length }}</strong>
        </div>
        <span class="stat-label">全部模块</span>
      </div>
      <div class="mobile-stat">
        <div class="stat-icon-row">
          <span class="stat-icon stat-icon-on">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
          </span>
          <strong>{{ configuredCount }}</strong>
        </div>
        <span class="stat-label">已配置</span>
      </div>
      <div class="mobile-stat">
        <div class="stat-icon-row">
          <span class="stat-icon stat-icon-normal">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>
          </span>
          <strong>{{ enabledCount }}</strong>
        </div>
        <span class="stat-label">已启用</span>
      </div>
    </div>

    <!-- 加载中 -->
    <div v-if="loading" class="mobile-empty">加载中...</div>

    <!-- 模块列表 -->
    <div v-else class="fb-list">
      <div
        v-for="(item, idx) in modules"
        :key="item.moduleKey"
        class="fb-card"
        :class="{ 'is-off': isConfigured(item) && !item.enabled }"
        :style="{ animationDelay: (idx * 0.05) + 's' }"
      >
        <div class="fb-body">
          <div class="fb-top">
            <div class="fb-title-wrap">
              <h3 class="fb-name">{{ item.name || moduleLabel(item.moduleKey) }}</h3>
              <span class="fb-key">{{ item.moduleKey }}</span>
            </div>
            <label class="switch" :class="{ 'switch-off': !isConfigured(item) }" :title="isConfigured(item) ? (item.enabled ? '点击禁用' : '点击启用') : '请先配置代码'">
              <input
                :checked="!!item.enabled"
                :disabled="!isConfigured(item)"
                type="checkbox"
                @change="toggle(item, ($event.target as HTMLInputElement).checked)"
              />
              <span class="track"><span class="thumb"></span></span>
            </label>
          </div>

          <div v-if="isConfigured(item)" class="fb-meta">
            <span>v{{ item.version }} · {{ fmtDateTime(item.updated_at) || '-' }}</span>
            <span class="fb-digest" :title="item.digest">sha256:{{ shortDigest(item.digest) }}</span>
            <span v-if="item.remark">{{ item.remark }}</span>
          </div>
          <div v-else class="fb-meta fb-builtin">使用内置默认实现</div>

          <div class="fb-footer">
            <span class="fb-size">{{ codeSizeLabel(item) }}</span>
            <div class="fb-actions">
              <button class="mobile-btn" @click="openEditModal(item)">{{ isConfigured(item) ? '编辑' : '配置' }}</button>
              <button v-if="isConfigured(item)" class="mobile-btn danger" @click="remove(item)">删除</button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 编辑弹窗 -->
    <Transition name="modal" @before-leave="removeBackdropBlur">
      <div v-if="modalVisible" class="modal-backdrop">
        <div class="modal-dialog">
          <div class="modal-head">
            <h3>{{ isConfigured(form) ? '编辑兜底模块' : '配置兜底模块' }}</h3>
            <button class="modal-close" @click="closeModal">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
          <div class="modal-body">
            <div class="modal-field">
              <span>模块</span>
              <div class="fb-module-line">{{ form.moduleKey }} · {{ moduleLabel(form.moduleKey) }}</div>
            </div>
            <label class="modal-field">
              <span>名称 <em class="optional">（可空）</em></span>
              <input v-model="form.name" type="text" :placeholder="moduleLabel(form.moduleKey)" />
            </label>
            <label class="modal-field">
              <span>备注 <em class="optional">（可空）</em></span>
              <input v-model="form.remark" type="text" placeholder="如：修复酷我搜索失效" />
            </label>
            <div class="modal-field">
              <span>启用状态</span>
              <div class="type-picker">
                <button class="type-option pick-enable" :class="{ active: form.enabled }" @click="form.enabled = true"><span class="pick-dot"></span>启用下发</button>
                <button class="type-option pick-disable" :class="{ active: !form.enabled }" @click="form.enabled = false"><span class="pick-dot"></span>仅保存</button>
              </div>
            </div>
            <div class="modal-field">
              <div class="fb-code-head">
                <span class="required">模块代码（JS）</span>
                <button class="mobile-btn fb-tpl-btn" @click="insertTemplate">填入模板</button>
              </div>
              <textarea v-model="form.code" class="fb-code-area" rows="12" spellcheck="false" placeholder="函数体接收 ctx（宿主能力注入），return { version, 方法名(args) { ... } }"></textarea>
              <p class="fb-hint">代码变更保存后版本号自动 +1，客户端下次拉取后缓存、下一次调用生效。</p>
            </div>
          </div>
          <div class="modal-foot">
            <button class="modal-btn cancel" @click="closeModal">取消</button>
            <button class="modal-btn save" :disabled="saving" @click="save">{{ saving ? '保存中...' : '保存' }}</button>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'
import './MobilePage.css'
import { mobileConfirm, removeBackdropBlur } from '@/utils/mobileDialog'
import { fmtDateTime } from '@/utils/time'

interface FallbackModule {
  moduleKey: string
  name: string
  version: number
  digest: string
  code: string
  remark: string
  enabled: boolean
  configured?: boolean
  created_at?: string
  updated_at?: string
}

const MODULE_LABELS: Record<string, string> = {
  lx_search: '落雪歌曲搜索',
  lx_album: '专辑/歌单获取',
  lx_duration: '歌曲时长加载',
  lx_lyric: '逐字歌词解码',
  lx_cover: '歌曲封面提取',
  plugin_fallback: '插件宿主兜底',
}

const MODULE_METHODS: Record<string, string[]> = {
  lx_search: ['search'],
  lx_album: ['searchAlbums', 'getAlbumSongs'],
  lx_duration: ['batchTrackInterval'],
  lx_lyric: ['fetchLyric'],
  lx_cover: ['extractCoverUrl'],
  plugin_fallback: [
    'isQqMusicPluginSource',
    'hostSearchFallback',
    'hostAlbumSearchFallback',
    'hostAlbumSongsFallback',
    'isQqTrialMediaUrl',
    'fillSongDurations',
  ],
}

function moduleLabel(key: string): string {
  return MODULE_LABELS[key] || key
}

function moduleMethods(key: string): string[] {
  return MODULE_METHODS[key] || []
}

const loading = ref(true)
const saving = ref(false)
const modules = ref<FallbackModule[]>([])

const configuredCount = computed(() => modules.value.filter(m => isConfigured(m)).length)
const enabledCount = computed(() => modules.value.filter(m => isConfigured(m) && m.enabled).length)

function isConfigured(item: FallbackModule): boolean {
  return item.configured !== false && !!item.code
}

async function loadList() {
  loading.value = true
  const res = await adminApi<{ list: FallbackModule[] }>('list_fallback_modules')
  if (res.code === 200 && res.data) {
    modules.value = Array.isArray(res.data.list) ? res.data.list : []
  } else {
    modules.value = []
    showToast(res.msg || '加载失败')
  }
  loading.value = false
}

function shortDigest(digest: string): string {
  if (!digest) return '-'
  return digest.length > 12 ? `${digest.slice(0, 8)}…${digest.slice(-4)}` : digest
}

function codeSizeLabel(item: FallbackModule): string {
  if (!isConfigured(item)) return '内置默认'
  const kb = item.code.length / 1024
  return kb >= 1 ? `${kb.toFixed(1)} KB` : `${item.code.length} B`
}

async function toggle(item: FallbackModule, enabled: boolean) {
  if (!isConfigured(item)) {
    showToast('请先配置模块代码')
    return
  }
  const res = await adminApi('toggle_fallback_module', { module_key: item.moduleKey, enabled: enabled ? 1 : 0 })
  if (res.code === 200) {
    showToast(res.msg || (enabled ? '已启用' : '已禁用'), 'success')
    loadList()
  } else {
    showToast(res.msg || '操作失败')
  }
}

async function remove(item: FallbackModule) {
  const ok = await mobileConfirm(`确认删除「${moduleLabel(item.moduleKey)}」的下发配置？客户端将回退内置默认实现。`, {
    title: '删除兜底模块',
    confirmText: '确认删除',
    danger: true,
  })
  if (!ok) return
  const res = await adminApi('delete_fallback_module', { module_key: item.moduleKey })
  if (res.code === 200) {
    showToast('删除成功', 'success')
    loadList()
  } else {
    showToast(res.msg || '删除失败')
  }
}

// ===== 弹窗 =====
const modalVisible = ref(false)
const form = ref<FallbackModule>({
  moduleKey: '',
  name: '',
  version: 0,
  digest: '',
  code: '',
  remark: '',
  enabled: true,
})

function openEditModal(item: FallbackModule) {
  form.value = {
    moduleKey: item.moduleKey,
    name: isConfigured(item) ? (item.name || '') : '',
    version: item.version || 0,
    digest: item.digest || '',
    code: item.code || '',
    remark: item.remark || '',
    enabled: isConfigured(item) ? !!item.enabled : true,
  }
  modalVisible.value = true
}

function closeModal() {
  if (saving.value) return
  modalVisible.value = false
}

function codeTemplate(key: string): string {
  const methods = moduleMethods(key)
  const methodLines = methods
    .map(m => `  ${m}: (args) => {\n    // 返回值结构需与内置实现一致；抛出异常时客户端自动回退内置实现\n    throw new Error('not implemented')\n  },`)
    .join('\n')
  return `// 兜底模块：函数体接收 ctx（宿主能力白名单），返回 { version, 方法名(args) }
// ctx.http.get/post / ctx.cache.get/set/del / ctx.log.info/warn/error
// ctx.config.get('dot.path') / ctx.utils.parseIntervalToSeconds 等
return {
  version: 1,
${methodLines}
}`
}

async function insertTemplate() {
  if (form.value.code.trim()) {
    const ok = await mobileConfirm('当前已有代码，填入模板会覆盖现有内容，是否继续？', { title: '覆盖代码', confirmText: '覆盖' })
    if (!ok) return
  }
  form.value.code = codeTemplate(form.value.moduleKey)
}

async function save() {
  if (!form.value.code.trim()) {
    showToast('模块代码不能为空')
    return
  }
  saving.value = true
  const res = await adminApi('save_fallback_module', {
    module_key: form.value.moduleKey,
    code: form.value.code,
    name: form.value.name.trim(),
    remark: form.value.remark.trim(),
    enabled: form.value.enabled ? 1 : 0,
  })
  saving.value = false
  if (res.code === 200) {
    showToast(res.msg || '保存成功', 'success')
    closeModal()
    loadList()
  } else {
    showToast(res.msg || '保存失败')
  }
}

onMounted(loadList)
</script>

<style scoped>
.fb-header { display: flex; flex-direction: column; gap: 10px; }
.fb-header-info { min-width: 0; }
.fb-title { font-size: 18px; font-weight: 850; margin: 0 0 4px; color: var(--text); }
.fb-desc { font-size: 12px; color: var(--text-light); line-height: 1.6; margin: 0; }

.fb-list { display: flex; flex-direction: column; gap: 12px; }
.fb-card {
  border: 1px solid var(--border);
  border-radius: 18px;
  background: var(--card);
  box-shadow: var(--shadow-soft);
  overflow: hidden;
  animation: fbCardIn 0.4s var(--motion, cubic-bezier(0.16, 1, 0.3, 1)) both;
}
.fb-card.is-off { opacity: 0.65; }
@keyframes fbCardIn {
  from { opacity: 0; transform: translateY(12px); }
  to { opacity: 1; transform: translateY(0); }
}
.fb-body { padding: 14px 15px; display: flex; flex-direction: column; gap: 8px; min-width: 0; }
.fb-top { display: flex; justify-content: space-between; align-items: flex-start; gap: 10px; }
.fb-title-wrap { display: flex; flex-direction: column; gap: 4px; min-width: 0; }
.fb-name { font-size: 15px; font-weight: 750; margin: 0; color: var(--text); line-height: 1.4; }
.fb-key {
  align-self: flex-start;
  padding: 2px 8px;
  border-radius: 6px;
  font-size: 11px;
  font-family: ui-monospace, 'SF Mono', Consolas, monospace;
  background: var(--track);
  color: var(--text-light);
}

.switch { position: relative; display: inline-flex; flex-shrink: 0; cursor: pointer; }
.switch.switch-off { opacity: 0.45; cursor: not-allowed; }
.switch input { display: none; }
.switch .track {
  width: 40px;
  height: 22px;
  border-radius: 999px;
  background: var(--border);
  position: relative;
  transition: background 0.25s;
}
.switch .thumb {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: var(--card-solid);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
  transition: transform 0.25s;
}
.switch input:checked + .track { background: #EC4141; }
.switch input:checked + .track .thumb { transform: translateX(18px); }

.fb-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 4px 12px;
  font-size: 11px;
  color: var(--text-muted);
}
.fb-meta.fb-builtin { font-style: italic; }
.fb-digest { font-family: ui-monospace, 'SF Mono', Consolas, monospace; }

.fb-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 10px;
  margin-top: auto;
  padding-top: 10px;
  border-top: 1px solid var(--border);
}
.fb-size { font-size: 11px; color: var(--text-muted); font-family: ui-monospace, 'SF Mono', Consolas, monospace; }
.fb-actions { display: flex; gap: 8px; flex-shrink: 0; }

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
.modal-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 18px 20px 0;
}
.modal-head h3 { margin: 0; font-size: 16px; font-weight: 850; color: var(--text); }
.modal-close {
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  padding: 4px;
  border-radius: 8px;
  display: flex;
}
.modal-close:active { background: var(--control-bg); color: var(--text); }
.modal-body {
  padding: 14px 20px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  overflow-y: auto;
}
.modal-field { display: flex; flex-direction: column; gap: 6px; }
.modal-field > span { font-size: 12px; font-weight: 700; color: var(--text-light); }
.modal-field > span.required::after { content: '*'; color: #ef4444; margin-left: 2px; }
.modal-field .optional { font-style: normal; font-weight: 400; color: var(--text-muted); font-size: 11px; }
.modal-field input,
.modal-field textarea {
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 11px 12px;
  font-size: 14px;
  font-family: inherit;
  outline: none;
  background: var(--control-bg);
  color: var(--text);
  resize: vertical;
  transition: border-color 0.18s, box-shadow 0.18s;
  width: 100%;
  box-sizing: border-box;
}
.modal-field input:focus,
.modal-field textarea:focus { border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-soft); }

.fb-module-line {
  font-size: 12.5px;
  color: var(--text-light);
  font-family: ui-monospace, 'SF Mono', Consolas, monospace;
  background: var(--control-bg);
  border-radius: 10px;
  padding: 10px 12px;
}
.fb-code-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.fb-tpl-btn { padding: 5px 12px; font-size: 12px; }
.fb-code-area {
  font-family: ui-monospace, 'SF Mono', Consolas, monospace;
  font-size: 12px;
  line-height: 1.6;
  white-space: pre;
  overflow-x: auto;
  tab-size: 2;
}
.fb-hint { font-size: 11px; color: var(--text-muted); margin: 0; line-height: 1.5; }

.type-picker { display: flex; gap: 8px; flex-wrap: wrap; }
.type-option {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 13px;
  border-radius: 10px;
  border: 1.5px solid var(--border);
  background: transparent;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-light);
  cursor: pointer;
  transition: all 0.18s;
}
.type-option.active { font-weight: 700; }
.pick-dot { width: 8px; height: 8px; border-radius: 50%; background: var(--border); transition: background 0.18s; }
.pick-enable.active { background: #ecfdf5; color: #10b981; border-color: transparent; }
.pick-enable.active .pick-dot { background: #10b981; }
.pick-disable.active { background: var(--control-bg); color: var(--text-muted); border-color: transparent; }
.pick-disable.active .pick-dot { background: #6b7280; }

.modal-foot { display: flex; gap: 10px; padding: 14px 20px 18px; }
.modal-btn {
  flex: 1;
  padding: 11px;
  border-radius: 12px;
  font-size: 14px;
  font-weight: 700;
  border: none;
  cursor: pointer;
}
.modal-btn.cancel { background: var(--control-bg); color: var(--text-light); }
.modal-btn.save { background: var(--accent); color: #fff; }
.modal-btn.save:disabled { opacity: 0.6; }

.modal-enter-active { transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1); }
.modal-leave-active { transition: all 0.2s ease; }
.modal-enter-from,
.modal-leave-to { opacity: 0; }
.modal-enter-from .modal-dialog,
.modal-leave-to .modal-dialog { transform: scale(0.92) translateY(20px); }
</style>
