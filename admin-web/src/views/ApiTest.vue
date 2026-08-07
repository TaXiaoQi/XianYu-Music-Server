<template>
  <div class="api-test-page">
    <!-- 页面头部 -->
    <Transition name="fade-down" appear>
      <div class="page-header">
        <div class="header-info">
          <h2 class="page-title">接口测试</h2>
          <p class="page-desc">
            在线测试主应用接口与 APP 管理接口的可用性，支持单接口测试与一键批量顺序测试，实时展示响应状态、返回信息与耗时。
          </p>
        </div>
      </div>
    </Transition>

    <!-- 主应用接口 -->
    <Transition name="fade-up" appear>
      <section class="card">
        <div class="card-head">
          <div class="card-head-left">
            <h3 class="card-title">主应用接口</h3>
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

    <!-- APP 管理接口 -->
    <Transition name="fade-up" appear>
      <section class="card" style="transition-delay: 0.1s">
        <div class="card-head">
          <div class="card-head-left">
            <h3 class="card-title">APP 管理接口</h3>
            <span class="card-sub">proxy_app_api_test · 共 {{ appApis.length }} 个接口</span>
          </div>
          <div class="card-actions">
            <button class="btn btn-primary" :disabled="appRunning" @click="runAllApp">
              <span v-if="appRunning" class="spinner-sm"></span>
              <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polygon points="5 3 19 12 5 21 5 3"/></svg>
              {{ appRunning ? '测试中...' : '一键测试全部' }}
            </button>
            <button class="btn btn-outline" :disabled="appRunning" @click="clearApp">清空结果</button>
          </div>
        </div>

        <!-- 测试参数 -->
        <div class="param-bar">
          <div class="param-field">
            <label>Admin Token</label>
            <input v-model="appAdminToken" type="password" placeholder="X-Admin-Token（运行 app_login 后自动填入）" autocomplete="off" />
          </div>
          <div class="param-field param-field-sm">
            <label>AES 加密</label>
            <select v-model="appNeedEncrypt">
              <option value="0">关闭</option>
              <option value="1">开启</option>
            </select>
          </div>
          <div class="param-hint">需要 Token 的接口若未填入将跳过请求并提示</div>
        </div>

        <!-- 汇总卡片 -->
        <Transition name="fade-up">
          <div v-if="appHasResult" class="summary-row">
            <div class="summary-card">
              <div class="summary-num">{{ appSummary.total }}</div>
              <div class="summary-label">接口总数</div>
            </div>
            <div class="summary-card summary-success">
              <div class="summary-num">{{ appSummary.success }}</div>
              <div class="summary-label">成功</div>
            </div>
            <div class="summary-card summary-failed">
              <div class="summary-num">{{ appSummary.failed }}</div>
              <div class="summary-label">失败</div>
            </div>
            <div class="summary-card summary-time">
              <div class="summary-num">{{ appSummary.time }}<span class="summary-unit">ms</span></div>
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
                v-for="(r, i) in appResults"
                :key="'app-' + r.name"
                :class="rowClass(r)"
              >
                <td class="col-idx">{{ i + 1 }}</td>
                <td>
                  <div class="api-name">{{ appApis[i].name }}</div>
                  <div class="api-desc">{{ appApis[i].desc }}</div>
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
                  <button class="btn btn-sm btn-outline" :disabled="appRunning || r.status === 'testing'" @click="testOne('app', i)">单独测试</button>
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
  needToken?: boolean
}

interface TestResult {
  name: string
  status: 'idle' | 'testing' | 'success' | 'failed'
  message: string
  duration: number
}

// ===== 接口定义 =====
const mainApis: ApiConfig[] = [
  { name: 'source_call', method: 'POST', sign: true, desc: '音源调用上报', body: { device_id: 'test_device_001', source_name: 'kg_official', action: 'getMusicUrl', song_name: '测试歌曲', singer: '测试歌手', status: 1, result_status: 'success', error_msg: '', duration_ms: 100 } },
  { name: 'login', method: 'POST', sign: true, desc: '登录上报', body: { device_id: 'test_device_001', user_id: '1', username: 'testuser', status: 1 } },
  { name: 'error', method: 'POST', sign: true, desc: '错误上报', body: { device_id: 'test_device_001', app_version: '1.0.0', os_version: '13', device_model: 'TestDevice', error_type: 'TestError', error_message: '测试错误', error_stack: '', page: 'test' } },
  { name: 'register', method: 'POST', sign: true, desc: '用户注册', body: { username: 'test_check_user', password: 'test123456', email: 'test@example.com', verify_code: '000000' } },
  { name: 'user_login', method: 'POST', sign: true, desc: '用户登录', body: { username: 'test_nonexist', password: 'test123456' } },
  { name: 'login_by_code', method: 'POST', sign: true, desc: '验证码登录', body: { email: 'test@example.com', verify_code: '000000' } },
  { name: 'send_verify_code', method: 'POST', sign: true, desc: '发送验证码', body: { email: 'test@example.com', type: 'register' } },
  { name: 'reset_password', method: 'POST', sign: true, desc: '重置密码', body: { email: 'test@example.com', verify_code: '000000', new_password: 'test123456' } },
  { name: 'get_banner', method: 'GET', sign: false, desc: '获取轮播图', body: {} },
  { name: 'get_source_status', method: 'GET', sign: false, desc: '获取音源状态', body: {} },
  { name: 'get_version_status', method: 'POST', sign: true, desc: '获取版本状态', body: { version: '1.0.0' } },
  { name: 'get_latest_version', method: 'POST', sign: true, desc: '获取最新版本', body: {} },
  { name: 'check', method: 'GET', sign: false, desc: '数据库连接检查', body: {} },
  { name: 'install', method: 'GET', sign: false, desc: '数据库安装', body: {} },
]

const appApis: ApiConfig[] = [
  { name: 'app_check', method: 'GET', sign: false, needToken: false, desc: 'APP 端服务器自检', body: {} },
  { name: 'app_install', method: 'GET', sign: false, needToken: false, desc: 'APP 端数据库初始化', body: {} },
  { name: 'app_login', method: 'POST', sign: true, needToken: false, desc: '管理员登录', body: { username: 'admin', password: 'admin123' } },
  { name: 'app_logout', method: 'POST', sign: true, needToken: true, desc: '管理员登出', body: {} },
  { name: 'app_profile', method: 'POST', sign: true, needToken: true, desc: '获取当前管理员资料', body: {} },
  { name: 'app_change_password', method: 'POST', sign: true, needToken: true, desc: '修改密码', body: { old_password: 'admin123', new_password: 'admin123', confirm_password: 'admin123' } },
  { name: 'app_dashboard', method: 'POST', sign: true, needToken: true, desc: '仪表盘统计', body: {} },
  { name: 'app_users_list', method: 'POST', sign: true, needToken: true, desc: '用户列表', body: { page: 1, per_page: 20, keyword: '', status: -1 } },
  { name: 'app_user_toggle_status', method: 'POST', sign: true, needToken: true, desc: '切换用户状态', body: { id: 0, status: 0 } },
  { name: 'app_user_delete', method: 'POST', sign: true, needToken: true, desc: '删除用户', body: { id: 0 } },
  { name: 'app_sources_list', method: 'POST', sign: true, needToken: true, desc: '音源列表', body: {} },
  { name: 'app_source_toggle', method: 'POST', sign: true, needToken: true, desc: '音源开关', body: { source_code: '', enabled: 0 } },
  { name: 'app_logs_list', method: 'POST', sign: true, needToken: true, desc: '各类日志列表', body: { type: 'login', page: 1, per_page: 20, keyword: '' } },
  { name: 'app_log_detail', method: 'POST', sign: true, needToken: true, desc: '日志详情', body: { type: 'login', id: 0 } },
  { name: 'app_app_login_logs', method: 'POST', sign: true, needToken: true, desc: 'APP 登录日志', body: { page: 1, per_page: 20 } },
  { name: 'app_banners_list', method: 'POST', sign: true, needToken: true, desc: 'Banner 列表', body: { page: 1, per_page: 20 } },
  { name: 'app_banner_add', method: 'POST', sign: true, needToken: true, desc: '新增 Banner', body: { title: '测试Banner', image_url: '', link_url: '', sort_order: 0, status: 1, expired_at: '' } },
  { name: 'app_banner_update', method: 'POST', sign: true, needToken: true, desc: '编辑 Banner', body: { id: 0, title: '测试Banner', image_url: '', link_url: '', sort_order: 0, status: 1, expired_at: '' } },
  { name: 'app_banner_delete', method: 'POST', sign: true, needToken: true, desc: '删除 Banner', body: { id: 0 } },
  { name: 'app_versions_list', method: 'POST', sign: true, needToken: true, desc: '版本列表', body: { page: 1, per_page: 20 } },
  { name: 'app_version_status_change', method: 'POST', sign: true, needToken: true, desc: '修改版本状态', body: { id: 0, status: 1 } },
  { name: 'app_version_delete', method: 'POST', sign: true, needToken: true, desc: '删除版本', body: { id: 0 } },
  { name: 'app_admins_list', method: 'POST', sign: true, needToken: true, desc: '管理员列表', body: {} },
  { name: 'app_admin_add', method: 'POST', sign: true, needToken: true, desc: '新增管理员', body: { username: '', password: '', role: 'admin' } },
  { name: 'app_admin_delete', method: 'POST', sign: true, needToken: true, desc: '删除管理员', body: { id: 0 } },
  { name: 'app_database_tables', method: 'POST', sign: true, needToken: true, desc: '数据库表列表', body: {} },
  { name: 'app_database_view', method: 'POST', sign: true, needToken: true, desc: '查看表数据', body: { table: 'app_users', page: 1, per_page: 20 } },
  { name: 'app_database_repair', method: 'POST', sign: true, needToken: true, desc: '数据库修复', body: {} },
  { name: 'app_operation_logs', method: 'POST', sign: true, needToken: true, desc: '后台操作日志', body: { page: 1, per_page: 20 } },
  { name: 'app_admin_login_logs', method: 'POST', sign: true, needToken: true, desc: '网页端登录日志', body: { page: 1, per_page: 20 } },
]

// 预期可接受的错误码（接口正常工作但业务参数不合法时的返回）
const MAIN_EXPECTED: Record<string, number[]> = {
  user_login: [401],
  register: [400, 401],
  login_by_code: [400, 401],
  reset_password: [400],
  send_verify_code: [200, 429, 500],
}
const APP_EXPECTED: Record<string, number[]> = {
  app_user_toggle_status: [400, 404],
  app_user_delete: [400, 404],
  app_source_toggle: [400, 404],
  app_log_detail: [400, 404],
  app_banner_add: [400],
  app_banner_update: [400, 404],
  app_banner_delete: [400, 404],
  app_version_status_change: [400, 404],
  app_version_delete: [400, 404],
  app_admin_add: [400],
  app_admin_delete: [400, 404],
  app_change_password: [400, 401],
}

// ===== 状态 =====
function makeIdle(apis: ApiConfig[]): TestResult[] {
  return apis.map(a => ({ name: a.name, status: 'idle', message: '', duration: 0 }))
}

const mainResults = ref<TestResult[]>(makeIdle(mainApis))
const appResults = ref<TestResult[]>(makeIdle(appApis))
const mainRunning = ref(false)
const appRunning = ref(false)
const appAdminToken = ref('')
const appNeedEncrypt = ref('0')

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
const appSummary = computed(() => sumResults(appResults.value))
const mainHasResult = computed(() => mainSummary.value.success + mainSummary.value.failed > 0)
const appHasResult = computed(() => appSummary.value.success + appSummary.value.failed > 0)

function rowClass(r: TestResult) {
  return {
    'row-testing': r.status === 'testing',
    'row-success': r.status === 'success',
    'row-failed': r.status === 'failed',
  }
}

// ===== 请求工具 =====
// 直接使用 fetch + getToken 调用后台代理接口。
// 不复用 adminApi，因为 adminApi 在响应 code === 401 时会清除 token 并跳转登录页，
// 而本页测试的多个接口会合法返回 401（如 user_login、app_login、app_change_password），
// 这会导致“一键测试全部”在遇到 401 时中断并把管理员登出。
// 此处保留同样的 Bearer 鉴权头，但不对 401 做跳转处理。
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
function evaluateResult(api: ApiConfig, json: any, elapsed: number, group: 'main' | 'app'): TestResult {
  const httpCode = json.http_code || 0
  const apiCode = typeof json.code === 'number' ? json.code : -1
  const apiMsg = json.msg || ''
  const expected = group === 'main' ? MAIN_EXPECTED[api.name] : APP_EXPECTED[api.name]
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
async function testOne(group: 'main' | 'app', index: number) {
  const apis = group === 'main' ? mainApis : appApis
  const results = group === 'main' ? mainResults : appResults
  const api = apis[index]

  if (results.value[index].status === 'testing') return

  // APP 接口若需要 Token 但未提供，则直接标记失败并提示
  if (group === 'app' && api.needToken && !appAdminToken.value.trim()) {
    results.value[index] = {
      name: api.name,
      status: 'failed',
      message: '缺少 admin_token，请先运行 app_login 获取 Token 后填入',
      duration: 0,
    }
    return
  }

  results.value[index] = { name: api.name, status: 'testing', message: '', duration: 0 }

  const payload: Record<string, any> = {
    api_action: api.name,
    method: api.method,
    body: JSON.stringify(api.body || {}),
    need_sign: api.sign ? '1' : '0',
  }
  if (group === 'app') {
    payload.need_encrypt = appNeedEncrypt.value
    payload.admin_token = appAdminToken.value.trim()
  }

  const start = Date.now()
  const json = await callProxy(group === 'main' ? 'proxy_api_test' : 'proxy_app_api_test', payload)
  const elapsed = Date.now() - start

  // app_login 成功后自动捕获并填入 token
  if (group === 'app' && api.name === 'app_login' && json.code === 200 && json.data && json.data.token) {
    appAdminToken.value = json.data.token
    showToast('已自动填入 admin_token', 'success')
  }

  results.value[index] = evaluateResult(api, json, elapsed, group)
}

// ===== 一键测试全部（顺序执行） =====
async function runAll(group: 'main' | 'app') {
  const apis = group === 'main' ? mainApis : appApis
  const results = group === 'main' ? mainResults : appResults
  const running = group === 'main' ? mainRunning : appRunning
  if (running.value) return
  running.value = true
  results.value = makeIdle(apis)

  for (let i = 0; i < apis.length; i++) {
    await testOne(group, i)
  }

  running.value = false
  const sum = group === 'main' ? mainSummary.value : appSummary.value
  showToast(
    `${group === 'main' ? '主应用' : 'APP 管理'}接口测试完成：成功 ${sum.success}，失败 ${sum.failed}`,
    sum.failed > 0 ? 'error' : 'success',
  )
}

function runAllMain() { runAll('main') }
function runAllApp() { runAll('app') }

// ===== 清空结果 =====
function clearMain() {
  if (mainRunning.value) return
  mainResults.value = makeIdle(mainApis)
}
function clearApp() {
  if (appRunning.value) return
  appResults.value = makeIdle(appApis)
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

/* ===== 参数栏（APP） ===== */
.param-bar { display: flex; align-items: flex-end; gap: 12px; flex-wrap: wrap; background: #fafafa; border: 1px solid var(--border); border-radius: 10px; padding: 12px 14px; margin-bottom: 16px; }
.param-field { display: flex; flex-direction: column; gap: 4px; }
.param-field-sm { min-width: 96px; }
.param-field label { font-size: 11px; font-weight: 600; color: var(--text-muted); }
.param-field input,
.param-field select { border: 1px solid var(--border); border-radius: 8px; padding: 7px 10px; font-size: 13px; outline: none; background: var(--white); color: var(--text); transition: border-color 0.2s; font-family: inherit; }
.param-field input { width: 320px; max-width: 100%; }
.param-field input:focus,
.param-field select:focus { border-color: var(--accent); }
.param-hint { font-size: 11px; color: var(--text-muted); margin-left: auto; align-self: center; }

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
  .param-field input { width: 100%; }
  .param-hint { width: 100%; margin-left: 0; }
  .card-head { align-items: flex-start; }
}
</style>
