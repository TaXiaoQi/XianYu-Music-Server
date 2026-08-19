<template>
  <div class="mobile-page">
    <!-- 页头 -->
    <div class="api-header">
      <div>
        <h2 class="api-title">接口测试</h2>
        <p class="api-desc">测试服务器 API 可用性，支持单接口测试与一键批量顺序测试，实时展示响应状态、返回信息与耗时。</p>
      </div>
    </div>

    <!-- 汇总卡片 -->
    <div v-if="mainHasResult" class="summary-grid">
      <div class="summary-card"><div class="summary-num">{{ mainSummary.total }}</div><div class="summary-label">接口总数</div></div>
      <div class="summary-card s-success"><div class="summary-num">{{ mainSummary.success }}</div><div class="summary-label">成功</div></div>
      <div class="summary-card s-failed"><div class="summary-num">{{ mainSummary.failed }}</div><div class="summary-label">失败</div></div>
      <div class="summary-card s-time"><div class="summary-num">{{ mainSummary.time }}<span class="summary-unit">ms</span></div><div class="summary-label">总耗时</div></div>
    </div>

    <!-- 接口列表 -->
    <section class="mobile-card api-card">
      <div class="api-card-head">
        <div>
          <h3 class="api-card-title">API 接口</h3>
          <span class="api-card-sub">共 {{ mainApis.length }} 个接口</span>
        </div>
        <div class="api-actions">
          <button class="mobile-btn primary run-btn" :disabled="mainRunning" @click="runAllMain">{{ mainRunning ? '测试中...' : '一键测试' }}</button>
          <button class="mobile-btn" :disabled="mainRunning || !mainHasResult" @click="clearMain">清空</button>
        </div>
      </div>

      <div v-if="mainResults.length === 0" class="mobile-empty">暂无接口</div>
      <div v-else class="api-list">
        <div v-for="(r, i) in mainResults" :key="'main-' + r.name" class="api-item" :class="rowClass(r)">
          <div class="api-item-top">
            <div class="api-name">
              {{ mainApis[i].name }}
              <span class="api-desc">{{ mainApis[i].desc }}</span>
            </div>
            <span v-if="r.status === 'testing'" class="st-tag testing">测试中</span>
            <span v-else-if="r.status === 'success'" class="st-tag success">成功</span>
            <span v-else-if="r.status === 'failed'" class="st-tag failed">失败</span>
            <span v-else class="st-tag pending">待测试</span>
          </div>
          <div class="api-item-bottom">
            <span class="api-msg" :class="r.status === 'success' ? 'msg-ok' : r.status === 'failed' ? 'msg-bad' : 'msg-idle'">
              {{ r.status === 'idle' ? '-' : r.status === 'testing' ? '请求中...' : r.message }}
            </span>
            <div class="api-op">
              <span v-if="r.status === 'success' || r.status === 'failed'" class="api-time">{{ r.duration }}ms</span>
              <button class="mini-btn" :disabled="mainRunning || r.status === 'testing'" @click="testOne(i)">单测</button>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- 手动调试 -->
    <section class="mobile-card mobile-form">
      <h3 class="mobile-card-title">手动调试</h3>
      <input v-model="action" class="mobile-input" placeholder="后台 action，例如 dashboard_stats" />
      <textarea v-model="body" class="mobile-textarea" placeholder='JSON 参数，例如 {"page":1}'></textarea>
      <button class="mobile-btn primary" :disabled="running" @click="run">发送测试</button>
    </section>
    <transition name="expand">
      <pre v-if="result" class="mobile-code">{{ result }}</pre>
    </transition>
  </div>
</template>
<script setup lang="ts">
import { ref, computed } from 'vue'
import { getToken, showToast, adminApi } from '@/api/client'
import './MobilePage.css'

interface ApiConfig {
  name: string
  method: 'GET' | 'POST'
  sign: boolean
  desc: string
  body: Record<string, any>
}
interface TestResult {
  name: string
  status: 'idle' | 'testing' | 'success' | 'failed'
  message: string
  duration: number
}

const mainApis: ApiConfig[] = [
  { name: 'check', method: 'GET', sign: false, desc: '服务器自检/数据库连接检查', body: {} },
  { name: 'install', method: 'GET', sign: false, desc: '数据库安装', body: {} },
  { name: 'get_source_status', method: 'GET', sign: false, desc: '获取音源状态', body: {} },
  { name: 'get_server_load', method: 'GET', sign: true, desc: '获取服务器负载', body: {} },
  { name: 'get_captcha', method: 'GET', sign: false, desc: '获取人机验证配置', body: {} },
  { name: 'get_user_agreement', method: 'GET', sign: false, desc: '获取用户协议', body: {} },
  { name: 'get_version_status', method: 'POST', sign: true, desc: '获取版本状态', body: { version: '1.0.0' } },
  { name: 'get_latest_version', method: 'POST', sign: true, desc: '获取最新版本', body: {} },
  { name: 'get_announcement', method: 'GET', sign: true, desc: '获取公告', body: {} },
  { name: 'get_about_config', method: 'GET', sign: true, desc: '获取关于页配置', body: {} },
  { name: 'get_site_logo', method: 'GET', sign: true, desc: '获取站点Logo', body: {} },
  { name: 'get_leaderboard', method: 'GET', sign: true, desc: '获取排行榜', body: {} },
  { name: 'register', method: 'POST', sign: true, desc: '用户注册', body: { username: 'test_check_user', password: 'test123456', email: 'test@example.com', verify_code: '000000' } },
  { name: 'user_login', method: 'POST', sign: true, desc: '用户登录', body: { username: 'test_nonexist', password: 'test123456' } },
  { name: 'login_by_code', method: 'POST', sign: true, desc: '验证码登录', body: { email: 'test@example.com', verify_code: '000000' } },
  { name: 'send_verify_code', method: 'POST', sign: true, desc: '发送验证码', body: { email: 'test@example.com', type: 'register' } },
  { name: 'reset_password', method: 'POST', sign: true, desc: '重置密码', body: { email: 'test@example.com', verify_code: '000000', new_password: 'test123456' } },
  { name: 'verify_captcha', method: 'POST', sign: false, desc: '验证人机验证', body: { token: 'test' } },
  { name: 'check_ban_status', method: 'POST', sign: true, desc: '检查封禁状态', body: { device_id: 'test_device_001' } },
  { name: 'delete_account', method: 'POST', sign: true, desc: '删除账号', body: { ciyuanxi_id: 'test', password: 'test123456' } },
  { name: 'get_user_info', method: 'POST', sign: true, desc: '获取用户信息', body: { ciyuanxi_id: 'test' } },
  { name: 'get_user_settings', method: 'POST', sign: true, desc: '获取用户设置', body: { ciyuanxi_id: 'test' } },
  { name: 'check_username', method: 'POST', sign: true, desc: '检查弦予号可用性', body: { username: 'test_check_user' } },
  { name: 'report_listen_stats', method: 'POST', sign: true, desc: '上报听歌统计', body: { ciyuanxi_id: 'test', listen_duration: 100 } },
  { name: 'submit_feedback', method: 'POST', sign: true, desc: '提交反馈', body: { ciyuanxi_id: 'test', content: '测试反馈', feedback_type: 'problem' } },
  { name: 'submit_appeal', method: 'POST', sign: true, desc: '提交申诉', body: { ciyuanxi_id: 'test', content: '测试申诉', device_id: 'test_device_001' } },
  { name: 'list_my_feedback', method: 'POST', sign: true, desc: '获取我的反馈列表', body: { ciyuanxi_id: 'test' } },
  { name: 'list_wallpapers', method: 'GET', sign: false, desc: '获取壁纸列表', body: {} },
  { name: 'error', method: 'POST', sign: true, desc: '错误上报', body: { device_id: 'test_device_001', app_version: '1.0.0', os_version: '13', device_model: 'TestDevice', error_type: 'TestError', error_message: '测试错误', error_stack: '', page: 'test' } },
  { name: 'open', method: 'POST', sign: false, desc: 'APP启动上报', body: { device_id: 'test_device_001', app_version: '1.0.0', os_version: '13', device_model: 'TestDevice' } },
  { name: 'email_get_captcha_config', method: 'GET', sign: false, desc: '获取验证码配置', body: {} },
  { name: 'email_get_turnstile_config', method: 'GET', sign: false, desc: '获取Turnstile配置', body: {} },
  { name: 'email_send_code', method: 'POST', sign: false, desc: '发送邮箱验证码', body: { email: 'test@example.com', type: 'register' } },
  { name: 'email_register', method: 'POST', sign: false, desc: '邮箱注册', body: { email: 'test@example.com', password: 'test123456', verify_code: '000000' } },
  { name: 'email_login', method: 'POST', sign: false, desc: '邮箱登录', body: { email: 'test@example.com', verify_code: '000000' } },
  { name: 'email_reset_password', method: 'POST', sign: false, desc: '邮箱重置密码', body: { email: 'test@example.com', verify_code: '000000', new_password: 'test123456' } },
  { name: 'email_get_profile', method: 'POST', sign: false, desc: '获取邮箱用户资料', body: { token: 'test' } },
]

const MAIN_EXPECTED: Record<string, number[]> = {
  user_login: [401],
  register: [400, 401],
  login_by_code: [400, 401],
  reset_password: [400],
  send_verify_code: [200, 429, 500],
  verify_captcha: [400, 401],
  check_username: [200, 400],
  get_user_info: [404],
  get_user_settings: [200, 404],
  submit_feedback: [400, 401],
  submit_appeal: [400, 401],
  upload_avatar: [400],
  delete_account: [400, 401],
  email_send_code: [200, 429, 500],
  email_register: [400, 401],
  email_login: [400, 401],
  email_reset_password: [400],
  email_get_profile: [401],
}

function makeIdle(apis: ApiConfig[]): TestResult[] {
  return apis.map(a => ({ name: a.name, status: 'idle', message: '', duration: 0 }))
}

const mainResults = ref<TestResult[]>(makeIdle(mainApis))
const mainRunning = ref(false)

function sumResults(results: TestResult[]) {
  let success = 0
  let failed = 0
  let time = 0
  for (const r of results) {
    if (r.status === 'success') success++
    else if (r.status === 'failed') failed++
    time += r.duration || 0
  }
  return { total: results.length, success, failed, time }
}
const mainSummary = computed(() => sumResults(mainResults.value))
const mainHasResult = computed(() => mainSummary.value.success + mainSummary.value.failed > 0)

function rowClass(r: TestResult) {
  return {
    'row-testing': r.status === 'testing',
    'row-success': r.status === 'success',
    'row-failed': r.status === 'failed',
  }
}

const ADMIN_API_BASE = '/admin/api'
async function callProxy(action: string, payload: Record<string, any>): Promise<any> {
  const token = getToken()
  const headers: Record<string, string> = { 'Content-Type': 'application/json' }
  if (token) headers['Authorization'] = `Bearer ${token}`
  const url = `${ADMIN_API_BASE}?action=${encodeURIComponent(action)}`
  try {
    const res = await fetch(url, { method: 'POST', headers, body: JSON.stringify(payload) })
    const text = await res.text()
    try {
      return JSON.parse(text)
    } catch {
      return { code: 500, msg: '返回非 JSON: ' + text.substring(0, 120), http_code: res.status }
    }
  } catch (e: any) {
    return { code: 500, msg: '网络错误：' + (e?.message || '请求失败'), http_code: 0 }
  }
}

function evaluateResult(api: ApiConfig, json: any, elapsed: number): TestResult {
  const httpCode = json.http_code || 0
  const apiCode = typeof json.code === 'number' ? json.code : -1
  const apiMsg = json.msg || ''
  const expected = MAIN_EXPECTED[api.name]
  const isExpectedError = !!expected && expected.includes(apiCode)
  if (apiCode === 200 || isExpectedError) {
    const dataInfo = json.data ? ' · data: ' + JSON.stringify(json.data).substring(0, 120) : ''
    return { name: api.name, status: 'success', message: `code=${apiCode} ${apiMsg}${dataInfo}`, duration: elapsed }
  }
  if (apiCode === 500 && httpCode === 0) {
    return { name: api.name, status: 'failed', message: `服务器连接失败: ${apiMsg}`, duration: elapsed }
  }
  return { name: api.name, status: 'failed', message: `code=${apiCode} ${apiMsg}`, duration: elapsed }
}

async function testOne(index: number) {
  const api = mainApis[index]
  if (mainResults.value[index].status === 'testing') return
  mainResults.value[index] = { name: api.name, status: 'testing', message: '', duration: 0 }
  const payload: Record<string, any> = {
    api_action: api.name,
    method: api.method,
    body: JSON.stringify(api.body || {}),
    need_sign: api.sign ? '1' : '0',
  }
  const start = Date.now()
  const json = await callProxy('proxy_api_test', payload)
  const elapsed = Date.now() - start
  mainResults.value[index] = evaluateResult(api, json, elapsed)
}

async function runAllMain() {
  if (mainRunning.value) return
  mainRunning.value = true
  mainResults.value = makeIdle(mainApis)
  for (let i = 0; i < mainApis.length; i++) {
    await testOne(i)
  }
  mainRunning.value = false
  const sum = mainSummary.value
  showToast(`接口测试完成：成功 ${sum.success}，失败 ${sum.failed}`, sum.failed > 0 ? 'error' : 'success')
}

function clearMain() {
  if (mainRunning.value) return
  mainResults.value = makeIdle(mainApis)
}

// ===== 手动调试 =====
const action = ref('dashboard_stats')
const body = ref('{}')
const result = ref('')
const running = ref(false)
async function run() {
  let data: any = {}
  try { data = JSON.parse(body.value || '{}') } catch { return showToast('JSON 参数格式错误') }
  running.value = true
  const res = await adminApi(action.value.trim(), data)
  result.value = JSON.stringify(res, null, 2)
  running.value = false
}
</script>
<style scoped>
.api-header { margin-bottom: 4px; }
.api-title { font-size: 18px; font-weight: 850; margin: 0 0 4px; color: var(--text); }
.api-desc { font-size: 12px; color: var(--text-light); line-height: 1.6; margin: 0; }

.summary-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}
.summary-card {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 12px;
  border-radius: 14px;
  background: var(--card);
  border: 1px solid var(--border);
  text-align: center;
}
.summary-num { font-size: 20px; font-weight: 850; line-height: 1.1; }
.summary-label { font-size: 11px; color: var(--text-muted); }
.summary-unit { font-size: 12px; font-weight: 600; color: var(--text-muted); margin-left: 2px; }
.s-success .summary-num { color: #16a34a; }
.s-failed .summary-num { color: #dc2626; }
.s-time .summary-num { color: #7c3aed; }

.api-card { padding: 16px; }
.api-card-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
  margin-bottom: 12px;
}
.api-card-title { margin: 0; font-size: 15px; }
.api-card-sub { font-size: 12px; color: var(--text-light); }
.api-actions { display: flex; gap: 8px; flex-shrink: 0; }
.run-btn { flex-shrink: 0; }

.api-list { display: flex; flex-direction: column; gap: 8px; }
.api-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 10px 12px;
  border-radius: 12px;
  background: var(--control-bg);
  border: 1px solid var(--border);
}
.api-item.row-failed { border-color: rgba(236, 65, 65, 0.25); background: rgba(254, 242, 242, 0.5); }
.api-item.row-success { border-color: #bbf7d0; background: rgba(240, 253, 244, 0.4); }
.api-item.row-testing { border-color: #ddd6fe; }
.api-item-top { display: flex; align-items: flex-start; justify-content: space-between; gap: 10px; }
.api-name {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
  font-size: 13px;
  font-weight: 800;
  color: var(--text);
  word-break: break-all;
}
.api-desc { font-size: 11px; font-weight: 400; color: var(--text-muted); }
.st-tag {
  flex-shrink: 0;
  font-size: 11px;
  font-weight: 700;
  padding: 3px 9px;
  border-radius: 20px;
}
.st-tag.success { background: #f0fdf4; color: #16a34a; }
.st-tag.failed { background: rgba(236, 65, 65, 0.12); color: #dc2626; }
.st-tag.pending { background: #f5f5f5; color: var(--text-muted); }
.st-tag.testing { background: #f5f3ff; color: #7c3aed; }
.api-item-bottom { display: flex; align-items: center; justify-content: space-between; gap: 10px; }
.api-msg {
  min-width: 0;
  flex: 1;
  font-size: 11px;
  line-height: 1.5;
  word-break: break-all;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
.msg-ok { color: #16a34a; }
.msg-bad { color: #dc2626; }
.msg-idle { color: var(--text-muted); }
.api-op { display: flex; align-items: center; gap: 8px; flex-shrink: 0; }
.api-time { font-size: 11px; color: var(--text-muted); }
.mini-btn {
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--card);
  color: var(--text);
  font-size: 11px;
  font-weight: 700;
  padding: 4px 10px;
  cursor: pointer;
}
.mini-btn:active { background: var(--accent-soft); }
.mini-btn:disabled { opacity: 0.5; }
</style>