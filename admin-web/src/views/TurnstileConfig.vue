<template>
  <div class="turnstile-config-page">
    <!-- 页面头部动效 -->
    <Transition name="fade-down" appear>
      <div class="page-header">
        <div>
          <h2 class="page-title">审核设置</h2>
          <p class="page-desc">
            管理注册和找回密码的人机验证，以及昵称、头像、壁纸等内容审核策略。内容审核支持接入外部审核服务与内置违禁词库。
          </p>
        </div>
      </div>
    </Transition>

    <!-- 标签栏 -->
    <div class="tab-bar">
      <button class="tab-item" :class="{ active: activeTab === 'captcha' }" @click="activeTab = 'captcha'">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/><path d="M12 8v4"/><path d="M12 16h.01"/></svg>
        人机验证
      </button>
      <button class="tab-item" :class="{ active: activeTab === 'content' }" @click="activeTab = 'content'">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 12l2 2 4-4"/><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
        内容审核
      </button>
    </div>

    <!-- ==================== 人机验证 Tab ==================== -->
    <div v-if="activeTab === 'captcha'" class="tab-panel">
      <Transition name="fade-up" appear>
        <div v-if="loading" class="state-box"><span class="loader"></span> 加载中...</div>
        <div v-else class="config-card">
          <div class="section-title">基本设置</div>
          <div class="toggle-row">
            <div>
              <div class="toggle-label">启用人机验证</div>
              <div class="toggle-desc">开启后，注册和找回密码页面发送验证码前需完成所选服务商的人机验证</div>
            </div>
            <label class="switch">
              <input type="checkbox" v-model="form.enabled" />
              <span class="slider"></span>
            </label>
          </div>

          <div class="field-grid" style="margin-top: 20px">
            <div class="field">
              <span>验证服务商 <em class="required">*</em></span>
              <select v-model="form.provider" :disabled="saving">
                <option value="turnstile">Cloudflare Turnstile</option>
                <option value="hcaptcha">hCaptcha</option>
              </select>
              <small class="hint">切换后需填写对应平台的 Site Key 和 Secret Key</small>
            </div>
            <div class="field">
              <span>Site Key <em class="required">*</em></span>
              <input
                v-model="form.site_key"
                type="text"
                :placeholder="siteKeyPlaceholder"
                :disabled="saving"
              />
              <small class="hint">前端展示用，在所选验证服务商控制台获取</small>
            </div>
            <div class="field">
              <span>Secret Key</span>
              <input
                v-model="form.secret"
                type="password"
                :placeholder="secretPlaceholder"
                :disabled="saving"
              />
              <small class="hint">
                后端校验用{{ hasSecret ? '（已设置，留空保留原值）' : '（留空则回退环境变量 CAPTCHA_SECRET / 服务商专用 Secret）' }}
              </small>
            </div>
          </div>

          <div class="card-actions">
            <button class="btn-save" :disabled="saving" @click="save">
              <span v-if="saving" class="spinner"></span>
              {{ saving ? '保存中...' : '保存配置' }}
            </button>
          </div>
        </div>
      </Transition>

      <Transition name="fade-up" appear>
        <div class="config-card">
          <div class="section-title">使用说明</div>
          <ul class="tips-list">
            <li>Turnstile：在 Cloudflare 控制台创建 Turnstile widget，建议选择 "Managed" 模式。</li>
            <li>hCaptcha：在 hCaptcha 控制台创建 site，获取对应的 Site Key 和 Secret Key。</li>
            <li>勾选"启用人机验证"并保存后，注册页和找回密码页会自动加载验证组件。</li>
            <li>关闭开关后，前端不会显示验证组件，后端也不会校验 token。</li>
            <li>Secret Key 为空时会依次回退环境变量 <code>CAPTCHA_SECRET</code>、<code>TURNSTILE_SECRET</code> 或 <code>HCAPTCHA_SECRET</code>。</li>
            <li>配置保存后立即生效，无需重启服务端。</li>
          </ul>
        </div>
      </Transition>
    </div>

    <!-- ==================== 内容审核 Tab ==================== -->
    <div v-else class="tab-panel">
      <!-- 外部内容审核 -->
      <Transition name="fade-up" appear>
        <section class="audit-policy-card">
          <div class="policy-head">
            <div>
              <h3>外部内容审核</h3>
              <p>开启后，昵称、头像、壁纸会先走外部机审；机审无法判断或服务失败时，再进入人工审核队列。</p>
            </div>
            <label class="switch-line">
              <input v-model="auditConfig.enabled" type="checkbox" />
              <span>{{ auditConfig.enabled ? '已启用' : '未启用' }}</span>
            </label>
          </div>

          <div class="policy-grid">
            <label>
              <span :class="{ required: auditConfig.enabled }">服务类型</span>
              <select v-model="auditConfig.provider">
                <option value="generic">通用 HTTP</option>
                <option value="aliyun">阿里云内容安全</option>
                <option value="tencent">腾讯云内容安全</option>
              </select>
            </label>
            <label>
              <span :class="{ required: auditConfig.enabled }">审核接口地址</span>
              <input v-model.trim="auditConfig.endpoint" placeholder="https://example.com/audit" />
            </label>
            <label>
              <span>接口密钥</span>
              <input v-model.trim="auditConfig.api_key" type="password" placeholder="留空则保留原密钥" />
            </label>
            <label>
              <span :class="{ required: auditConfig.enabled }">超时时间</span>
              <input v-model.number="auditConfig.timeout_ms" type="number" min="1000" max="30000" />
            </label>
          </div>

          <div class="policy-options">
            <label><input v-model="auditConfig.nickname_enabled" type="checkbox" /> 改名机审</label>
            <label><input v-model="auditConfig.avatar_enabled" type="checkbox" /> 头像机审</label>
            <label><input v-model="auditConfig.wallpaper_enabled" type="checkbox" /> 壁纸机审</label>
            <label><input v-model="auditConfig.fail_to_manual" type="checkbox" /> 失败转人工</label>
          </div>

          <div class="policy-actions">
            <input v-model.trim="auditTestText" class="test-input" placeholder="测试文本，例如：测试昵称" />
            <button class="btn btn-primary btn-sm" @click="saveAuditConfig" :disabled="auditSaving">保存策略</button>
            <button class="btn btn-sm" @click="testAuditConfig" :disabled="auditTesting">测试连接</button>
          </div>
          <p class="policy-tip">
            通用 HTTP 服务需返回 <code>{"decision":"pass|reject|review","reason":"原因"}</code>。阿里云可先通过网关或函数计算适配为该格式。
          </p>
        </section>
      </Transition>

      <!-- 内置违禁词库 -->
      <Transition name="fade-up" appear>
        <section class="audit-policy-card">
          <div class="policy-head">
            <div>
              <h3>内置违禁词库</h3>
              <p>开启后，昵称等文本内容会先在本机匹配违禁词，命中直接拒绝；未命中再走外部审核或人工审核。</p>
            </div>
            <label class="switch-line">
              <input v-model="bannedWords.enabled" type="checkbox" />
              <span>{{ bannedWords.enabled ? '已启用' : '未启用' }}</span>
            </label>
          </div>

          <div class="banned-words-area">
            <textarea
              v-model="bannedWordsText"
              class="banned-words-textarea"
              rows="8"
              placeholder="每行一个违禁词，例如：&#10;违禁词1&#10;违禁词2"
            ></textarea>
            <div class="banned-words-meta">
              <span>共 {{ bannedWordCount }} 个词条</span>
              <span class="banned-words-hint">按行分隔，保存时自动去重、去除空行</span>
            </div>
          </div>

          <div class="policy-actions">
            <input v-model.trim="bannedTestText" class="test-input" placeholder="测试文本，例如：测试昵称" />
            <button class="btn btn-primary btn-sm" @click="saveBannedWords" :disabled="bannedSaving">保存词库</button>
            <button class="btn btn-sm" @click="testBannedWords" :disabled="bannedTesting">测试词库</button>
          </div>
          <p class="policy-tip">
            内置违禁词库在本机匹配，无需联网；命中后返回「拒绝」并记录命中词条。建议与外部内容审核搭配使用。
          </p>
        </section>
      </Transition>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'

const activeTab = ref<'captcha' | 'content'>('captcha')
const loading = ref(true)
const saving = ref(false)
const hasSecret = ref(false)

const form = ref({
  enabled: false,
  provider: 'turnstile',
  site_key: '',
  secret: '',
})

// ===== 外部内容审核 =====
const auditSaving = ref(false)
const auditTesting = ref(false)
const auditTestText = ref('测试昵称')
const auditConfig = ref({
  enabled: false,
  provider: 'generic',
  endpoint: '',
  api_key: '',
  nickname_enabled: true,
  avatar_enabled: true,
  wallpaper_enabled: true,
  timeout_ms: 5000,
  fail_to_manual: true,
})

// ===== 内置违禁词库 =====
const bannedSaving = ref(false)
const bannedTesting = ref(false)
const bannedTestText = ref('测试昵称')
const bannedWords = ref({ enabled: false, words: [] as string[] })
const bannedWordsText = ref('')

const bannedWordCount = computed(() =>
  bannedWordsText.value.split('\n').map(s => s.trim()).filter(Boolean).length
)

async function loadAuditConfig() {
  const res = await adminApi<any>('get_audit_external_config')
  if (res.code === 200 && res.data) {
    Object.assign(auditConfig.value, res.data, { api_key: '' })
  }
}

async function saveAuditConfig() {
  auditSaving.value = true
  const res = await adminApi<any>('save_audit_external_config', auditConfig.value as any)
  auditSaving.value = false
  if (res.code === 200) {
    auditConfig.value.api_key = ''
    showToast('外部审核策略已保存', 'success')
  } else {
    showToast(res.msg || '保存失败')
  }
}

async function testAuditConfig() {
  auditTesting.value = true
  const res = await adminApi<any>('test_audit_external_config', {
    text: auditTestText.value || '测试昵称',
  })
  auditTesting.value = false
  if (res.code === 200 && res.data) {
    const label = res.data.decision === 'pass' ? '通过' : res.data.decision === 'reject' ? '拒绝' : '转人工'
    showToast(`测试结果：${label}${res.data.reason ? '，' + res.data.reason : ''}`, 'success')
  } else {
    showToast(res.msg || '测试失败')
  }
}

async function loadBannedWords() {
  const res = await adminApi<any>('get_banned_words_config')
  if (res.code === 200 && res.data) {
    bannedWords.value.enabled = !!res.data.enabled
    bannedWords.value.words = res.data.words || []
    bannedWordsText.value = (res.data.words || []).join('\n')
  }
}

async function saveBannedWords() {
  bannedSaving.value = true
  const words = bannedWordsText.value.split('\n').map(s => s.trim()).filter(Boolean)
  const res = await adminApi<any>('save_banned_words_config', {
    enabled: bannedWords.value.enabled ? 1 : 0,
    words,
  })
  bannedSaving.value = false
  if (res.code === 200) {
    const saved = res.data?.words || words
    bannedWords.value.words = saved
    bannedWordsText.value = saved.join('\n')
    showToast('违禁词库已保存', 'success')
  } else {
    showToast(res.msg || '保存失败')
  }
}

async function testBannedWords() {
  bannedTesting.value = true
  const res = await adminApi<any>('test_banned_words', {
    text: bannedTestText.value || '测试昵称',
  })
  bannedTesting.value = false
  if (res.code === 200 && res.data) {
    const label = res.data.decision === 'pass' ? '通过' : '拒绝'
    showToast(`测试结果：${label}${res.data.reason ? '，' + res.data.reason : ''}`, 'success')
  } else {
    showToast(res.msg || '测试失败')
  }
}

const siteKeyPlaceholder = computed(() => form.value.provider === 'hcaptcha' ? '10000000-ffff-ffff-ffff-000000000001' : '0x4AAAAAAB...')
const secretPlaceholder = computed(() => form.value.provider === 'hcaptcha' ? '0x0000000000000000000000000000000000000000' : '0x4AAAAAAB...')

async function loadConfig() {
  loading.value = true
  const res = await adminApi('get_captcha_config')
  loading.value = false

  if (res.code === 200 && res.data) {
    form.value.enabled = res.data.enabled
    form.value.provider = res.data.provider || 'turnstile'
    form.value.site_key = res.data.site_key || ''
    form.value.secret = res.data.secret || ''
    hasSecret.value = res.data.has_secret
  } else {
    showToast(res.msg || '加载失败')
  }
}

async function save() {
  if (!form.value.site_key.trim() && form.value.enabled) {
    showToast('启用时必须填写 Site Key')
    return
  }
  saving.value = true
  const res = await adminApi('save_captcha_config', {
    enabled: form.value.enabled ? '1' : '0',
    provider: form.value.provider,
    site_key: form.value.site_key.trim(),
    secret: form.value.secret.trim(),
  })
  saving.value = false

  if (res.code === 200) {
    showToast(res.msg || '保存成功', 'success')
    if (form.value.secret && form.value.secret !== '********') {
      form.value.secret = '********'
      hasSecret.value = true
    }
  } else {
    showToast(res.msg || '保存失败')
  }
}

onMounted(() => {
  loadConfig()
  loadAuditConfig()
  loadBannedWords()
})
</script>

<style scoped>
.turnstile-config-page {
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
  max-width: 560px;
}
.page-desc a {
  color: var(--accent);
  text-decoration: none;
}
.page-desc a:hover {
  text-decoration: underline;
}

/* ===== 标签栏 ===== */
.tab-bar {
  display: flex;
  gap: 4px;
  border-bottom: 1px solid var(--border);
  margin-bottom: 20px;
  padding: 0 4px;
  overflow-x: auto;
}
.tab-item {
  position: relative;
  display: inline-flex;
  align-items: center;
  gap: 7px;
  padding: 11px 18px;
  border: none;
  background: transparent;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-muted);
  cursor: pointer;
  white-space: nowrap;
  border-radius: 10px 10px 0 0;
  transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}
.tab-item:hover { color: var(--accent); background: var(--accent-soft); }
.tab-item.active { color: var(--accent); background: var(--accent-soft); }
.tab-item.active::after {
  content: '';
  position: absolute;
  left: 10px;
  right: 10px;
  bottom: -1px;
  height: 3px;
  border-radius: 3px 3px 0 0;
  background: var(--accent);
}
.tab-panel {
  animation: fadeUp 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}
@keyframes fadeUp {
  from { opacity: 0; transform: translateY(8px); }
  to { opacity: 1; transform: translateY(0); }
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
  white-space: nowrap;
}
.btn-save:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 4px 14px rgba(0, 0, 0, 0.14);
}
.btn-save:disabled {
  opacity: 0.65;
  cursor: not-allowed;
}
.card-actions {
  display: flex;
  justify-content: flex-end;
  margin-top: 20px;
  padding-top: 16px;
  border-top: 1px solid var(--border);
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
  margin-bottom: 20px;
}
.section-title {
  font-size: 15px;
  font-weight: 800;
  color: var(--text);
  margin: 0 0 16px;
}
.toggle-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}
.toggle-label {
  font-size: 14px;
  font-weight: 600;
  color: var(--text);
}
.toggle-desc {
  font-size: 12px;
  color: var(--text-muted);
  margin-top: 4px;
  line-height: 1.5;
}
.switch {
  position: relative;
  display: inline-block;
  width: 48px;
  height: 26px;
  flex-shrink: 0;
}
.switch input {
  opacity: 0;
  width: 0;
  height: 0;
}
.slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: #ccc;
  border-radius: 26px;
  transition: 0.3s;
}
.slider::before {
  content: '';
  position: absolute;
  height: 20px;
  width: 20px;
  left: 3px;
  bottom: 3px;
  background: white;
  border-radius: 50%;
  transition: 0.3s;
}
.switch input:checked + .slider {
  background: var(--accent);
}
.switch input:checked + .slider::before {
  transform: translateX(22px);
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
.field span {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-light);
}
.field input,
.field select {
  padding: 10px 14px;
  border: 1px solid var(--border);
  border-radius: 10px;
  font-size: 14px;
  color: var(--text);
  outline: none;
  transition: all 0.2s;
  background: #fafafa;
}
.field input:focus,
.field select:focus {
  border-color: var(--accent);
  background: var(--card-solid);
  box-shadow: 0 0 0 3px rgba(var(--accent-rgb, 99, 102, 241), 0.08);
}
.field input:disabled,
.field select:disabled {
  opacity: 0.6;
}
.required {
  color: #e74c3c;
  font-style: normal;
}
.hint {
  font-size: 11px;
  color: var(--text-muted);
  line-height: 1.4;
}
.tips-list {
  margin: 0;
  padding-left: 18px;
  font-size: 13px;
  color: var(--text-muted);
  line-height: 2;
}
.tips-list code {
  background: rgba(0, 0, 0, 0.05);
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 12px;
  color: var(--text);
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

/* ===== 过渡动画 ===== */
.fade-down-enter-active, .fade-down-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-down-enter-from { opacity: 0; transform: translateY(-12px); }
.fade-up-enter-active, .fade-up-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-up-enter-from { opacity: 0; transform: translateY(12px); }
.config-card { animation: cardIn 0.4s cubic-bezier(0.16, 1, 0.3, 1) both; }
@keyframes cardIn { from { opacity: 0; transform: translateY(8px); } to { opacity: 1; transform: translateY(0); } }

/* ===== 外部内容审核 / 内置违禁词库 ===== */
.audit-policy-card {
  margin-bottom: 20px;
  padding: 18px;
  border-radius: 16px;
  border: 1px solid var(--border);
  background: var(--card-solid);
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.04);
}
.policy-head {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  align-items: flex-start;
  margin-bottom: 14px;
}
.policy-head h3 {
  margin: 0 0 6px;
  font-size: 16px;
}
.policy-head p,
.policy-tip {
  margin: 0;
  color: var(--text-muted);
  font-size: 13px;
  line-height: 1.7;
}
.switch-line {
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  gap: 7px;
  padding: 7px 12px;
  border-radius: 999px;
  background: var(--accent-soft);
  color: var(--accent);
  font-size: 12px;
  font-weight: 800;
  cursor: pointer;
}
.policy-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 12px;
}
.policy-grid label {
  display: flex;
  flex-direction: column;
  gap: 6px;
  font-size: 12px;
  color: var(--text-muted);
  font-weight: 700;
}
.policy-grid input,
.policy-grid select,
.test-input {
  height: 38px;
  border-radius: 11px;
  border: 1px solid var(--border);
  background: var(--control-bg);
  color: var(--text);
  padding: 0 12px;
  outline: none;
}
.policy-grid input:focus,
.policy-grid select:focus,
.test-input:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 4px var(--accent-soft);
}
.policy-options {
  display: flex;
  flex-wrap: wrap;
  gap: 10px 16px;
  margin-top: 14px;
  color: var(--text-muted);
  font-size: 13px;
}
.policy-options label {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}
.policy-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  margin-top: 14px;
}
.test-input {
  min-width: 240px;
}
.policy-tip {
  margin-top: 10px;
}
.policy-tip code {
  color: var(--accent);
}

/* ===== 内置违禁词库 ===== */
.banned-words-area {
  margin-top: 4px;
}
.banned-words-textarea {
  width: 100%;
  box-sizing: border-box;
  min-height: 160px;
  padding: 12px 14px;
  border-radius: 11px;
  border: 1px solid var(--border);
  background: var(--control-bg);
  color: var(--text);
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 13px;
  line-height: 1.7;
  resize: vertical;
  outline: none;
  transition: all 0.2s;
}
.banned-words-textarea:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 4px var(--accent-soft);
}
.banned-words-meta {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  margin-top: 8px;
  font-size: 12px;
  color: var(--text-muted);
}
.banned-words-hint {
  color: var(--text-muted);
  opacity: 0.85;
}
</style>
