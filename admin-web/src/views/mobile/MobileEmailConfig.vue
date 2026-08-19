<template>
  <div class="mobile-page">
    <!-- 页头 -->
    <div class="email-header">
      <div class="email-header-info">
        <h2 class="email-title">邮箱机设置</h2>
        <p class="email-desc">通用配置、外部 API、SMTP 账号池各可通过顶部开关启用；多开时按顺序尝试，发送成功即轮换。SMTP 地址与端口按发件邮箱域名自动识别。</p>
      </div>
      <button class="mobile-btn primary save-btn" :disabled="saving" @click="save">{{ saving ? '保存中...' : '保存' }}</button>
    </div>

    <!-- 通用配置 -->
    <section class="mobile-card">
      <div class="channel-head">
        <h3>通用配置</h3>
        <label class="switch">
          <input v-model="form.email_channel_general" type="checkbox" />
          <span class="track"><span class="thumb"></span></span>
        </label>
      </div>
      <input v-model.trim="form.email_sender" class="mobile-input" placeholder="发件邮箱地址" />
      <input v-model="form.email_password" class="mobile-input" type="password" :placeholder="hasPassword ? '********（留空保留原值）' : '邮箱密码 / 授权码'" />
      <div class="endpoint">
        <span>SMTP 服务器（自动识别）</span>
        <code>{{ smtpEndpointText(form.email_sender) }}</code>
      </div>
    </section>

    <!-- 外部 API -->
    <section class="mobile-card">
      <div class="channel-head">
        <h3>外部 API</h3>
        <label class="switch">
          <input v-model="form.email_channel_api" type="checkbox" />
          <span class="track"><span class="thumb"></span></span>
        </label>
      </div>
      <input v-model.trim="form.email_api_primary" class="mobile-input" placeholder="主地址，如 https://mail-api/send" />
      <input v-model.trim="form.email_api_backup" class="mobile-input" placeholder="备用地址（主地址失败时回退）" />
    </section>

    <!-- SMTP 账号池 -->
    <section class="mobile-card">
      <div class="channel-head">
        <h3>SMTP 账号池</h3>
        <label class="switch">
          <input v-model="form.email_channel_pool" type="checkbox" />
          <span class="track"><span class="thumb"></span></span>
        </label>
      </div>
      <p class="pool-tip">启用后每个可用账号作为独立投递通道轮流发送，降低单个邮箱触发风控的概率。SMTP 地址与端口按域名自动识别。</p>
      <button class="mobile-btn add-btn" @click="addSmtpAccount">+ 添加邮箱</button>
      <div v-if="form.smtp_accounts.length === 0" class="empty-pool">暂未配置账号池，添加后每条账号将作为独立投递通道参与轮换。</div>
      <div v-else class="account-list">
        <div v-for="(acc, index) in form.smtp_accounts" :key="index" class="account-card">
          <div class="account-head">
            <strong>发件账号 {{ index + 1 }}</strong>
            <label class="enabled-line"><input v-model="acc.enabled" type="checkbox" /> 启用</label>
            <button class="remove-btn" @click="removeSmtpAccount(index)">删除</button>
          </div>
          <input v-model.trim="acc.sender" class="mobile-input" placeholder="发件邮箱" @input="syncAccountEndpoint(acc)" />
          <input v-model.trim="acc.remark" class="mobile-input" placeholder="备注，如：QQ邮箱一号" />
          <input v-model.trim="acc.username" class="mobile-input" placeholder="SMTP 用户名（留空用发件邮箱）" />
          <input v-model="acc.password" class="mobile-input" type="password" :placeholder="acc.has_password ? '********（留空保留原值）' : '授权码'" />
          <div class="endpoint">
            <span>SMTP 服务器</span>
            <code>{{ smtpEndpointText(acc.sender) }}</code>
          </div>
        </div>
      </div>
    </section>

    <!-- 发送测试 -->
    <section class="mobile-card">
      <h3 class="mobile-card-title">发送测试邮件</h3>
      <input v-model="testEmail" class="mobile-input" placeholder="测试收件邮箱" @keydown.enter="test" />
      <button class="mobile-btn primary" :disabled="testing" @click="test">{{ testing ? '发送中...' : '发送测试' }}</button>
      <p v-if="testResult" :class="['test-result', testResultType]">{{ testResult }}</p>
    </section>
  </div>
</template>
<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { adminApi, showToast } from '@/api/client'
import './MobilePage.css'

interface SmtpAccount {
  sender: string
  host: string
  port: string | number
  username: string
  password: string
  has_password?: boolean
  enabled: boolean
  remark: string
}

const SMTP_ENDPOINTS: Record<string, [string, number]> = {
  'qq.com': ['smtp.qq.com', 465],
  'vip.qq.com': ['smtp.qq.com', 465],
  'foxmail.com': ['smtp.qq.com', 465],
  'exmail.qq.com': ['smtp.exmail.qq.com', 465],
  '163.com': ['smtp.163.com', 465],
  '126.com': ['smtp.126.com', 465],
  'yeah.net': ['smtp.yeah.net', 465],
  '188.com': ['smtp.188.com', 465],
  'qiye.163.com': ['smtp.qiye.163.com', 465],
  '139.com': ['smtp.139.com', 465],
  '21cn.com': ['smtp.21cn.com', 465],
  'sohu.com': ['smtp.sohu.com', 465],
  'sina.com': ['smtp.sina.com', 465],
  'vip.sina.com': ['smtp.sina.com', 465],
  'aliyun.com': ['smtp.aliyun.com', 465],
  'aliyunmail.com': ['smtp.aliyun.com', 465],
  'qiye.aliyun.com': ['smtp.qiye.aliyun.com', 465],
  '189.cn': ['smtp.189.cn', 465],
  '263.net': ['smtp.263.net', 465],
  'x263.net': ['smtp.263.net', 465],
  'gmail.com': ['smtp.gmail.com', 465],
  'outlook.com': ['smtp.office365.com', 587],
  'hotmail.com': ['smtp.office365.com', 587],
  'hotmail.co.uk': ['smtp.office365.com', 587],
  'live.com': ['smtp.office365.com', 587],
  'msn.com': ['smtp.office365.com', 587],
  'yandex.com': ['smtp.yandex.ru', 465],
  'yandex.ru': ['smtp.yandex.ru', 465],
  'zoho.com': ['smtp.zoho.com', 465],
  'zohomail.com': ['smtp.zoho.com', 465],
}

function smtpEndpoint(sender: string): { host: string; port: number } {
  const domain = (sender.split('@')[1] || '').trim().toLowerCase()
  if (!domain) return { host: '', port: 465 }
  const hit = SMTP_ENDPOINTS[domain]
  if (hit) return { host: hit[0], port: hit[1] }
  return { host: `smtp.${domain}`, port: 465 }
}

function smtpEndpointText(sender: string): string {
  const { host, port } = smtpEndpoint(sender)
  return host ? `${host}:${port}` : '请先填写发件邮箱'
}

function syncAccountEndpoint(account: SmtpAccount) {
  const { host, port } = smtpEndpoint(account.sender)
  account.host = host
  account.port = port
}

const saving = ref(false)
const testing = ref(false)
const testEmail = ref('')
const hasPassword = ref(false)
const testResult = ref('')
const testResultType = ref<'success' | 'error'>('success')
const form = ref<any>({
  email_channel_general: true,
  email_channel_api: false,
  email_channel_pool: true,
  email_api_primary: '',
  email_api_backup: '',
  email_sender: '',
  email_password: '',
  smtp_host: '',
  smtp_port: '465',
  smtp_username: '',
  smtp_password: '',
  smtp_accounts: [] as SmtpAccount[],
})

async function load() {
  const res = await adminApi<any>('get_email_config')
  if (res.code === 200 && res.data) {
    form.value = {
      email_channel_general: res.data.email_channel_general !== false,
      email_channel_api: res.data.email_channel_api === true,
      email_channel_pool: res.data.email_channel_pool !== false,
      email_api_primary: res.data.email_api_primary || '',
      email_api_backup: res.data.email_api_backup || '',
      email_sender: res.data.email_sender || '',
      email_password: res.data.has_password ? '********' : '',
      smtp_host: res.data.smtp_host || '',
      smtp_port: String(res.data.smtp_port || 465),
      smtp_username: res.data.smtp_username || '',
      smtp_password: res.data.has_smtp_password ? '********' : '',
      smtp_accounts: (res.data.smtp_accounts || []).map((a: any) => ({
        sender: a.sender || '',
        host: a.host || '',
        port: String(a.port || 465),
        username: a.username || '',
        password: a.has_password ? '********' : '',
        has_password: a.has_password,
        enabled: a.enabled !== false,
        remark: a.remark || '',
      })),
    }
    hasPassword.value = res.data.has_password
  } else {
    showToast(res.msg || '加载配置失败')
  }
}

function addSmtpAccount() {
  const { host, port } = smtpEndpoint(form.value.email_sender)
  form.value.smtp_accounts.push({ sender: '', host, port, username: '', password: '', enabled: true, remark: '' })
}

function removeSmtpAccount(index: number) {
  form.value.smtp_accounts.splice(index, 1)
}

async function save() {
  saving.value = true
  const payload = {
    ...form.value,
    smtp_accounts: form.value.smtp_accounts.map((a: any) => ({ ...a, port: Number(a.port || 465) })),
  }
  const res = await adminApi('update_email_config', payload)
  saving.value = false
  if (res.code === 200) { showToast('邮箱机配置已保存', 'success'); await load() }
  else showToast(res.msg || '保存失败')
}

async function test() {
  if (!testEmail.value.trim()) return showToast('请输入测试收件邮箱')
  testing.value = true
  testResult.value = ''
  const res = await adminApi('test_email_config', { email: testEmail.value.trim() })
  testing.value = false
  if (res.code === 200) {
    testResult.value = res.msg || '测试邮件已发送'
    testResultType.value = 'success'
  } else {
    testResult.value = res.msg || '发送失败'
    testResultType.value = 'error'
  }
}

onMounted(load)
</script>
<style scoped>
.email-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}
.email-header-info { min-width: 0; }
.email-title { font-size: 18px; font-weight: 850; margin: 0 0 4px; color: var(--text); }
.email-desc { font-size: 12px; color: var(--text-light); line-height: 1.6; margin: 0; }
.save-btn { flex-shrink: 0; }

.channel-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  margin-bottom: 12px;
}
.channel-head h3 { margin: 0; font-size: 15px; }
.switch { position: relative; display: inline-flex; flex-shrink: 0; cursor: pointer; }
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

.endpoint {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  border: 1.5px dashed var(--border);
  border-radius: 12px;
  background: var(--control-bg);
  margin-top: 10px;
}
.endpoint span { font-size: 11px; font-weight: 600; color: var(--text-muted); white-space: nowrap; }
.endpoint code { font-size: 12px; font-weight: 700; color: #EC4141; overflow-wrap: anywhere; }

.pool-tip { font-size: 12px; color: var(--text-light); line-height: 1.6; margin: 0 0 12px; }
.add-btn { width: 100%; }
.empty-pool {
  border: 1px dashed var(--border);
  border-radius: 12px;
  color: var(--text-muted);
  font-size: 12px;
  padding: 16px;
  margin-top: 12px;
  text-align: center;
}
.account-list { display: flex; flex-direction: column; gap: 12px; margin-top: 12px; }
.account-card {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 14px;
  border-radius: 14px;
  background: var(--control-bg);
  border: 1px solid var(--border);
}
.account-head { display: flex; align-items: center; gap: 8px; }
.account-head strong { font-size: 13px; color: var(--text); margin-right: auto; }
.enabled-line { display: inline-flex; align-items: center; gap: 5px; font-size: 12px; color: var(--text-light); font-weight: 700; cursor: pointer; }
.enabled-line input { accent-color: #EC4141; }
.remove-btn {
  border: 1px solid #dc2626;
  border-radius: 8px;
  background: transparent;
  color: #dc2626;
  font-size: 12px;
  font-weight: 700;
  padding: 5px 10px;
  cursor: pointer;
}
.remove-btn:active { background: #dc2626; color: #fff; }

.test-result { margin: 10px 0 0; font-size: 12px; font-weight: 600; }
.test-result.success { color: #16a34a; }
.test-result.error { color: #dc2626; }
</style>