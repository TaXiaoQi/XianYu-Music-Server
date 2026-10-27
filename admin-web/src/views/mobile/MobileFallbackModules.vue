<template>
  <div class="mobile-page">
    <!-- 页头 -->
    <div class="fb-header">
      <div class="fb-header-info">
        <h2 class="fb-title">兜底配置</h2>
        <p class="fb-desc">服务端只下发签名配置（链路开关 / 平台来源），不含任何可执行代码；客户端全部逻辑为内置原生实现，配置验签通过后在下一次拉取后生效。</p>
      </div>
    </div>

    <!-- 统计 -->
    <div class="mobile-grid" style="grid-template-columns: repeat(3, minmax(0, 1fr));">
      <div class="mobile-stat">
        <div class="stat-icon-row">
          <span class="stat-icon stat-icon-total">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg>
          </span>
          <strong>{{ list.length }}</strong>
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
          <strong>{{ disabledMethodCount }}</strong>
        </div>
        <span class="stat-label">关闭链路</span>
      </div>
    </div>

    <!-- 下发总开关 -->
    <div class="fb-card fb-global">
      <div class="fb-body" style="padding-top: 12px; padding-bottom: 12px;">
        <div class="fb-top">
          <div class="fb-title-wrap">
            <h3 class="fb-name">下发总开关</h3>
            <span class="fb-meta">关闭后客户端拉取到空配置，全部回退默认行为（配置本体保留）</span>
          </div>
          <label class="switch" :class="{ 'switch-off': configVersion === 0 }"
                 :title="configVersion === 0 ? '尚未保存任何模块配置' : ''">
            <input
              :checked="deliveryEnabled"
              :disabled="configVersion === 0 || toggling"
              type="checkbox"
              @change="toggleDelivery(($event.target as HTMLInputElement).checked)"
            />
            <span class="track"><span class="thumb"></span></span>
          </label>
        </div>
        <div v-if="configVersion > 0" class="fb-meta">
          <span>配置 v{{ configVersion }} · 签名 {{ shortSig(signature) }}</span>
          <span v-if="configKeyId">密钥 {{ configKeyId }}</span>
        </div>
      </div>
    </div>

    <!-- 签名密钥 -->
    <div class="fb-card fb-global">
      <div class="fb-body" style="padding-top: 12px; padding-bottom: 12px;">
        <div class="fb-top">
          <div class="fb-title-wrap">
            <h3 class="fb-name">签名密钥</h3>
            <span class="fb-meta">配置/内测响应的 ed25519 签名密钥；客户端内嵌双公钥槽位，轮换先预发布新公钥、再切换签发、最后清理旧槽位</span>
          </div>
          <span class="fb-badge" :class="signerBadgeClass">{{ signerModeLabel }}</span>
        </div>
        <div v-if="signerStatus" class="fb-meta">
          <template v-if="signerStatus.mode === 'signer'">
            <span>当前签发：{{ signerStatus.active || '-' }}</span>
            <span>过渡旧钥：{{ signerStatus.previous || '无' }}</span>
            <span v-if="signerStatus.activePublicKey" class="fb-key">{{ shortSig(signerStatus.activePublicKey) }}</span>
          </template>
          <template v-else-if="signerStatus.mode === 'local'">
            <span>A 档本机密钥（FALLBACK_SIGN_PRIVATE_KEY / api/fallback_sign_key.txt），无槽位管理</span>
            <span v-if="signerStatus.activePublicKey" class="fb-key">{{ shortSig(signerStatus.activePublicKey) }}</span>
          </template>
          <template v-else>
            <span>未配置签名密钥：保存兜底配置将失败；客户端拉取不受影响</span>
          </template>
        </div>
        <div v-if="signerStatus?.signerError" class="fb-meta">
          <span class="fb-signer-error">签发服务不可达：{{ signerStatus.signerError }}</span>
        </div>
        <div v-if="signerStatus?.mode === 'signer' && (signerStatus.available?.length ?? 0) > 0" class="fb-footer" style="border-top: none; padding-top: 4px;">
          <span class="fb-size">可用槽位：{{ signerStatus!.available!.join('、') }}</span>
          <div class="fb-actions">
            <button
              v-for="kid in signerStatus!.available!"
              :key="kid"
              class="mobile-btn"
              :disabled="kid === signerStatus!.active"
              @click="switchSignerKey(kid)"
            >{{ kid === signerStatus!.active ? '当前' : `切换到 ${kid}` }}</button>
          </div>
        </div>
      </div>
    </div>

    <!-- 加载中 -->
    <div v-if="loading" class="mobile-empty">加载中...</div>

    <!-- 模块列表 -->
    <div v-else class="fb-list">
      <div
        v-for="(item, idx) in list"
        :key="item.moduleKey"
        class="fb-card"
        :class="{ 'is-off': !item.configured }"
        :style="{ animationDelay: (idx * 0.05) + 's' }"
      >
        <div class="fb-body">
          <div class="fb-top">
            <div class="fb-title-wrap">
              <h3 class="fb-name">{{ item.label }}</h3>
              <span class="fb-key">{{ item.moduleKey }}</span>
            </div>
            <span v-if="item.configured" class="fb-badge">v{{ item.version }}</span>
            <span v-else class="fb-badge fb-badge-muted">默认</span>
          </div>

          <div v-if="item.configured" class="fb-meta">
            <span>{{ fmtDateTime(item.updated_at) || '-' }}</span>
            <span v-if="item.disabledMethods.length">关闭 {{ item.disabledMethods.length }} 条链路</span>
            <span v-if="item.sources.length">来源：{{ item.sources.join(' → ') }}</span>
            <span v-if="item.remark">{{ item.remark }}</span>
          </div>
          <div v-else class="fb-meta fb-builtin">默认行为（全部链路开启）</div>

          <div class="fb-footer">
            <span class="fb-size">{{ methodSummary(item) }}</span>
            <div class="fb-actions">
              <button class="mobile-btn" @click="openEditModal(item)">{{ item.configured ? '编辑' : '配置' }}</button>
              <button v-if="item.configured" class="mobile-btn danger" @click="reset(item)">恢复默认</button>
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
            <h3>{{ form.configured ? '编辑兜底配置' : '配置兜底行为' }}</h3>
            <button class="modal-close" @click="closeModal">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
          <div class="modal-body">
            <div class="modal-field">
              <span>模块</span>
              <div class="fb-module-line">{{ form.moduleKey }} · {{ form.label }}</div>
            </div>
            <div class="modal-field">
              <span>关闭链路 <em class="optional">（勾选后客户端该行为直接返回空，不再走内置实现）</em></span>
              <div class="fb-method-grid">
                <label v-for="m in form.methods" :key="m" class="fb-check">
                  <input v-model="form.disabledMethods" type="checkbox" :value="m" />
                  <span>{{ m }}</span>
                </label>
              </div>
            </div>
            <div v-if="form.sourcesSelectable" class="modal-field">
              <span>平台来源 <em class="optional">（点选决定顺序；留空 = 默认全部）</em></span>
              <div class="type-picker">
                <button
                  v-for="p in sourcePlatforms"
                  :key="p"
                  class="type-option"
                  :class="{ active: form.sources.includes(p) }"
                  @click="toggleSource(p)"
                ><span class="pick-dot"></span>{{ p }}</button>
              </div>
              <p v-if="form.sources.length" class="fb-hint">生效顺序：{{ form.sources.join(' → ') }}</p>
            </div>
            <label class="modal-field">
              <span>备注 <em class="optional">（可空）</em></span>
              <input v-model="form.remark" type="text" placeholder="如：酷我榜单接口异常，临时关闭" />
            </label>
            <p class="fb-hint">保存后配置版本号 +1 并整体重新签名，客户端下次拉取（启动时/每 30 分钟）后生效。</p>
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

interface FallbackModuleItem {
  moduleKey: string
  label: string
  methods: string[]
  sourcesSelectable: boolean
  configured: boolean
  version: number
  disabledMethods: string[]
  sources: string[]
  remark: string
  updated_at: string
}

const loading = ref(true)
const saving = ref(false)
const toggling = ref(false)
const deliveryEnabled = ref(false)
const configVersion = ref(0)
const signature = ref('')
const configKeyId = ref('')
const sourcePlatforms = ref<string[]>([])
const list = ref<FallbackModuleItem[]>([])

interface SignerStatus {
  mode: 'signer' | 'local' | 'none'
  active?: string
  previous?: string | null
  available?: string[]
  activePublicKey?: string
  signerError?: string
}
const signerStatus = ref<SignerStatus | null>(null)

const signerModeLabel = computed(() => {
  switch (signerStatus.value?.mode) {
    case 'signer': return '托管签发'
    case 'local': return '本机密钥'
    default: return '未配置'
  }
})
const signerBadgeClass = computed(() => {
  if (signerStatus.value?.mode === 'signer') {
    return signerStatus.value.signerError ? 'fb-badge-muted' : ''
  }
  return 'fb-badge-muted'
})

const configuredCount = computed(() => list.value.filter(m => m.configured).length)
const disabledMethodCount = computed(
  () => list.value.reduce((n, m) => n + (m.configured ? m.disabledMethods.length : 0), 0),
)

async function loadList() {
  loading.value = true
  const res = await adminApi<{
    enabled: boolean
    configVersion: number
    signature: string
    configKeyId?: string
    sourcePlatforms: string[]
    list: FallbackModuleItem[]
  }>('get_fallback_config')
  if (res.code === 200 && res.data) {
    deliveryEnabled.value = !!res.data.enabled
    configVersion.value = res.data.configVersion || 0
    signature.value = res.data.signature || ''
    configKeyId.value = res.data.configKeyId || ''
    sourcePlatforms.value = Array.isArray(res.data.sourcePlatforms) ? res.data.sourcePlatforms : []
    list.value = Array.isArray(res.data.list) ? res.data.list : []
  } else {
    list.value = []
    showToast(res.msg || '加载失败')
  }
  loading.value = false
  loadSignerStatus()
}

async function loadSignerStatus() {
  const res = await adminApi<SignerStatus>('get_fallback_signer_status')
  if (res.code === 200 && res.data) {
    signerStatus.value = res.data
  } else {
    signerStatus.value = { mode: 'none', signerError: res.msg }
  }
}

async function switchSignerKey(keyId: string) {
  const okToSwitch = await mobileConfirm(
    `确认将签发密钥切换到「${keyId}」？切换后新保存的配置将以新密钥签发（旧密钥移入过渡槽位继续可用）。请确认已在客户端预发布该公钥。`,
    { title: '切换签名密钥', confirmText: '确认切换', danger: true },
  )
  if (!okToSwitch) return
  const res = await adminApi<SignerStatus>('switch_fallback_signer_key', { key_id: keyId })
  if (res.code === 200) {
    showToast(res.msg || '已切换', 'success')
    if (res.data) signerStatus.value = res.data
    loadList()
  } else {
    showToast(res.msg || '切换失败')
  }
}

function shortSig(sig: string): string {
  if (!sig) return '-'
  return sig.length > 12 ? `${sig.slice(0, 8)}…${sig.slice(-4)}` : sig
}

function methodSummary(item: FallbackModuleItem): string {
  if (!item.configured) return `${item.methods.length} 条链路 · 内置实现`
  const off = item.disabledMethods.length
  return `${item.methods.length - off}/${item.methods.length} 条链路开启`
}

async function toggleDelivery(enabled: boolean) {
  toggling.value = true
  const res = await adminApi('toggle_fallback_config', { enabled: enabled ? 1 : 0 })
  toggling.value = false
  if (res.code === 200) {
    showToast(res.msg || (enabled ? '已开启下发' : '已暂停下发'), 'success')
    loadList()
  } else {
    showToast(res.msg || '操作失败')
    loadList()
  }
}

async function reset(item: FallbackModuleItem) {
  const ok = await mobileConfirm(`确认将「${item.label}」恢复默认？客户端将在下次拉取后生效。`, {
    title: '恢复默认',
    confirmText: '确认恢复',
    danger: true,
  })
  if (!ok) return
  const res = await adminApi('delete_fallback_module', { module_key: item.moduleKey })
  if (res.code === 200) {
    showToast(res.msg || '已恢复默认', 'success')
    loadList()
  } else {
    showToast(res.msg || '操作失败')
  }
}

// ===== 弹窗 =====
const modalVisible = ref(false)
const form = ref({
  moduleKey: '',
  label: '',
  methods: [] as string[],
  sourcesSelectable: false,
  configured: false,
  disabledMethods: [] as string[],
  sources: [] as string[],
  remark: '',
})

function openEditModal(item: FallbackModuleItem) {
  form.value = {
    moduleKey: item.moduleKey,
    label: item.label,
    methods: item.methods,
    sourcesSelectable: item.sourcesSelectable,
    configured: item.configured,
    disabledMethods: [...(item.disabledMethods || [])],
    sources: [...(item.sources || [])],
    remark: item.remark || '',
  }
  modalVisible.value = true
}

function closeModal() {
  if (saving.value) return
  modalVisible.value = false
}

function toggleSource(p: string) {
  const arr = form.value.sources
  const idx = arr.indexOf(p)
  if (idx >= 0) arr.splice(idx, 1)
  else arr.push(p)
}

async function save() {
  saving.value = true
  const res = await adminApi('save_fallback_module', {
    module_key: form.value.moduleKey,
    disabled_methods: form.value.disabledMethods,
    sources: form.value.sourcesSelectable ? form.value.sources : [],
    remark: form.value.remark.trim(),
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

.fb-global { margin-bottom: 4px; }
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
.fb-badge {
  flex-shrink: 0;
  padding: 2px 8px;
  border-radius: 6px;
  font-size: 11px;
  font-weight: 700;
  background: #ecfdf5;
  color: #10b981;
}
.fb-badge-muted { background: var(--track); color: var(--text-muted); font-weight: 500; }

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
.fb-signer-error { color: #ef4444; }

.fb-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 10px;
  margin-top: auto;
  padding-top: 10px;
  border-top: 1px solid var(--border);
}
.fb-size { font-size: 11px; color: var(--text-muted); }
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
.modal-field input[type='text'] {
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 11px 12px;
  font-size: 14px;
  font-family: inherit;
  outline: none;
  background: var(--control-bg);
  color: var(--text);
  width: 100%;
  box-sizing: border-box;
}
.modal-field input[type='text']:focus { border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-soft); }

.fb-module-line {
  font-size: 12.5px;
  color: var(--text-light);
  font-family: ui-monospace, 'SF Mono', Consolas, monospace;
  background: var(--control-bg);
  border-radius: 10px;
  padding: 10px 12px;
}
.fb-method-grid { display: flex; flex-direction: column; gap: 6px; }
.fb-check {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 9px 12px;
  border: 1px solid var(--border);
  border-radius: 10px;
  font-size: 12.5px;
  font-family: ui-monospace, 'SF Mono', Consolas, monospace;
  color: var(--text-light);
  cursor: pointer;
  transition: all 0.15s;
}
.fb-check input { accent-color: #ef4444; }
.fb-check:has(input:checked) { border-color: #fca5a5; color: var(--text); background: #fef2f2; }
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
.type-option.active { background: #ecfdf5; color: #10b981; border-color: transparent; }
.type-option.active .pick-dot { background: #10b981; }

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
.modal-btn.danger { background: #fef2f2; color: #ef4444; }

.modal-enter-active { transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1); }
.modal-leave-active { transition: all 0.2s ease; }
.modal-enter-from,
.modal-leave-to { opacity: 0; }
.modal-enter-from .modal-dialog,
.modal-leave-to .modal-dialog { transform: scale(0.92) translateY(20px); }
</style>
