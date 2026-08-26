<template>
  <div class="fb-page">
    <!-- 页面头部 -->
    <Transition name="fade-down" appear>
      <div class="page-header">
        <div class="header-info">
          <h2 class="page-title">兜底管理</h2>
          <p class="page-desc">
            桌面端内置了各音源兜底行为（搜索、专辑、时长、逐字歌词、封面等）的默认实现。在此下发的新实现会在客户端下次拉取（启动时/每 30 分钟）后缓存，下一次调用生效；执行异常时客户端自动回退内置实现。
          </p>
        </div>
      </div>
    </Transition>

    <!-- 统计栏 -->
    <Transition name="fade-up" appear>
      <div class="stats-row">
        <div class="stat-chip stat-total">
          <div class="stat-icon">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg>
          </div>
          <div class="stat-body">
            <span class="stat-num">{{ modules.length }}</span>
            <span class="stat-label">全部模块</span>
          </div>
        </div>
        <div class="stat-chip stat-on">
          <div class="stat-icon">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
          </div>
          <div class="stat-body">
            <span class="stat-num">{{ configuredCount }}</span>
            <span class="stat-label">已配置下发</span>
          </div>
        </div>
        <div class="stat-chip stat-off">
          <div class="stat-icon">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>
          </div>
          <div class="stat-body">
            <span class="stat-num">{{ enabledCount }}</span>
            <span class="stat-label">已启用</span>
          </div>
        </div>
      </div>
    </Transition>

    <!-- 加载中 -->
    <div v-if="loading" class="state-box">
      <div class="spinner"></div>
      <span>加载中...</span>
    </div>

    <!-- 加载失败 -->
    <div v-else-if="loadError" class="state-box state-error">
      <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
      <span>{{ loadError }}</span>
    </div>

    <!-- 模块卡片列表 -->
    <div v-else class="card-grid">
      <TransitionGroup name="card">
        <div
          v-for="(item, idx) in modules"
          :key="item.moduleKey"
          class="fb-card"
          :class="{ configured: isConfigured(item), off: isConfigured(item) && !item.enabled }"
          :style="{ animationDelay: `${idx * 60}ms` }"
        >
          <div class="card-body">
            <div class="card-top">
              <div class="card-title-wrap">
                <h3 class="card-title">{{ item.name || moduleLabel(item.moduleKey) }}</h3>
                <span class="key-badge">{{ item.moduleKey }}</span>
              </div>
              <label
                class="toggle-switch"
                :title="isConfigured(item) ? (item.enabled ? '点击禁用（客户端回退内置实现）' : '点击启用下发') : '请先配置模块代码'"
                :class="{ 'toggle-disabled': !isConfigured(item) }"
              >
                <input
                  type="checkbox"
                  :checked="!!item.enabled"
                  :disabled="!isConfigured(item)"
                  @change="toggleModule(item, ($event.target as HTMLInputElement).checked)"
                />
                <span class="toggle-slider"></span>
              </label>
            </div>

            <div class="card-methods">
              <span
                v-for="m in moduleMethods(item.moduleKey)"
                :key="m"
                class="method-chip"
                :class="{ covered: isMethodCovered(item, m) }"
              >{{ m }}</span>
            </div>

            <div v-if="isConfigured(item)" class="card-meta">
              <span class="meta-item">
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
                v{{ item.version }} · {{ fmtDateTime(item.updated_at) || '-' }}
              </span>
              <span class="meta-item digest" :title="item.digest">sha256:{{ shortDigest(item.digest) }}</span>
              <span v-if="item.remark" class="meta-item remark">{{ item.remark }}</span>
            </div>
            <div v-else class="card-meta">
              <span class="meta-item builtin">使用内置默认实现</span>
            </div>

            <div class="card-footer">
              <span class="code-size">{{ codeSizeLabel(item) }}</span>
              <div class="card-actions">
                <button class="icon-btn" :title="isConfigured(item) ? '编辑' : '配置下发'" @click="openEditModal(item)">
                  <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
                </button>
                <button
                  v-if="isConfigured(item)"
                  class="icon-btn icon-btn-danger"
                  title="删除（客户端回退内置实现）"
                  @click="deleteModule(item)"
                >
                  <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
                </button>
              </div>
            </div>
          </div>
        </div>
      </TransitionGroup>
    </div>

    <!-- 编辑弹窗 -->
    <Transition name="modal">
      <div v-if="modalVisible" class="modal-backdrop">
        <div class="modal-dialog modal-wide">
          <div class="modal-head">
            <div class="modal-head-left">
              <h3>{{ isConfigured(form) ? '编辑兜底模块' : '配置兜底模块' }}</h3>
              <span class="modal-head-key">{{ form.moduleKey }} · {{ moduleLabel(form.moduleKey) }}</span>
            </div>
            <button class="modal-close" @click="closeModal">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
          <div class="modal-form">
            <div class="field-row">
              <div class="field">
                <label>模块名称 <span class="field-optional">（可空，默认使用系统名称）</span></label>
                <input v-model="form.name" type="text" :placeholder="moduleLabel(form.moduleKey)" />
              </div>
              <div class="field">
                <label>备注 <span class="field-optional">（可空）</span></label>
                <input v-model="form.remark" type="text" placeholder="如：修复酷我搜索失效" />
              </div>
            </div>
            <div class="field">
              <label>启用状态</label>
              <div class="type-picker">
                <button class="type-option pick-enable" :class="{ active: form.enabled }" @click="form.enabled = true">
                  <span class="pick-dot"></span>启用下发
                </button>
                <button class="type-option pick-disable" :class="{ active: !form.enabled }" @click="form.enabled = false">
                  <span class="pick-dot"></span>仅保存
                </button>
              </div>
            </div>
            <div class="field">
              <div class="code-field-head">
                <label class="required">模块代码（JS）</label>
                <div class="code-field-tools">
                  <span class="code-count">{{ form.code.length }} 字符</span>
                  <button class="btn-tpl" @click="insertTemplate">填入模板</button>
                </div>
              </div>
              <textarea
                ref="codeAreaRef"
                v-model="form.code"
                class="code-area"
                rows="14"
                spellcheck="false"
                placeholder="函数体接收 ctx（宿主能力注入），return { version, 方法名(args) { ... } }"
                @keydown.tab.prevent="insertTab"
              ></textarea>
              <p class="code-hint">
                代码变更保存后版本号自动 +1，客户端下次拉取后缓存、下一次调用生效；代码未变更时版本号不变，客户端不会重复刷新。
              </p>
            </div>
            <details class="ctx-panel">
              <summary>查看 ctx 能力契约（点击展开）</summary>
              <pre class="ctx-pre">ctx.appVersion                          // 客户端版本号
ctx.http.get(url, opts)                 // 经客户端 Rust 代理的 GET（绕过 CORS）
ctx.http.post(url, body, opts)          // POST，body 为对象时自动 JSON 序列化
  // opts: { headers: {..}, timeoutMs }  → 返回 { status, headers, body }
ctx.cache.get(key)                      // 模块级内存缓存读取
ctx.cache.set(key, value, ttlSeconds?)  // 写入（可带 TTL）
ctx.cache.del(key)
ctx.log.info / warn / error(msg, data?) // 输出到客户端控制台
ctx.config.get('dot.path')              // 只读用户设置
ctx.utils.parseIntervalToSeconds('04:30')  // → 270
ctx.utils.normalizeQualityKey(raw)      // 任意音质标识 → QualityKey
ctx.utils.stripHtmlTags(html)</pre>
            </details>
          </div>
          <div class="modal-foot">
            <button class="btn-cancel" @click="closeModal">取消</button>
            <button class="btn-save" :disabled="saving" @click="save">
              <span v-if="saving" class="btn-spinner"></span>
              {{ saving ? '保存中...' : '保存' }}
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
import { webConfirm } from '@/utils/webDialog'
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

/** 与桌面端 FALLBACK_MODULE_METHODS 对应的方法清单 */
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

const MODULE_LABELS: Record<string, string> = {
  lx_search: '落雪歌曲搜索',
  lx_album: '专辑/歌单获取',
  lx_duration: '歌曲时长加载',
  lx_lyric: '逐字歌词解码',
  lx_cover: '歌曲封面提取',
  plugin_fallback: '插件宿主兜底',
}

function moduleLabel(key: string): string {
  return MODULE_LABELS[key] || key
}

function moduleMethods(key: string): string[] {
  return MODULE_METHODS[key] || []
}

/** 方法是否被当前代码覆盖（简单文本探测，仅供后台提示） */
function isMethodCovered(item: FallbackModule, method: string): boolean {
  if (!item.code) return false
  return new RegExp(`\\b${method}\\s*[:=(]`).test(item.code)
}

// ===== 列表 =====
const modules = ref<FallbackModule[]>([])
const loading = ref(true)
const loadError = ref('')

const configuredCount = computed(() => modules.value.filter(m => isConfigured(m)).length)
const enabledCount = computed(() => modules.value.filter(m => isConfigured(m) && m.enabled).length)

function isConfigured(item: FallbackModule): boolean {
  return item.configured !== false && !!item.code
}

async function loadList() {
  loading.value = true
  loadError.value = ''
  const res = await adminApi<{ list: FallbackModule[] }>('list_fallback_modules')
  if (res.code === 200 && res.data) {
    modules.value = Array.isArray(res.data.list) ? res.data.list : []
  } else {
    loadError.value = res.msg || '加载失败'
    modules.value = []
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

// ===== 切换状态 =====
async function toggleModule(item: FallbackModule, enabled: boolean) {
  if (!isConfigured(item)) {
    showToast('请先配置模块代码')
    loadList()
    return
  }
  const res = await adminApi('toggle_fallback_module', { module_key: item.moduleKey, enabled: enabled ? 1 : 0 })
  if (res.code === 200) {
    showToast(res.msg || (enabled ? '已启用' : '已禁用'), 'success')
    loadList()
  } else {
    showToast(res.msg || '操作失败')
    loadList()
  }
}

// ===== 删除 =====
async function deleteModule(item: FallbackModule) {
  const ok = await webConfirm(`确认删除「${moduleLabel(item.moduleKey)}」的下发配置？客户端将在下次拉取后回退内置默认实现。`, {
    title: '删除兜底模块',
    confirmText: '确认删除',
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
const saving = ref(false)
const codeAreaRef = ref<HTMLTextAreaElement | null>(null)
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

function insertTab(e: KeyboardEvent) {
  const el = e.target as HTMLTextAreaElement
  const start = el.selectionStart
  const end = el.selectionEnd
  form.value.code = `${form.value.code.slice(0, start)}  ${form.value.code.slice(end)}`
  requestAnimationFrame(() => {
    el.selectionStart = el.selectionEnd = start + 2
  })
}

function codeTemplate(key: string): string {
  const methods = moduleMethods(key)
  const methodLines = methods
    .map(m => `  ${m}: (args) => {\n    // args: ${key === 'lx_search' ? '{ source, keyword, page, limit }' : '见该方法的内置实现入参'}\n    // 返回值结构需与内置实现一致；抛出异常时客户端自动回退内置实现\n    throw new Error('not implemented')\n  },`)
    .join('\n')
  return `// 兜底模块：函数体接收 ctx（宿主能力白名单），返回 { version, 方法名(args) }
// ctx.http.get/post(url, body?, opts?)  —— 经客户端 Rust 代理的网络请求
// ctx.cache.get/set/del(key, ...)       —— 模块级内存缓存（set 可带 ttlSeconds）
// ctx.log.info/warn/error(msg, data?)   —— 日志输出到客户端控制台
// ctx.config.get('dot.path')            —— 只读用户设置
// ctx.utils.parseIntervalToSeconds / normalizeQualityKey / stripHtmlTags
return {
  version: 1,
${methodLines}
}`
}

function insertTemplate() {
  if (form.value.code.trim() && form.value.code.trim() !== '') {
    // 已有代码时确认覆盖
    webConfirm('当前已有代码，填入模板会覆盖现有内容，是否继续？', { title: '覆盖代码', confirmText: '覆盖' }).then(ok => {
      if (ok) form.value.code = codeTemplate(form.value.moduleKey)
    })
    return
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

onMounted(() => {
  loadList()
})
</script>

<style scoped>
.fb-page {
  max-width: 1200px;
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
}
.page-desc {
  font-size: 13px;
  color: var(--text-muted);
  line-height: 1.6;
  margin: 0;
  max-width: 680px;
}

/* ===== 统计栏 ===== */
.stats-row {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
  margin-bottom: 24px;
}
.stat-chip {
  background: var(--card-solid);
  border: 1px solid var(--border);
  border-radius: 16px;
  padding: 20px;
  display: flex;
  align-items: center;
  gap: 16px;
  transition: all 0.2s;
}
.stat-chip:hover { transform: translateY(-3px); box-shadow: 0 8px 24px rgba(0, 0, 0, 0.06); }
.stat-icon {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.stat-total .stat-icon { background: var(--track); color: var(--text-light); }
.stat-on .stat-icon { background: #f0fdf4; color: #16a34a; }
.stat-off .stat-icon { background: rgba(59, 130, 246, 0.12); color: #3b82f6; }
.stat-body { display: flex; flex-direction: column; }
.stat-num { font-size: 26px; font-weight: 800; line-height: 1.1; color: var(--text); }
.stat-label { font-size: 12px; color: var(--text-muted); margin-top: 2px; }
@media (max-width: 768px) {
  .stats-row { grid-template-columns: 1fr 1fr 1fr; gap: 10px; }
  .stat-chip { padding: 14px; flex-direction: column; align-items: flex-start; gap: 8px; }
  .stat-num { font-size: 22px; }
}

/* ===== 加载/错误状态 ===== */
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
.state-error { color: #ef4444; }
.spinner {
  width: 28px; height: 28px;
  border: 3px solid var(--border);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }

/* ===== 模块卡片网格 ===== */
.card-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(360px, 1fr));
  gap: 16px;
}

.fb-card {
  background: var(--card-solid);
  border-radius: 14px;
  border: 1px solid var(--border);
  overflow: hidden;
  transition: transform 0.3s cubic-bezier(0.16, 1, 0.3, 1), box-shadow 0.3s ease, border-color 0.2s;
  animation: cardEnter 0.5s cubic-bezier(0.16, 1, 0.3, 1) backwards;
}
.fb-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.08);
  border-color: transparent;
}
.fb-card.off { opacity: 0.72; }
.fb-card.off:hover { opacity: 0.9; }

@keyframes cardEnter {
  from { opacity: 0; transform: translateY(20px); }
  to { opacity: 1; transform: translateY(0); }
}

.card-body {
  padding: 16px 18px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-width: 0;
}
.card-top {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 10px;
}
.card-title-wrap { display: flex; flex-direction: column; gap: 4px; min-width: 0; }
.card-title {
  font-size: 15px;
  font-weight: 700;
  margin: 0;
  color: var(--text);
  line-height: 1.4;
}
.key-badge {
  display: inline-block;
  align-self: flex-start;
  padding: 2px 8px;
  border-radius: 6px;
  font-size: 11px;
  font-family: ui-monospace, 'SF Mono', Consolas, monospace;
  background: var(--track);
  color: var(--text-light);
}

/* 方法 chips */
.card-methods {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.method-chip {
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 11px;
  font-family: ui-monospace, 'SF Mono', Consolas, monospace;
  background: var(--track);
  color: var(--text-muted);
  border: 1px dashed var(--border);
}
.method-chip.covered {
  background: #ecfdf5;
  color: #10b981;
  border-color: rgba(16, 185, 129, 0.3);
  border-style: solid;
}

/* 元信息 */
.card-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 8px 14px;
  font-size: 11px;
  color: var(--text-muted);
}
.meta-item {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}
.meta-item.digest {
  font-family: ui-monospace, 'SF Mono', Consolas, monospace;
}
.meta-item.remark { color: var(--text-light); }
.meta-item.builtin {
  color: var(--text-muted);
  font-style: italic;
}

/* 底部 */
.card-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: auto;
  padding-top: 8px;
  border-top: 1px solid #f5f5f5;
}
.code-size {
  font-size: 11px;
  color: var(--text-muted);
  font-family: ui-monospace, 'SF Mono', Consolas, monospace;
}
.card-actions { display: flex; gap: 4px; }
.icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border-radius: 8px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.2s;
}
.icon-btn:hover { background: #f5f5f5; color: var(--text); }
.icon-btn-danger:hover { background: rgba(236, 65, 65, 0.12); color: #ef4444; }

/* Toggle 开关 */
.toggle-switch {
  position: relative;
  display: inline-block;
  width: 38px;
  height: 22px;
  cursor: pointer;
  flex-shrink: 0;
}
.toggle-switch.toggle-disabled { cursor: not-allowed; opacity: 0.45; }
.toggle-switch input { opacity: 0; width: 0; height: 0; }
.toggle-slider {
  position: absolute;
  inset: 0;
  background: #d1d5db;
  border-radius: 22px;
  transition: background 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}
.toggle-slider::before {
  content: '';
  position: absolute;
  width: 16px;
  height: 16px;
  left: 3px;
  top: 3px;
  background: var(--card-solid);
  border-radius: 50%;
  box-shadow: 0 1px 3px rgba(0,0,0,0.2);
  transition: transform 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}
.toggle-switch input:checked + .toggle-slider { background: #10b981; }
.toggle-switch input:checked + .toggle-slider::before { transform: translateX(16px); }

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
  max-width: 520px;
  max-height: 90vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.16);
}
.modal-dialog.modal-wide { max-width: 760px; }
.modal-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px 16px;
}
.modal-head-left { display: flex; flex-direction: column; gap: 3px; }
.modal-head h3 {
  font-size: 17px;
  font-weight: 700;
  margin: 0;
}
.modal-head-key {
  font-size: 12px;
  color: var(--text-muted);
  font-family: ui-monospace, 'SF Mono', Consolas, monospace;
}
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
  gap: 16px;
}
.field { display: flex; flex-direction: column; gap: 6px; }
.field label {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-light);
}
.field-optional {
  font-weight: 400;
  color: var(--text-muted);
  font-size: 12px;
}
.field input {
  padding: 10px 14px;
  border: 1.5px solid var(--border);
  border-radius: 10px;
  font-size: 14px;
  outline: none;
  background: #fafafa;
  font-family: inherit;
  transition: border-color 0.2s, background 0.2s;
}
.field input:focus { border-color: var(--accent); background: var(--card-solid); }
.field-row { display: flex; gap: 16px; }
.field-row .field { flex: 1; }

/* 代码区 */
.code-field-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.code-field-tools {
  display: flex;
  align-items: center;
  gap: 10px;
}
.code-count {
  font-size: 11px;
  color: var(--text-muted);
  font-family: ui-monospace, 'SF Mono', Consolas, monospace;
}
.btn-tpl {
  padding: 4px 12px;
  border-radius: 8px;
  border: 1.5px solid var(--border);
  background: var(--card-solid);
  font-size: 12px;
  color: var(--text-light);
  cursor: pointer;
  transition: all 0.2s;
}
.btn-tpl:hover { border-color: var(--accent); color: var(--accent); }
.code-area {
  padding: 12px 14px;
  border: 1.5px solid var(--border);
  border-radius: 10px;
  font-size: 12.5px;
  line-height: 1.6;
  outline: none;
  background: #0f172a;
  color: #e2e8f0;
  font-family: ui-monospace, 'SF Mono', Consolas, monospace;
  resize: vertical;
  tab-size: 2;
  transition: border-color 0.2s;
  white-space: pre;
  overflow-wrap: normal;
  overflow-x: auto;
}
.code-area:focus { border-color: var(--accent); }
.code-hint {
  font-size: 11.5px;
  color: var(--text-muted);
  margin: 0;
  line-height: 1.5;
}

/* ctx 契约折叠面板 */
.ctx-panel {
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 10px 14px;
  background: #fafafa;
}
.ctx-panel summary {
  font-size: 12.5px;
  font-weight: 600;
  color: var(--text-light);
  cursor: pointer;
  user-select: none;
}
.ctx-pre {
  margin: 10px 0 0;
  padding: 12px;
  background: #0f172a;
  color: #cbd5e1;
  border-radius: 8px;
  font-size: 11.5px;
  line-height: 1.7;
  font-family: ui-monospace, 'SF Mono', Consolas, monospace;
  overflow-x: auto;
}

/* 启用/禁用选择器 */
.type-picker { display: flex; gap: 8px; }
.type-option {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border-radius: 10px;
  border: 1.5px solid var(--border);
  background: #fafafa;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-light);
  cursor: pointer;
  transition: all 0.2s;
}
.type-option:hover { border-color: var(--text-muted); }
.type-option.active { border-color: transparent; font-weight: 600; }
.pick-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #d1d5db;
  transition: background 0.2s;
}
.pick-enable.active { background: #ecfdf5; color: #10b981; }
.pick-enable.active .pick-dot { background: #10b981; }
.pick-disable.active { background: #f9fafb; color: #6b7280; }
.pick-disable.active .pick-dot { background: #6b7280; }

/* 弹窗底部 */
.modal-foot {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  padding: 16px 24px 20px;
  margin-top: 8px;
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

.modal-enter-active { transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1); }
.modal-leave-active { transition: all 0.2s ease; }
.modal-enter-from,
.modal-leave-to { opacity: 0; }
.modal-enter-from .modal-dialog,
.modal-leave-to .modal-dialog { transform: scale(0.92) translateY(20px); }
.modal-enter-active .modal-dialog,
.modal-leave-active .modal-dialog { transition: transform 0.3s cubic-bezier(0.16, 1, 0.3, 1); }

.card-enter-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.card-leave-active { transition: all 0.3s ease; }
.card-enter-from { opacity: 0; transform: translateY(20px); }
.card-leave-to { opacity: 0; transform: scale(0.9); }

/* ===== 响应式 ===== */
@media (max-width: 768px) {
  .page-header { flex-direction: column; }
  .page-desc { max-width: 100%; }
  .card-grid { grid-template-columns: 1fr; }
  .field-row { flex-direction: column; gap: 16px; }
}
</style>
