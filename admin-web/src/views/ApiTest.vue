<template>
  <div class="api-test-page">
    <!-- 页面头部 -->
    <Transition name="fade-down" appear>
      <div class="page-header">
        <div class="header-info">
          <h2 class="page-title">接口测试</h2>
          <p class="page-desc">
            在线测试服务器所有 API 接口的可用性，支持单接口测试与一键批量顺序测试，实时展示响应状态、返回信息与耗时。
          </p>
        </div>
      </div>
    </Transition>

    <!-- 主应用接口 -->
    <Transition name="fade-up" appear>
      <section class="card">
        <div class="card-head">
          <div class="card-head-left">
            <h3 class="card-title">API 接口</h3>
            <span class="card-sub">proxy_api_test · 共 {{ mainApis.length }} 个接口</span>
          </div>
          <div class="card-actions">
            <button class="btn btn-primary" :disabled="mainRunning" @click="runAllMain">
              <span v-if="mainRunning" class="spinner-sm"></span>
              <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polygon points="5 3 19 12 5 21 5 3"/></svg>
              {{ mainRunning ? '测试中...' : '一键测试全部' }}
            </button>
            <button class="btn btn-outline" :disabled="mainRunning" @click="clearMain">清空结果</button>
          </div>
        </div>

        <!-- 汇总卡片 -->
        <Transition name="fade-up">
          <div v-if="mainHasResult" class="summary-row">
            <div class="summary-card">
              <div class="summary-num">{{ mainSummary.total }}</div>
              <div class="summary-label">接口总数</div>
            </div>
            <div class="summary-card summary-success">
              <div class="summary-num">{{ mainSummary.success }}</div>
              <div class="summary-label">成功</div>
            </div>
            <div class="summary-card summary-failed">
              <div class="summary-num">{{ mainSummary.failed }}</div>
              <div class="summary-label">失败</div>
            </div>
            <div class="summary-card summary-time">
              <div class="summary-num">{{ mainSummary.time }}<span class="summary-unit">ms</span></div>
              <div class="summary-label">总耗时</div>
            </div>
          </div>
        </Transition>

        <!-- 结果表格 -->
        <div class="table-wrap">
          <table class="data-table">
            <thead>
              <tr>
                <th class="col-idx">#</th>
                <th>接口名称</th>
                <th class="col-status">状态</th>
                <th class="col-msg">响应信息</th>
                <th class="col-time">耗时</th>
                <th class="col-op">操作</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="(r, i) in mainResults"
                :key="'main-' + r.name"
                :class="rowClass(r)"
                class="row-anim"
                :style="{ animationDelay: `${i * 40}ms` }"
              >
                <td class="col-idx">{{ i + 1 }}</td>
                <td>
                  <div class="api-name">{{ mainApis[i].name }}</div>
                  <div class="api-desc">{{ mainApis[i].desc }}</div>
                </td>
                <td class="col-status">
                  <span v-if="r.status === 'testing'" class="testing-tag"><span class="spinner-sm"></span>测试中</span>
                  <span v-else-if="r.status === 'success'" class="badge badge-success">成功</span>
                  <span v-else-if="r.status === 'failed'" class="badge badge-error">失败</span>
                  <span v-else class="badge badge-pending">待测试</span>
                </td>
                <td class="col-msg">
                  <span v-if="r.status === 'testing'" class="msg-pending">请求中...</span>
                  <span v-else-if="r.status === 'idle'" class="msg-empty">-</span>
                  <span v-else class="msg-text" :class="r.status === 'success' ? 'msg-success' : 'msg-failed'" :title="r.message">{{ r.message }}</span>
                </td>
                <td class="col-time">{{ r.status === 'success' || r.status === 'failed' ? r.duration + 'ms' : '-' }}</td>
                <td class="col-op">
                  <button class="btn btn-sm btn-outline" :disabled="mainRunning || r.status === 'testing'" @click="testOne('main', i)">单独测试</button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { getToken, showToast } from '@/api/client'

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

// ===== 接口定义（与 Rust 服务器 handlers/mod.rs dispatch 完全对齐） =====
const mainApis: ApiConfig[] = [
  // 系统自检
  { name: 'check', method: 'GET', sign: false, desc: '服务器自检/数据库连接检查', body: {} },
  { name: 'install', method: 'GET', sign: false, desc: '数据库安装', body: {} },
  { name: 'get_source_status', method: 'GET', sign: false, desc: '获取音源状态', body: {} },
  { name: 'get_server_load', method: 'GET', sign: true, desc: '获取服务器负载', body: {} },
  { name: 'get_captcha', method: 'GET', sign: false, desc: '获取人机验证配置', body: {} },
  { name: 'get_user_agreement', method: 'GET', sign: false, desc: '获取用户协议', body: {} },
  // 版本与公告
  { name: 'get_version_status', method: 'POST', sign: true, desc: '获取版本状态', body: { version: '1.0.0' } },
  { name: 'get_latest_version', method: 'POST', sign: true, desc: '获取最新版本', body: {} },
  { name: 'get_announcement', method: 'GET', sign: true, desc: '获取公告', body: {} },
  { name: 'get_about_config', method: 'GET', sign: true, desc: '获取关于页配置', body: {} },
  { name: 'get_site_logo', method: 'GET', sign: true, desc: '获取站点Logo', body: {} },
  { name: 'get_leaderboard', method: 'GET', sign: true, desc: '获取排行榜', body: {} },
  // 用户认证
  { name: 'register', method: 'POST', sign: true, desc: '用户注册', body: { username: 'test_check_user', password: 'test123456', email: 'test@example.com', verify_code: '000000' } },
  { name: 'user_login', method: 'POST', sign: true, desc: '用户登录', body: { username: 'test_nonexist', password: 'test123456' } },
  { name: 'login_by_code', method: 'POST', sign: true, desc: '验证码登录', body: { email: 'test@example.com', verify_code: '000000' } },
  { name: 'send_verify_code', method: 'POST', sign: true, desc: '发送验证码', body: { email: 'test@example.com', type: 'register' } },
  { name: 'reset_password', method: 'POST', sign: true, desc: '重置密码', body: { email: 'test@example.com', verify_code: '000000', new_password: 'test123456' } },
  { name: 'verify_captcha', method: 'POST', sign: false, desc: '验证人机验证', body: { token: 'test' } },
  { name: 'check_ban_status', method: 'POST', sign: true, desc: '检查封禁状态', body: { device_id: 'test_device_001' } },
  { name: 'delete_account', method: 'POST', sign: true, desc: '删除账号', body: { ciyuanxi_id: 'test', password: 'test123456' } },
  // 用户信息
  { name: 'get_user_info', method: 'POST', sign: true, desc: '获取用户信息', body: { ciyuanxi_id: 'test' } },
  { name: 'get_user_settings', method: 'POST', sign: true, desc: '获取用户设置', body: { ciyuanxi_id: 'test' } },
  { name: 'check_username', method: 'POST', sign: true, desc: '检查弦予号可用性', body: { username: 'test_check_user' } },
  { name: 'report_listen_stats', method: 'POST', sign: true, desc: '上报听歌统计', body: { ciyuanxi_id: 'test', listen_duration: 100 } },
  // 反馈与社交
  { name: 'submit_feedback', method: 'POST', sign: true, desc: '提交反馈', body: { ciyuanxi_id: 'test', content: '测试反馈', feedback_type: 'problem' } },
  { name: 'submit_appeal', method: 'POST', sign: true, desc: '提交申诉', body: { ciyuanxi_id: 'test', content: '测试申诉', device_id: 'test_device_001' } },
  { name: 'list_my_feedback', method: 'POST', sign: true, desc: '获取我的反馈列表', body: { ciyuanxi_id: 'test' } },
  // 壁纸
  { name: 'list_wallpapers', method: 'GET', sign: false, desc: '获取壁纸列表', body: {} },
  // 上报
  { name: 'error', method: 'POST', sign: true, desc: '错误上报', body: { device_id: 'test_device_001', app_version: '1.0.0', os_version: '13', device_model: 'TestDevice', error_type: 'TestError', error_message: '测试错误', error_stack: '', page: 'test' } },
  { name: 'open', method: 'POST', sign: false, desc: 'APP启动上报', body: { device_id: 'test_device_001', app_version: '1.0.0', os_version: '13', device_model: 'TestDevice' } },
  // 邮箱认证
  { name: 'email_get_captcha_config', method: 'GET', sign: false, desc: '获取验证码配置', body: {} },
  { name: 'email_get_turnstile_config', method: 'GET', sign: false, desc: '获取Turnstile配置', body: {} },
  { name: 'email_send_code', method: 'POST', sign: false, desc: '发送邮箱验证码', body: { email: 'test@example.com', type: 'register' } },
  { name: 'email_register', method: 'POST', sign: false, desc: '邮箱注册', body: { email: 'test@example.com', password: 'test123456', verify_code: '000000' } },
  { name: 'email_login', method: 'POST', sign: false, desc: '邮箱登录', body: { email: 'test@example.com', verify_code: '000000' } },
  { name: 'email_reset_password', method: 'POST', sign: false, desc: '邮箱重置密码', body: { email: 'test@example.com', verify_code: '000000', new_password: 'test123456' } },
  { name: 'email_get_profile', method: 'POST', sign: false, desc: '获取邮箱用户资料', body: { token: 'test' } },
]

// 预期可接受的错误码（接口正常工作但业务参数不合法时的返回）
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

// ===== 状态 =====
function makeIdle(apis: ApiConfig[]): TestResult[] {
  return apis.map(a => ({ name: a.name, status: 'idle', message: '', duration: 0 }))
}

const mainResults = ref<TestResult[]>(makeIdle(mainApis))
const mainRunning = ref(false)

// ===== 汇总 =====
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

// ===== 请求工具 =====
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

// ===== 结果评估 =====
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

// ===== 单接口测试 =====
async function testOne(group: 'main', index: number) {
  const apis = mainApis
  const results = mainResults
  const api = apis[index]

  if (results.value[index].status === 'testing') return

  results.value[index] = { name: api.name, status: 'testing', message: '', duration: 0 }

  const payload: Record<string, any> = {
    api_action: api.name,
    method: api.method,
    body: JSON.stringify(api.body || {}),
    need_sign: api.sign ? '1' : '0',
  }

  const start = Date.now()
  const json = await callProxy('proxy_api_test', payload)
  const elapsed = Date.now() - start

  results.value[index] = evaluateResult(api, json, elapsed)
}

// ===== 一键测试全部（顺序执行） =====
async function runAllMain() {
  if (mainRunning.value) return
  mainRunning.value = true
  mainResults.value = makeIdle(mainApis)

  for (let i = 0; i < mainApis.length; i++) {
    await testOne('main', i)
  }

  mainRunning.value = false
  const sum = mainSummary.value
  showToast(
    `接口测试完成：成功 ${sum.success}，失败 ${sum.failed}`,
    sum.failed > 0 ? 'error' : 'success',
  )
}

// ===== 清空结果 =====
function clearMain() {
  if (mainRunning.value) return
  mainResults.value = makeIdle(mainApis)
}
</script>

<style scoped>
.api-test-page {
  max-width: 1100px;
  margin: 0 auto;
}

/* ===== 页面头部 ===== */
.page-header { margin-bottom: 20px; }
.page-title { font-size: 22px; font-weight: 800; letter-spacing: -0.02em; margin: 0 0 6px 0; }
.page-desc { font-size: 13px; color: var(--text-muted); line-height: 1.6; margin: 0; max-width: 560px; }

/* ===== 卡片 ===== */
.card { background: var(--white); border: 1px solid var(--border); border-radius: 14px; padding: 20px; margin-bottom: 16px; }
.card-head { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; flex-wrap: wrap; gap: 12px; }
.card-head-left { display: flex; flex-direction: column; gap: 2px; }
.card-title { font-size: 16px; font-weight: 700; margin: 0; }
.card-sub { font-size: 12px; color: var(--text-muted); }
.card-actions { display: flex; gap: 8px; flex-wrap: wrap; }

/* ===== 按钮 ===== */
.btn { display: inline-flex; align-items: center; gap: 6px; padding: 8px 16px; border-radius: 10px; border: none; font-size: 13px; font-weight: 600; cursor: pointer; transition: all 0.25s; white-space: nowrap; }
.btn-primary { background: var(--accent); color: #fff; }
.btn-primary:hover:not(:disabled) { transform: translateY(-1px); box-shadow: 0 6px 18px rgba(0, 0, 0, 0.15); }
.btn-outline { background: var(--white); border: 1px solid var(--border); color: var(--text-light); }
.btn-outline:hover:not(:disabled) { border-color: #ccc; color: var(--text); transform: translateY(-1px); }
.btn:disabled { opacity: 0.5; cursor: not-allowed; transform: none; }
.btn-sm { padding: 5px 12px; font-size: 12px; border-radius: 8px; }

/* ===== 汇总卡片 ===== */
.summary-row { display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; margin-bottom: 16px; }
.summary-card { background: #fafafa; border-radius: 10px; padding: 14px 16px; text-align: center; transition: transform 0.25s cubic-bezier(0.16, 1, 0.3, 1); }
.summary-card:hover { transform: translateY(-2px); }
.summary-num { font-size: 24px; font-weight: 800; line-height: 1.1; }
.summary-unit { font-size: 13px; font-weight: 600; color: var(--text-muted); margin-left: 2px; }
.summary-label { font-size: 11px; color: var(--text-muted); margin-top: 4px; }
.summary-success { background: #f0fdf4; }
.summary-success .summary-num { color: #16a34a; }
.summary-failed { background: #fef2f2; }
.summary-failed .summary-num { color: #dc2626; }
.summary-time { background: #f5f3ff; }
.summary-time .summary-num { color: #7c3aed; }

/* ===== 表格 ===== */
.table-wrap { overflow-x: auto; border: 1px solid var(--border); border-radius: 10px; }
.data-table { width: 100%; border-collapse: collapse; min-width: 720px; }
.data-table th { padding: 10px 12px; text-align: left; font-size: 12px; font-weight: 600; color: var(--text-muted); background: #fafafa; border-bottom: 1px solid var(--border); white-space: nowrap; }
.data-table td { padding: 10px 12px; font-size: 13px; color: var(--text); border-bottom: 1px solid #f5f5f5; vertical-align: middle; }
.data-table tbody tr:last-child td { border-bottom: none; }
.data-table tbody tr { transition: background 0.15s; }
.data-table tbody tr:hover { background: #fafafa; }
.data-table tbody tr.row-testing { background: #fbfbfe; }
.data-table tbody tr.row-failed { background: #fffdfd; }
.data-table tbody tr.row-failed:hover { background: #fff8f8; }

/* 表格行逐条加载动画（与数据库管理页一致） */
.data-table tbody tr.row-anim {
  animation: rowIn 0.45s cubic-bezier(0.16, 1, 0.3, 1) both;
}
@keyframes rowIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

.col-idx { width: 44px; color: var(--text-muted); font-weight: 600; text-align: center; }
.col-status { width: 92px; white-space: nowrap; }
.col-time { width: 80px; white-space: nowrap; color: var(--text-muted); }
.col-op { width: 96px; white-space: nowrap; text-align: right; }
.col-msg { min-width: 200px; }

.api-name { font-weight: 600; }
.api-desc { font-size: 11px; color: var(--text-muted); margin-top: 1px; }

/* ===== 徽章 ===== */
.badge { display: inline-flex; align-items: center; padding: 3px 10px; border-radius: 20px; font-size: 11px; font-weight: 600; }
.badge-success { background: #f0fdf4; color: #16a34a; }
.badge-error { background: #fef2f2; color: #dc2626; }
.badge-pending { background: #f5f5f5; color: var(--text-muted); }

.testing-tag { display: inline-flex; align-items: center; gap: 5px; font-size: 11px; font-weight: 600; color: #7c3aed; }

.msg-empty { color: #d4d4d4; }
.msg-pending { color: var(--text-muted); font-size: 12px; }
.msg-text { font-size: 12px; word-break: break-all; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; line-height: 1.5; }
.msg-success { color: #16a34a; }
.msg-failed { color: #dc2626; }

/* ===== Spinner ===== */
.spinner-sm { width: 14px; height: 14px; border: 2px solid #e5e5e5; border-top-color: var(--accent); border-radius: 50%; animation: spin 0.7s linear infinite; display: inline-block; }
.testing-tag .spinner-sm { border-color: #ede9fe; border-top-color: #7c3aed; }
@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }

/* ===== 过渡动画 ===== */
.fade-down-enter-active, .fade-down-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-down-enter-from { opacity: 0; transform: translateY(-12px); }
.fade-up-enter-active, .fade-up-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-up-enter-from { opacity: 0; transform: translateY(12px); }

/* ===== 响应式 ===== */
@media (max-width: 768px) {
  .summary-row { grid-template-columns: 1fr 1fr; gap: 8px; }
  .summary-num { font-size: 20px; }
  .card-head { align-items: flex-start; }
}
</style>
