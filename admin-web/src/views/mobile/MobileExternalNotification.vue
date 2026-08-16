<template>
  <div class="mobile-page">
    <!-- 页头 -->
    <div class="notify-header">
      <div class="notify-header-info">
        <h2 class="notify-title">外部通知</h2>
        <p class="notify-desc">管理用于接收后台状态通知的绑定邮箱，并可分别设置壁纸审核、头像审核、昵称审核、反馈更新等板块的通知开关。</p>
      </div>
      <div v-if="activeTab === 'email'" class="notify-header-actions">
        <button class="mobile-btn" :disabled="importing" @click="doImportAdmin">{{ importing ? '导入中...' : '导入管理员邮箱' }}</button>
        <button class="mobile-btn primary" @click="openAddModal">+ 新增</button>
      </div>
    </div>

    <!-- 标签栏 -->
    <div class="tab-bar">
      <button class="tab-item" :class="{ active: activeTab === 'settings' }" @click="activeTab = 'settings'">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
        通知设置
      </button>
      <button class="tab-item" :class="{ active: activeTab === 'email' }" @click="activeTab = 'email'">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="4" width="20" height="16" rx="2"/><path d="M22 7l-10 6L2 7"/></svg>
        通知邮箱
      </button>
      <button class="tab-item" :class="{ active: activeTab === 'webhook' }" @click="activeTab = 'webhook'">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg>
        Webhook 通知
      </button>
      <button class="tab-item" :class="{ active: activeTab === 'commtool' }" @click="activeTab = 'commtool'">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"/><path d="M8 9h8M8 13h5"/></svg>
        通信工具
      </button>
    </div>

    <!-- ==================== 通知设置 Tab ==================== -->
    <template v-if="activeTab === 'settings'">
      <section class="mobile-card">
        <div class="module-head">
          <div class="module-icon">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 0 1-3.46 0"/></svg>
          </div>
          <div class="module-text">
            <h3 class="module-name">通知设置</h3>
            <p class="module-desc">配置浏览器 / 系统通知的授权状态、总开关与各板块开关。</p>
          </div>
        </div>

        <div v-if="!notify.supportedByBrowser" class="notify-tip warn">
          当前环境不支持系统通知，请改用最新版 Chrome / Edge / Firefox 或系统浏览器打开。
        </div>

        <template v-else>
          <div v-if="notify.permission === 'default'" class="notify-perm-row">
            <button class="mobile-btn primary" @click="handleRequestPermission">授权通知</button>
            <span class="notify-hint">{{ notify.nativeBridgeAvailable ? '系统将询问是否允许发送通知' : '浏览器将询问是否允许本站发送通知' }}</span>
          </div>
          <div v-else-if="notify.permission === 'denied'" class="notify-tip warn">
            {{ notify.nativeBridgeAvailable ? '系统通知权限已被拒绝' : '浏览器已拒绝通知权限，请在浏览器设置中允许本站通知后重试。' }}
            <button v-if="notify.nativeBridgeAvailable" class="notify-settings-btn" @click="notify.openNotificationSettings()">打开系统设置</button>
          </div>
          <div v-else class="notify-perm-ok">
            <span class="notify-hint">已授权，有新事件时将在此提醒</span>
          </div>

          <div class="notify-divider"></div>

          <div class="notify-row">
            <span class="notify-row-label">启用通知</span>
            <label class="switch">
              <input :checked="notify.enabled" type="checkbox" @change="notify.setEnabled(!notify.enabled)" />
              <span class="track"><span class="thumb"></span></span>
            </label>
          </div>

          <div class="settings-modules">
            <div v-for="m in notify.moduleList" :key="m.key" class="notify-row">
              <span class="notify-row-label">
                {{ m.label }}
                <span class="notify-row-desc">{{ m.desc }}</span>
              </span>
              <label class="switch">
                <input :checked="notify.modules[m.key]" type="checkbox" @change="notify.toggleModule(m.key)" />
                <span class="track"><span class="thumb"></span></span>
              </label>
            </div>
          </div>

          <div class="notify-divider"></div>

          <button class="mobile-btn" :disabled="!notify.granted" @click="notify.testNotification()">发送测试通知</button>
          <div class="notify-tip">有新反馈或待审核项时，将弹出系统通知提醒。</div>
        </template>
      </section>
    </template>

    <!-- ==================== 通知邮箱 Tab ==================== -->
    <template v-if="activeTab === 'email'">
      <!-- 通知板块设置 -->
      <section class="mobile-card">
        <div class="module-head">
          <div class="module-icon">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
          </div>
          <div class="module-text">
            <h3 class="module-name">通知板块</h3>
            <p class="module-desc">全局开关，控制哪些板块默认发送通知。关闭后该板块所有邮箱将不再收到通知；开启后下方邮箱可单独关闭。</p>
          </div>
        </div>
        <div class="module-list">
          <div v-for="mod in moduleList" :key="mod.key" class="module-item">
            <div class="module-item-icon" :class="'mi-' + mod.key">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M3 9h18"/><path d="M9 21V9"/></svg>
            </div>
            <div class="module-item-text">
              <span class="module-item-name">{{ mod.label }}</span>
              <span class="module-item-desc">{{ mod.desc }}</span>
            </div>
            <label class="switch">
              <input :checked="moduleSettings[mod.key] === true" type="checkbox" @change="toggleModule(mod.key)" />
              <span class="track"><span class="thumb"></span></span>
            </label>
          </div>
        </div>
      </section>

      <!-- 统计卡片 -->
      <div class="stats-grid">
        <div class="stat-chip">
          <div class="stat-icon stat-icon-total">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="4" width="20" height="16" rx="2"/><path d="M22 7l-10 6L2 7"/></svg>
          </div>
          <div class="stat-body"><span class="stat-num">{{ stats.total }}</span><span class="stat-label">总数</span></div>
        </div>
        <div class="stat-chip">
          <div class="stat-icon stat-icon-active">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
          </div>
          <div class="stat-body"><span class="stat-num">{{ stats.active }}</span><span class="stat-label">启用中</span></div>
        </div>
        <div class="stat-chip">
          <div class="stat-icon stat-icon-disabled">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="4.93" y1="4.93" x2="19.07" y2="19.07"/></svg>
          </div>
          <div class="stat-body"><span class="stat-num">{{ stats.disabled }}</span><span class="stat-label">已停用</span></div>
        </div>
      </div>

      <!-- 加载中 -->
      <div v-if="loading" class="mobile-empty">加载中...</div>

      <!-- 空状态 -->
      <div v-else-if="list.length === 0" class="mobile-empty">
        <div class="empty-title">暂无通知邮箱</div>
        <div class="empty-sub">点击上方"新增"添加，用于接收后台状态通知</div>
      </div>

      <!-- 通知邮箱列表 -->
      <div v-else class="mobile-list">
        <div v-for="item in list" :key="item.id" class="mobile-item" :class="{ 'is-disabled': item.status == 0 }">
          <div class="mobile-item-head">
            <div class="mobile-admin-left">
              <div class="notify-avatar" :class="item.status == 1 ? 'avatar-active' : 'avatar-disabled'">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="4" width="20" height="16" rx="2"/><path d="M22 7l-10 6L2 7"/></svg>
              </div>
              <div>
                <div class="mobile-item-title">{{ item.email }}</div>
                <div class="mobile-item-sub">{{ item.remark || '暂无备注' }}</div>
                <div class="mobile-item-sub muted-time">创建时间：{{ item.created_at || '-' }}</div>
              </div>
            </div>
            <span class="mobile-badge" :class="item.status == 1 ? 'green' : 'red'">{{ item.status == 1 ? '启用中' : '已停用' }}</span>
          </div>
          <div class="notify-modules">
            <span
              v-for="mod in moduleList"
              :key="mod.key"
              class="notify-module-tag"
              :class="{ 'tag-on': item[mod.field] == 1, 'tag-off': item[mod.field] == 0 }"
              @click="toggleModuleOnEmail(item, mod.key)"
            >
              <span class="tag-dot"></span>
              {{ mod.label }}
            </span>
          </div>
          <div class="mobile-actions">
            <button class="mobile-btn" :disabled="testingId === item.id" @click="sendTest(item)">{{ testingId === item.id ? '发送中...' : '发送测试' }}</button>
            <button class="mobile-btn" @click="toggle(item)">{{ item.status == 1 ? '停用' : '启用' }}</button>
            <button class="mobile-btn danger" @click="remove(item)">删除</button>
          </div>
        </div>
      </div>

      <!-- 新增通知邮箱弹窗 -->
      <Transition name="modal" @before-leave="removeBackdropBlur">
        <div v-if="addModalVisible" class="modal-backdrop">
          <div class="modal-dialog">
            <div class="modal-head">
              <h3>新增通知邮箱</h3>
              <button class="modal-close" @click="closeAddModal">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
              </button>
            </div>
            <div class="modal-body">
              <label class="modal-field">
                <span class="required">邮箱地址</span>
                <input v-model="form.email" type="email" placeholder="notify@example.com" @keydown.enter="doAdd" />
              </label>
              <label class="modal-field">
                <span>备注</span>
                <input v-model="form.remark" type="text" placeholder="例如：站长通知邮箱" @keydown.enter="doAdd" />
              </label>
              <div class="modal-field">
                <span>通知板块</span>
                <div class="modal-modules">
                  <label
                    v-for="mod in moduleList"
                    :key="mod.key"
                    class="modal-module-item"
                    :class="{ on: form[mod.field] === 1 }"
                  >
                    <input type="checkbox" v-model="form[mod.field]" :true-value="1" :false-value="0" />
                    <span class="modal-module-check">
                      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><polyline points="20 6 9 17 4 12"/></svg>
                    </span>
                    <span class="modal-module-name">{{ mod.label }}</span>
                  </label>
                </div>
              </div>
              <p class="modal-tip">添加后默认启用，用于接收所选板块的状态通知；也可在列表中单独调整。</p>
            </div>
            <div class="modal-foot">
              <button class="modal-btn cancel" @click="closeAddModal">取消</button>
              <button class="modal-btn save" :disabled="saving" @click="doAdd">{{ saving ? '添加中...' : '确认添加' }}</button>
            </div>
          </div>
        </div>
      </Transition>
    </template>

    <!-- ==================== Webhook 通知 Tab ==================== -->
    <template v-if="activeTab === 'webhook'">
      <!-- Webhook 配置 -->
      <section class="mobile-card">
        <div class="module-head">
          <div class="module-icon mi-orange">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg>
          </div>
          <div class="module-text">
            <h3 class="module-name">Webhook 配置</h3>
            <p class="module-desc">配置回调地址与请求模板，后台发生审核/反馈事件时自动推送通知到指定地址。</p>
          </div>
        </div>

        <div class="form-section">
          <div class="form-row">
            <div class="form-row-text">
              <span class="form-label">启用 Webhook</span>
              <span class="form-hint">开启后才会向回调地址发送通知</span>
            </div>
            <label class="switch">
              <input type="checkbox" v-model="webhookForm.enabled" />
              <span class="track"><span class="thumb"></span></span>
            </label>
          </div>
          <label class="form-field">
            <span class="form-label">回调地址</span>
            <input v-model="webhookForm.url" type="text" placeholder="https://example.com/webhook" />
          </label>
          <label class="form-field">
            <span class="form-label">请求方法</span>
            <select v-model="webhookForm.method">
              <option value="POST">POST</option>
              <option value="GET">GET</option>
              <option value="PUT">PUT</option>
            </select>
          </label>
          <label class="form-field">
            <span class="form-label">自定义请求头</span>
            <textarea v-model="webhookForm.headers" rows="4" placeholder="每行 Key: Value&#10;例如:&#10;Authorization: Bearer xxx&#10;X-Custom-Header: value"></textarea>
          </label>
          <label class="form-field">
            <span class="form-label">请求体模板</span>
            <textarea v-model="webhookForm.body_template" rows="4" placeholder="支持 &#123;&#123;event&#125;&#125; &#123;&#123;title&#125;&#125; &#123;&#123;detail&#125;&#125; &#123;&#123;image_url&#125;&#125; &#123;&#123;link&#125;&#125; &#123;&#123;time&#125;&#125; 占位符，留空则使用默认 JSON 格式"></textarea>
          </label>
        </div>
      </section>

      <!-- 通知板块 -->
      <section class="mobile-card">
        <div class="module-head">
          <div class="module-icon">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
          </div>
          <div class="module-text">
            <h3 class="module-name">通知板块</h3>
            <p class="module-desc">选择哪些板块触发 Webhook 通知。开启后对应板块发生事件时即会推送。</p>
          </div>
        </div>
        <div class="module-list">
          <div v-for="mod in moduleList" :key="mod.key" class="module-item">
            <div class="module-item-icon" :class="'mi-' + mod.key">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M3 9h18"/><path d="M9 21V9"/></svg>
            </div>
            <div class="module-item-text">
              <span class="module-item-name">{{ mod.label }}</span>
              <span class="module-item-desc">{{ mod.desc }}</span>
            </div>
            <label class="switch">
              <input type="checkbox" v-model="webhookForm.modules['wh_' + mod.key]" />
              <span class="track"><span class="thumb"></span></span>
            </label>
          </div>
        </div>
      </section>

      <!-- 操作 -->
      <section class="mobile-card">
        <div class="form-actions">
          <button class="mobile-btn" :disabled="webhookTesting" @click="testWebhook">{{ webhookTesting ? '测试中...' : '发送测试' }}</button>
          <button class="mobile-btn primary" :disabled="webhookSaving" @click="saveWebhookConfig">{{ webhookSaving ? '保存中...' : '保存配置' }}</button>
        </div>

        <div v-if="webhookTestResult" class="result-box">
          <div class="result-head">
            测试结果
            <span class="status-badge" :class="webhookTestResult.status < 400 ? 'on' : 'off'">
              <span class="status-dot"></span>状态码 {{ webhookTestResult.status }}
            </span>
          </div>
          <div class="result-body">
            <p class="result-label">响应内容</p>
            <pre>{{ webhookTestResult.body }}</pre>
          </div>
        </div>
      </section>
    </template>

    <!-- ==================== 通信工具 Tab ==================== -->
    <template v-if="activeTab === 'commtool'">
      <div class="sub-tab-bar">
        <button class="sub-tab-item" :class="{ active: commSubTab === 'http-server' }" @click="commSubTab = 'http-server'">HTTP服务器</button>
        <button class="sub-tab-item" :class="{ active: commSubTab === 'http-client' }" @click="commSubTab = 'http-client'">HTTP客户端</button>
        <button class="sub-tab-item" :class="{ active: commSubTab === 'sse-server' }" @click="commSubTab = 'sse-server'">HTTP SSE服务器</button>
        <button class="sub-tab-item" :class="{ active: commSubTab === 'ws-server' }" @click="commSubTab = 'ws-server'">WebSocket服务器</button>
        <button class="sub-tab-item" :class="{ active: commSubTab === 'ws-client' }" @click="commSubTab = 'ws-client'">WebSocket客户端</button>
        <button class="sub-tab-item" :class="{ active: commSubTab === 'auth' }" @click="commSubTab = 'auth'">连接鉴权</button>
      </div>

      <!-- HTTP 服务器 -->
      <template v-if="commSubTab === 'http-server'">
        <section class="mobile-card">
          <div class="module-head">
            <div class="module-icon mi-blue">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="2" width="20" height="8" rx="2"/><rect x="2" y="14" width="20" height="8" rx="2"/><line x1="6" y1="6" x2="6.01" y2="6"/><line x1="6" y1="18" x2="6.01" y2="18"/></svg>
            </div>
            <div class="module-text">
              <h3 class="module-name">HTTP 服务器</h3>
              <p class="module-desc">监听独立端口，接收并记录所有发来的 HTTP 请求。服务根据配置自动启停。</p>
            </div>
            <span class="status-badge" :class="commStatus.server_running ? 'on' : 'off'">
              <span class="status-dot"></span>{{ commStatus.server_running ? '运行中' : '未运行' }}
            </span>
          </div>
          <div class="info-line">
            <span>监听端口：<strong>{{ commStatus.server_port || '-' }}</strong></span>
          </div>
          <p class="info-hint">服务随配置自动启停，无需手动操作</p>
        </section>

        <section class="mobile-card">
          <div class="module-head">
            <div class="module-icon mi-green">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg>
            </div>
            <div class="module-text">
              <h3 class="module-name">收到的请求</h3>
              <p class="module-desc">最近收到的 HTTP 请求记录（最多 200 条）</p>
            </div>
            <button class="mobile-btn danger" @click="clearHttpLogs">清空日志</button>
          </div>
          <div v-if="httpLogs.length === 0" class="empty-mini">暂无请求记录</div>
          <div v-else class="log-list">
            <div v-for="(log, i) in httpLogs" :key="i" class="log-item">
              <div class="log-item-head">
                <span class="log-method" :class="log.method">{{ log.method }}</span>
                <span class="log-path">{{ log.path }}</span>
                <span class="log-time">{{ log.time }}</span>
              </div>
              <pre v-if="log.body" class="log-body">{{ log.body }}</pre>
            </div>
          </div>
        </section>
      </template>

      <!-- HTTP 客户端 -->
      <template v-if="commSubTab === 'http-client'">
        <section class="mobile-card">
          <div class="module-head">
            <div class="module-icon mi-green">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 2L11 13"/><path d="M22 2l-7 20-4-9-9-4 20-7z"/></svg>
            </div>
            <div class="module-text">
              <h3 class="module-name">HTTP 客户端</h3>
              <p class="module-desc">从后台发起 HTTP 请求，用于调试接口与验证回调。</p>
            </div>
          </div>
          <div class="form-section">
            <label class="form-field">
              <span class="form-label">请求地址</span>
              <input v-model="httpClientUrl" type="text" placeholder="https://example.com/api" />
            </label>
            <label class="form-field">
              <span class="form-label">请求方法</span>
              <select v-model="httpClientMethod">
                <option value="GET">GET</option>
                <option value="POST">POST</option>
              </select>
            </label>
            <label class="form-field">
              <span class="form-label">请求头（每行 Key: Value）</span>
              <textarea v-model="httpClientHeaders" rows="3" placeholder="Authorization: Bearer xxx"></textarea>
            </label>
            <label class="form-field">
              <span class="form-label">请求体</span>
              <textarea v-model="httpClientBody" rows="4" placeholder='{"key": "value"}'></textarea>
            </label>
            <button class="mobile-btn primary full" :disabled="httpClientSending" @click="sendHttpClient">{{ httpClientSending ? '发送中...' : '发送请求' }}</button>
          </div>

          <div v-if="httpClientResult" class="result-box">
            <div class="result-head">
              响应结果
              <span class="status-badge" :class="httpClientResult.status < 400 ? 'on' : 'off'">
                <span class="status-dot"></span>状态码 {{ httpClientResult.status }}
              </span>
              <span class="result-elapsed">耗时 {{ httpClientResult.elapsed_ms }}ms</span>
            </div>
            <div class="result-body">
              <p class="result-label">响应头</p>
              <pre>{{ formatHeaders(httpClientResult.headers) }}</pre>
              <p class="result-label">响应体</p>
              <pre>{{ httpClientResult.body }}</pre>
            </div>
          </div>
        </section>
      </template>

      <!-- HTTP SSE 服务器 -->
      <template v-if="commSubTab === 'sse-server'">
        <section class="mobile-card">
          <div class="module-head">
            <div class="module-icon mi-purple">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/></svg>
            </div>
            <div class="module-text">
              <h3 class="module-name">HTTP SSE 服务器</h3>
              <p class="module-desc">服务端推送事件（Server-Sent Events），客户端连接后可实时接收推送消息。</p>
            </div>
            <span class="status-badge" :class="commStatus.server_running ? 'on' : 'off'">
              <span class="status-dot"></span>{{ commStatus.server_running ? '运行中' : '未运行' }}
            </span>
          </div>
          <div class="info-line">
            <span>SSE 地址：<code>http://服务器IP:{{ commStatus.server_port || '8090' }}/sse</code></span>
          </div>
          <div class="form-section">
            <label class="form-field">
              <span class="form-label">推送消息</span>
              <textarea v-model="sseMessage" rows="3" placeholder="输入要推送的内容"></textarea>
            </label>
            <button class="mobile-btn primary full" @click="sendSsePush">推送消息</button>
          </div>
        </section>
      </template>

      <!-- WebSocket 服务器 -->
      <template v-if="commSubTab === 'ws-server'">
        <section class="mobile-card">
          <div class="module-head">
            <div class="module-icon mi-pink">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 12a9 9 0 1 1-9-9"/><path d="M21 3v6h-6"/><path d="M12 8v8M8 12h8"/></svg>
            </div>
            <div class="module-text">
              <h3 class="module-name">WebSocket 服务器</h3>
              <p class="module-desc">监听 /ws 路径，管理客户端连接并收发消息。</p>
            </div>
            <span class="status-badge" :class="commStatus.server_running ? 'on' : 'off'">
              <span class="status-dot"></span>{{ commStatus.server_running ? '运行中' : '未运行' }}
            </span>
          </div>
          <div class="info-line">
            <span>连接数：<strong>{{ commStatus.ws_server_count }}</strong></span>
            <span>WS 地址：<code>ws://服务器IP:{{ commStatus.server_port || '8090' }}/ws</code></span>
          </div>
        </section>

        <section class="mobile-card">
          <div class="module-head">
            <div class="module-icon mi-blue">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg>
            </div>
            <div class="module-text">
              <h3 class="module-name">已连接客户端</h3>
              <p class="module-desc">向指定客户端发送消息</p>
            </div>
          </div>
          <div v-if="wsServerClients.length === 0" class="empty-mini">暂无客户端连接</div>
          <div v-else class="client-list">
            <div v-for="c in wsServerClients" :key="c.id" class="client-item">
              <div class="client-info">
                <span class="client-id">{{ c.id.slice(0, 8) }}</span>
                <span class="client-time">{{ c.connected_at }}</span>
                <span v-if="c.events && c.events.length" class="client-events">
                  <span v-for="ev in c.events" :key="ev" class="client-event-tag">{{ ev }}</span>
                </span>
                <span v-else class="client-events">
                  <span class="client-event-tag tag-all">全部事件</span>
                </span>
              </div>
              <div class="client-row">
                <input v-model="wsMessage" type="text" placeholder="输入消息..." @keydown.enter="sendWsServerMessage(c.id)" />
                <button class="mobile-btn primary" @click="sendWsServerMessage(c.id)">发送</button>
              </div>
            </div>
          </div>
          <div class="form-section">
            <label class="form-field">
              <span class="form-label">广播消息</span>
              <input v-model="wsBroadcastMessage" type="text" placeholder="输入要广播给所有连接的消息..." @keydown.enter="broadcastWsMessage" />
            </label>
            <button class="mobile-btn primary full" @click="broadcastWsMessage">广播</button>
          </div>
        </section>

        <section class="mobile-card">
          <div class="module-head">
            <div class="module-icon mi-green">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg>
            </div>
            <div class="module-text">
              <h3 class="module-name">消息日志</h3>
              <p class="module-desc">服务器收发消息记录</p>
            </div>
            <button class="mobile-btn danger" @click="clearWsLogs">清空日志</button>
          </div>
          <div v-if="wsLogs.length === 0" class="empty-mini">暂无消息记录</div>
          <div v-else class="log-list">
            <div v-for="(log, i) in wsLogs" :key="i" class="log-item">
              <div class="log-item-head">
                <span class="log-direction" :class="log.direction">{{ log.direction === 'in' ? '接收' : '发送' }}</span>
                <span class="log-time">{{ log.time }}</span>
              </div>
              <pre class="log-body">{{ log.data }}</pre>
            </div>
          </div>
        </section>
      </template>

      <!-- WebSocket 客户端 -->
      <template v-if="commSubTab === 'ws-client'">
        <section class="mobile-card">
          <div class="module-head">
            <div class="module-icon mi-cyan">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>
            </div>
            <div class="module-text">
              <h3 class="module-name">WebSocket 客户端</h3>
              <p class="module-desc">作为客户端连接外部 WebSocket 服务并收发消息。</p>
            </div>
            <span class="status-badge" :class="wsClientConnected ? 'on' : 'off'">
              <span class="status-dot"></span>{{ wsClientConnected ? '已连接' : '未连接' }}
            </span>
          </div>
          <div class="form-section">
            <label class="form-field">
              <span class="form-label">连接地址</span>
              <input v-model="wsClientUrl" type="text" placeholder="ws://example.com/ws" :disabled="wsClientConnected" />
            </label>
            <button v-if="!wsClientConnected" class="mobile-btn primary full" @click="connectWsClient">连接</button>
            <button v-else class="mobile-btn danger full" @click="disconnectWsClient">断开连接</button>
          </div>
          <div v-if="wsClientConnected" class="form-section">
            <label class="form-field">
              <span class="form-label">发送消息</span>
              <input v-model="wsClientMessage" type="text" placeholder="输入要发送的消息..." @keydown.enter="sendWsClientMessage" />
            </label>
            <button class="mobile-btn primary full" @click="sendWsClientMessage">发送</button>
          </div>
        </section>

        <section class="mobile-card">
          <div class="module-head">
            <div class="module-icon mi-orange">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M23 4v6h-6"/><path d="M1 20v-6h6"/><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/></svg>
            </div>
            <div class="module-text">
              <h3 class="module-name">自动重连配置</h3>
              <p class="module-desc">保存后后台将按此配置自动连接外部服务，断线后自动重连。</p>
            </div>
          </div>
          <div class="form-section">
            <div class="form-row">
              <div class="form-row-text">
                <span class="form-label">启用自动重连</span>
                <span class="form-hint">开启后后台定时检查并自动连接/重连</span>
              </div>
              <label class="switch">
                <input type="checkbox" v-model="wsClientConfig.auto_reconnect" />
                <span class="track"><span class="thumb"></span></span>
              </label>
            </div>
            <label class="form-field">
              <span class="form-label">重连地址</span>
              <input v-model="wsClientConfig.url" type="text" placeholder="ws://example.com/ws" />
            </label>
            <div class="form-row-inline">
              <label class="form-field form-field-half">
                <span class="form-label">重连间隔（秒）</span>
                <input v-model="wsClientConfig.reconnect_interval" type="number" min="5" placeholder="10" />
              </label>
              <label class="form-field form-field-half">
                <span class="form-label">心跳间隔（秒）</span>
                <input v-model="wsClientConfig.heartbeat_interval" type="number" min="1" placeholder="30" />
              </label>
            </div>
            <button class="mobile-btn primary full" :disabled="wsClientConfigSaving" @click="saveWsClientConfig">{{ wsClientConfigSaving ? '保存中...' : '保存配置' }}</button>
          </div>
        </section>

        <section class="mobile-card">
          <div class="module-head">
            <div class="module-icon mi-green">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg>
            </div>
            <div class="module-text">
              <h3 class="module-name">消息日志</h3>
              <p class="module-desc">客户端收发消息记录</p>
            </div>
            <button class="mobile-btn danger" @click="clearWsLogs">清空日志</button>
          </div>
          <div v-if="wsLogs.length === 0" class="empty-mini">暂无消息记录</div>
          <div v-else class="log-list">
            <div v-for="(log, i) in wsLogs" :key="i" class="log-item">
              <div class="log-item-head">
                <span class="log-direction" :class="log.direction">{{ log.direction === 'in' ? '接收' : '发送' }}</span>
                <span class="log-time">{{ log.time }}</span>
              </div>
              <pre class="log-body">{{ log.data }}</pre>
            </div>
          </div>
        </section>
      </template>

      <!-- 连接鉴权 -->
      <template v-if="commSubTab === 'auth'">
        <section class="mobile-card">
          <div class="module-head">
            <div class="module-icon mi-red">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
            </div>
            <div class="module-text">
              <h3 class="module-name">连接鉴权</h3>
              <p class="module-desc">设置通信工具服务访问令牌，所有 HTTP / SSE / WebSocket 连接必须携带有效 Token 才能访问。</p>
            </div>
            <span class="status-badge" :class="authForm.token_enabled ? 'on' : 'off'">
              <span class="status-dot"></span>{{ authForm.token_enabled ? '鉴权已开启' : '鉴权未开启' }}
            </span>
          </div>
          <div class="form-section">
            <label class="form-field">
              <span class="form-label">访问令牌 Token</span>
              <div class="input-with-action">
                <input v-model="authForm.token" :type="authShowToken ? 'text' : 'password'" placeholder="留空表示不开启鉴权" />
                <button class="mobile-btn" @click="authShowToken = !authShowToken">{{ authShowToken ? '隐藏' : '显示' }}</button>
              </div>
            </label>
            <p class="form-hint-text">客户端通过 <code>?token=xxx</code> 查询参数、<code>Authorization: Bearer xxx</code> 或 <code>X-Token</code> 请求头携带令牌。</p>
            <button class="mobile-btn primary full" :disabled="authSaving" @click="saveAuthConfig">{{ authSaving ? '保存中...' : '保存配置' }}</button>
          </div>
        </section>

        <section class="mobile-card">
          <div class="module-head">
            <div class="module-icon mi-blue">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>
            </div>
            <div class="module-text">
              <h3 class="module-name">连接地址示例</h3>
              <p class="module-desc">开启鉴权后，客户端需按以下格式携带 Token 连接</p>
            </div>
          </div>
          <div class="hint-codes">
            <div class="hint-code-row">
              <span class="hint-code-label">WebSocket：</span>
              <code>ws://IP:{{ commStatus.server_port || '8090' }}/ws?token=你的令牌</code>
            </div>
            <div class="hint-code-row">
              <span class="hint-code-label">SSE：</span>
              <code>http://IP:{{ commStatus.server_port || '8090' }}/sse?token=你的令牌</code>
            </div>
            <div class="hint-code-row">
              <span class="hint-code-label">HTTP：</span>
              <code>http://IP:{{ commStatus.server_port || '8090' }}/任意路径?token=你的令牌</code>
            </div>
          </div>
        </section>
      </template>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { adminApi, showToast } from '@/api/client'
import { useNotificationStore } from '@/stores/notification'
import './MobilePage.css'
import { mobileConfirm, removeBackdropBlur } from '@/utils/mobileDialog'

interface ModuleItem {
  key: string
  field: string
  label: string
  desc: string
}

const moduleList: ModuleItem[] = [
  { key: 'wallpaper', field: 'notify_wallpaper', label: '壁纸审核', desc: '新壁纸提交待审核' },
  { key: 'avatar', field: 'notify_avatar', label: '头像审核', desc: '新头像提交待审核' },
  { key: 'nickname', field: 'notify_nickname', label: '昵称审核', desc: '新昵称提交待审核' },
  { key: 'feedback', field: 'notify_feedback', label: '反馈更新', desc: '用户提交新反馈' },
]

// ===== 标签页 =====
const notify = useNotificationStore()
const route = useRoute()
const activeTab = ref<'settings' | 'email' | 'webhook' | 'commtool'>('settings')
const commSubTab = ref<'http-server' | 'http-client' | 'sse-server' | 'ws-server' | 'ws-client' | 'auth'>('http-server')

const loading = ref(true)
const saving = ref(false)
const importing = ref(false)
const testingId = ref<number | null>(null)
const list = ref<any[]>([])
const moduleSettings = ref<Record<string, boolean>>({ wallpaper: true, avatar: true, nickname: true, feedback: true })

const stats = computed(() => {
  const total = list.value.length
  const active = list.value.filter(i => i.status == 1).length
  return { total, active, disabled: total - active }
})

async function loadList() {
  loading.value = true
  const [listRes, moduleRes] = await Promise.all([
    adminApi<any[]>('list_notification_emails'),
    adminApi<Record<string, boolean>>('get_notification_modules'),
  ])
  list.value = listRes.code === 200 && Array.isArray(listRes.data) ? listRes.data : []
  if (moduleRes.code === 200 && moduleRes.data) {
    moduleSettings.value = { ...moduleSettings.value, ...moduleRes.data }
  }
  loading.value = false
}

async function toggleModule(key: string) {
  moduleSettings.value[key] = !moduleSettings.value[key]
  const payload: Record<string, boolean> = {}
  moduleList.forEach(m => { payload[m.key] = moduleSettings.value[m.key] })
  const res = await adminApi('update_notification_modules', payload)
  if (res.code !== 200) {
    moduleSettings.value[key] = !moduleSettings.value[key]
    showToast(res.msg || '保存失败')
  }
}

async function toggleModuleOnEmail(item: any, key: string) {
  const field = moduleList.find(m => m.key === key)?.field || ''
  if (!field) return
  item[field] = item[field] == 1 ? 0 : 1
  const res = await adminApi('update_notification_email', {
    id: item.id,
    remark: item.remark || '',
    notify_wallpaper: item.notify_wallpaper == 1 ? 1 : 0,
    notify_avatar: item.notify_avatar == 1 ? 1 : 0,
    notify_nickname: item.notify_nickname == 1 ? 1 : 0,
    notify_feedback: item.notify_feedback == 1 ? 1 : 0,
  })
  if (res.code !== 200) {
    item[field] = item[field] == 1 ? 0 : 1
    showToast(res.msg || '操作失败')
  }
}

async function doImportAdmin() {
  const ok = await mobileConfirm('导入后，后台所有已填写邮箱的管理员账号将自动加入通知邮箱列表（已存在的会跳过）。继续吗？', {
    title: '导入管理员邮箱',
    confirmText: '确认导入',
  })
  if (!ok) return
  importing.value = true
  const res = await adminApi<{ imported: string[]; skipped: string[] }>('import_admin_emails')
  importing.value = false
  if (res.code === 200) {
    showToast(res.msg || '导入成功', 'success')
    loadList()
  } else {
    showToast(res.msg || '导入失败')
  }
}

const addModalVisible = ref(false)
const form = ref<any>({ email: '', remark: '', notify_wallpaper: 1, notify_avatar: 1, notify_nickname: 1, notify_feedback: 1 })

function openAddModal() {
  form.value = { email: '', remark: '', notify_wallpaper: 1, notify_avatar: 1, notify_nickname: 1, notify_feedback: 1 }
  addModalVisible.value = true
}

function closeAddModal() {
  if (saving.value) return
  addModalVisible.value = false
}

function isValidEmail(email: string): boolean {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)
}

async function doAdd() {
  const email = form.value.email.trim()
  if (!email) { showToast('请输入邮箱地址'); return }
  if (!isValidEmail(email)) { showToast('邮箱格式不正确'); return }
  saving.value = true
  const res = await adminApi('add_notification_email', {
    email,
    remark: form.value.remark.trim(),
    notify_wallpaper: form.value.notify_wallpaper,
    notify_avatar: form.value.notify_avatar,
    notify_nickname: form.value.notify_nickname,
    notify_feedback: form.value.notify_feedback,
  })
  saving.value = false
  if (res.code === 200) {
    showToast('添加成功', 'success')
    addModalVisible.value = false
    loadList()
  } else {
    showToast(res.msg || '添加失败')
  }
}

async function toggle(item: any) {
  const res = await adminApi('toggle_notification_email', { id: item.id })
  if (res.code === 200) {
    item.status = item.status == 1 ? 0 : 1
    showToast('已更新', 'success')
  } else {
    showToast(res.msg || '操作失败')
  }
}

async function remove(item: any) {
  const ok = await mobileConfirm(`确认删除通知邮箱 "${item.email}"？`, {
    title: '删除通知邮箱',
    confirmText: '确认删除',
    danger: true,
  })
  if (!ok) return
  const res = await adminApi('delete_notification_email', { id: item.id })
  if (res.code === 200) {
    showToast('删除成功', 'success')
    list.value = list.value.filter(i => i.id !== item.id)
  } else {
    showToast(res.msg || '删除失败')
  }
}

async function sendTest(item: any) {
  testingId.value = item.id
  const res = await adminApi('test_notification_email', { email: item.email })
  testingId.value = null
  if (res.code === 200) {
    showToast(res.msg || '测试通知已发送', 'success')
  } else {
    showToast(res.msg || '发送失败')
  }
}

// ==================== Webhook 通知 ====================

interface WebhookConfig {
  enabled: boolean
  url: string
  method: string
  headers: string
  body_template: string
  modules: Record<string, boolean>
}

const webhookForm = reactive<WebhookConfig>({
  enabled: false,
  url: '',
  method: 'POST',
  headers: '',
  body_template: '',
  modules: {
    wh_wallpaper: true,
    wh_avatar: true,
    wh_nickname: true,
    wh_feedback: true,
  },
})
const webhookSaving = ref(false)
const webhookTesting = ref(false)
const webhookTestResult = ref<{ status: number; body: string } | null>(null)

async function loadWebhookConfig() {
  const res = await adminApi<WebhookConfig>('get_webhook_config')
  if (res.code === 200 && res.data) {
    webhookForm.enabled = res.data.enabled
    webhookForm.url = res.data.url || ''
    webhookForm.method = res.data.method || 'POST'
    webhookForm.headers = res.data.headers || ''
    webhookForm.body_template = res.data.body_template || ''
    if (res.data.modules) {
      webhookForm.modules = { ...webhookForm.modules, ...res.data.modules }
    }
  }
}

async function saveWebhookConfig() {
  if (webhookForm.enabled && !webhookForm.url.trim()) {
    showToast('启用 Webhook 时请输入回调地址')
    return
  }
  webhookSaving.value = true
  const res = await adminApi('save_webhook_config', {
    enabled: webhookForm.enabled,
    url: webhookForm.url.trim(),
    method: webhookForm.method,
    headers: webhookForm.headers,
    body_template: webhookForm.body_template,
    modules: webhookForm.modules,
  })
  webhookSaving.value = false
  if (res.code === 200) {
    showToast(res.msg || '已保存', 'success')
  } else {
    showToast(res.msg || '保存失败')
  }
}

async function testWebhook() {
  if (!webhookForm.url.trim()) {
    showToast('请输入回调地址')
    return
  }
  webhookTesting.value = true
  const res = await adminApi<{ status: number; body: string }>('test_webhook', {
    url: webhookForm.url.trim(),
    method: webhookForm.method,
    headers: webhookForm.headers,
    body_template: webhookForm.body_template,
  })
  webhookTesting.value = false
  if (res.code === 200 && res.data) {
    webhookTestResult.value = res.data
    showToast('测试完成', 'success')
  } else {
    webhookTestResult.value = null
    showToast(res.msg || '测试失败')
  }
}

// ==================== 通信工具 ====================

interface CommStatus {
  server_running: boolean
  server_port: number
  ws_server_count: number
  token_enabled?: boolean
  ws_client: { url: string; connected_at: string } | null
  ws_client_config?: {
    url: string
    auto_reconnect: boolean
    reconnect_interval: string
    heartbeat_interval: string
  }
}
const commStatus = ref<CommStatus>({
  server_running: false,
  server_port: 0,
  ws_server_count: 0,
  ws_client: null,
})

interface HttpLog {
  time: string
  method: string
  path: string
  query: string
  headers: Record<string, string>
  body: string
}
const httpLogs = ref<HttpLog[]>([])

interface WsServerClient {
  id: string
  addr: string
  connected_at: string
  events?: string[]
}
const wsServerClients = ref<WsServerClient[]>([])

interface WsLog {
  time: string
  direction: 'in' | 'out'
  client: string
  type: string
  data: string
}
const wsLogs = ref<WsLog[]>([])

const wsClientConnected = ref(false)

// HTTP 客户端
const httpClientUrl = ref('')
const httpClientMethod = ref('GET')
const httpClientHeaders = ref('')
const httpClientBody = ref('')
const httpClientSending = ref(false)
const httpClientResult = ref<{ status: number; headers: Record<string, string>; body: string; elapsed_ms: number } | null>(null)

// SSE
const sseMessage = ref('')

// WS 服务器
const wsMessage = ref('')
const wsBroadcastMessage = ref('')

// WS 客户端
const wsClientUrl = ref('')
const wsClientMessage = ref('')

// WS 客户端自动重连配置
const wsClientConfig = reactive({
  url: '',
  auto_reconnect: false,
  reconnect_interval: '10',
  heartbeat_interval: '30',
})
const wsClientConfigSaving = ref(false)

// 连接鉴权配置
const authForm = reactive({ token: '', token_enabled: false })
const authShowToken = ref(false)
const authSaving = ref(false)

async function loadWsClientConfig() {
  const res = await adminApi<typeof wsClientConfig>('comm_ws_client_config')
  if (res.code === 200 && res.data) {
    wsClientConfig.url = res.data.url || ''
    wsClientConfig.auto_reconnect = !!res.data.auto_reconnect
    wsClientConfig.reconnect_interval = res.data.reconnect_interval || '10'
    wsClientConfig.heartbeat_interval = res.data.heartbeat_interval || '30'
  }
}

async function saveWsClientConfig() {
  if (wsClientConfig.auto_reconnect && !wsClientConfig.url.trim()) {
    showToast('启用自动重连时请输入重连地址')
    return
  }
  wsClientConfigSaving.value = true
  const res = await adminApi('comm_ws_client_save_config', {
    url: wsClientConfig.url.trim(),
    auto_reconnect: wsClientConfig.auto_reconnect,
    reconnect_interval: wsClientConfig.reconnect_interval.trim() || '10',
    heartbeat_interval: wsClientConfig.heartbeat_interval.trim() || '30',
  })
  wsClientConfigSaving.value = false
  if (res.code === 200) {
    showToast(res.msg || '已保存', 'success')
  } else {
    showToast(res.msg || '保存失败')
  }
}

async function loadAuthConfig() {
  const res = await adminApi<{ token: string; token_enabled: boolean }>('comm_auth_config')
  if (res.code === 200 && res.data) {
    authForm.token = res.data.token || ''
    authForm.token_enabled = !!res.data.token_enabled
  }
}

async function saveAuthConfig() {
  authSaving.value = true
  const res = await adminApi('comm_auth_save_config', { token: authForm.token.trim() })
  authSaving.value = false
  if (res.code === 200) {
    authForm.token_enabled = !!authForm.token.trim()
    showToast(res.msg || '已保存', 'success')
    loadCommStatus()
  } else {
    showToast(res.msg || '保存失败')
  }
}

async function loadCommStatus() {
  const res = await adminApi<CommStatus>('comm_get_status')
  if (res.code === 200 && res.data) {
    commStatus.value = res.data
    wsClientConnected.value = !!res.data.ws_client
    if (res.data.ws_client_config) {
      const cfg = res.data.ws_client_config
      wsClientConfig.url = cfg.url || ''
      wsClientConfig.auto_reconnect = !!cfg.auto_reconnect
      wsClientConfig.reconnect_interval = cfg.reconnect_interval || '10'
      wsClientConfig.heartbeat_interval = cfg.heartbeat_interval || '30'
    }
    if (res.data.token_enabled != null) {
      authForm.token_enabled = res.data.token_enabled
    }
  }
}

async function loadHttpLogs() {
  const res = await adminApi<HttpLog[]>('comm_http_logs', { limit: 100 })
  if (res.code === 200 && Array.isArray(res.data)) {
    httpLogs.value = res.data
  }
}

async function clearHttpLogs() {
  const ok = await mobileConfirm('确认清空所有 HTTP 请求日志？', { title: '清空日志', confirmText: '确认清空' })
  if (!ok) return
  const res = await adminApi('comm_http_clear')
  if (res.code === 200) {
    httpLogs.value = []
    showToast('已清空', 'success')
  } else {
    showToast(res.msg || '清空失败')
  }
}

async function sendHttpClient() {
  if (!httpClientUrl.value.trim()) {
    showToast('请输入请求地址')
    return
  }
  httpClientSending.value = true
  const res = await adminApi<typeof httpClientResult.value>('comm_http_client', {
    url: httpClientUrl.value.trim(),
    method: httpClientMethod.value,
    headers: httpClientHeaders.value,
    body: httpClientBody.value,
  })
  httpClientSending.value = false
  if (res.code === 200 && res.data) {
    httpClientResult.value = res.data
  } else {
    httpClientResult.value = null
    showToast(res.msg || '请求失败')
  }
}

async function sendSsePush() {
  if (!sseMessage.value.trim()) {
    showToast('请输入推送内容')
    return
  }
  const res = await adminApi('comm_sse_push', { message: sseMessage.value })
  if (res.code === 200) {
    showToast(res.msg || '已推送', 'success')
    sseMessage.value = ''
  } else {
    showToast(res.msg || '推送失败')
  }
}

async function loadWsServerClients() {
  const res = await adminApi<WsServerClient[]>('comm_ws_server_list')
  if (res.code === 200 && Array.isArray(res.data)) {
    wsServerClients.value = res.data
  }
}

async function sendWsServerMessage(clientId: string) {
  if (!wsMessage.value.trim()) {
    showToast('请输入消息内容')
    return
  }
  const res = await adminApi('comm_ws_server_send', { id: clientId, message: wsMessage.value })
  if (res.code === 200) {
    showToast(res.msg || '已发送', 'success')
    wsMessage.value = ''
    loadWsLogs()
  } else {
    showToast(res.msg || '发送失败')
  }
}

async function broadcastWsMessage() {
  if (!wsBroadcastMessage.value.trim()) {
    showToast('请输入广播内容')
    return
  }
  const res = await adminApi('comm_ws_server_broadcast', { message: wsBroadcastMessage.value })
  if (res.code === 200) {
    showToast(res.msg || '已广播', 'success')
    wsBroadcastMessage.value = ''
    loadWsLogs()
  } else {
    showToast(res.msg || '广播失败')
  }
}

async function connectWsClient() {
  if (!wsClientUrl.value.trim()) {
    showToast('请输入连接地址')
    return
  }
  const res = await adminApi('comm_ws_client_connect', { url: wsClientUrl.value.trim() })
  if (res.code === 200) {
    wsClientConnected.value = true
    showToast(res.msg || '连接成功', 'success')
    loadCommStatus()
    loadWsLogs()
  } else {
    showToast(res.msg || '连接失败')
  }
}

async function disconnectWsClient() {
  const ok = await mobileConfirm('确认断开 WebSocket 客户端连接？', { title: '断开连接', confirmText: '确认断开' })
  if (!ok) return
  const res = await adminApi('comm_ws_client_disconnect')
  if (res.code === 200) {
    wsClientConnected.value = false
    showToast(res.msg || '已断开', 'success')
    loadCommStatus()
  } else {
    showToast(res.msg || '断开失败')
  }
}

async function sendWsClientMessage() {
  if (!wsClientMessage.value.trim()) {
    showToast('请输入消息内容')
    return
  }
  const res = await adminApi('comm_ws_client_send', { message: wsClientMessage.value })
  if (res.code === 200) {
    showToast(res.msg || '已发送', 'success')
    wsClientMessage.value = ''
    loadWsLogs()
  } else {
    showToast(res.msg || '发送失败')
  }
}

async function loadWsLogs() {
  const res = await adminApi<WsLog[]>('comm_ws_client_logs', { limit: 100 })
  if (res.code === 200 && Array.isArray(res.data)) {
    wsLogs.value = res.data
  }
}

async function clearWsLogs() {
  const ok = await mobileConfirm('确认清空所有 WebSocket 消息日志？', { title: '清空日志', confirmText: '确认清空' })
  if (!ok) return
  const res = await adminApi('comm_ws_clear')
  if (res.code === 200) {
    wsLogs.value = []
    showToast('已清空', 'success')
  } else {
    showToast(res.msg || '清空失败')
  }
}

function formatHeaders(headers: Record<string, string>): string {
  return Object.entries(headers || {}).map(([k, v]) => `${k}: ${v}`).join('\n')
}

// ===== 切换标签时加载数据 =====
watch(activeTab, (tab) => {
  if (tab === 'webhook') {
    loadWebhookConfig()
  } else if (tab === 'commtool') {
    loadCommStatus()
    loadHttpLogs()
    loadWsServerClients()
    loadWsLogs()
  }
})

watch(commSubTab, (sub) => {
  if (sub === 'http-server') {
    loadCommStatus()
    loadHttpLogs()
  } else if (sub === 'sse-server') {
    loadCommStatus()
  } else if (sub === 'ws-server') {
    loadCommStatus()
    loadWsServerClients()
    loadWsLogs()
  } else if (sub === 'ws-client') {
    loadCommStatus()
    loadWsLogs()
    loadWsClientConfig()
  } else if (sub === 'auth') {
    loadCommStatus()
    loadAuthConfig()
  }
})

async function handleRequestPermission() {
  const p = await notify.requestPermission()
  if (p === 'granted') {
    showToast('已允许系统通知', 'success')
  } else if (p === 'denied') {
    showToast('通知权限被拒绝，请在系统设置中开启', 'error')
  }
}

onMounted(() => {
  const tab = route.query.tab as string
  if (['settings', 'email', 'webhook', 'commtool'].includes(tab)) {
    activeTab.value = tab as 'settings' | 'email' | 'webhook' | 'commtool'
  }
  loadList()
  loadWebhookConfig()
  loadCommStatus()
})
</script>

<style scoped>
.notify-header {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.notify-header-info { min-width: 0; }
.notify-title { font-size: 18px; font-weight: 850; margin: 0 0 4px; color: var(--text); }
.notify-desc { font-size: 12px; color: var(--text-light); line-height: 1.6; margin: 0; }
.notify-header-actions {
  display: grid;
  grid-template-columns: 1.4fr 1fr;
  gap: 8px;
}

/* ===== 标签栏 ===== */
.tab-bar {
  display: flex;
  gap: 8px;
  overflow-x: auto;
  white-space: nowrap;
  -webkit-overflow-scrolling: touch;
  scrollbar-width: none;
  padding: 2px;
}
.tab-bar::-webkit-scrollbar { display: none; }
.tab-item {
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 9px 16px;
  border-radius: 999px;
  border: 1px solid var(--border);
  background: var(--control-bg);
  color: var(--text-muted);
  font-size: 13px;
  font-weight: 750;
  cursor: pointer;
  transition: all 0.18s;
}
.tab-item:active { transform: scale(0.95); }
.tab-item.active {
  background: #EC4141;
  border-color: #EC4141;
  color: #fff;
}

/* ===== 子标签栏 ===== */
.sub-tab-bar {
  display: flex;
  gap: 8px;
  overflow-x: auto;
  white-space: nowrap;
  -webkit-overflow-scrolling: touch;
  scrollbar-width: none;
  padding: 2px;
  margin-top: -2px;
}
.sub-tab-bar::-webkit-scrollbar { display: none; }
.sub-tab-item {
  flex-shrink: 0;
  padding: 8px 14px;
  border-radius: 999px;
  border: 1px solid var(--border);
  background: var(--control-bg);
  color: var(--text-muted);
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.18s;
}
.sub-tab-item:active { transform: scale(0.95); }
.sub-tab-item.active {
  background: var(--accent);
  border-color: var(--accent);
  color: #fff;
}

/* 通知板块 */
.module-head {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  margin-bottom: 12px;
}
.module-icon {
  width: 36px;
  height: 36px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  background: #eff6ff;
  color: #3b82f6;
}
.mi-orange { background: #fffbeb; color: #f59e0b; }
.mi-blue { background: #eff6ff; color: #3b82f6; }
.mi-green { background: #f0fdf4; color: #16a34a; }
.mi-purple { background: #faf5ff; color: #a855f7; }
.mi-pink { background: #fdf2f8; color: #ec4899; }
.mi-cyan { background: #ecfeff; color: #06b6d4; }
.mi-red { background: #fef2f2; color: #ef4444; }
.module-text { min-width: 0; flex: 1; }
.module-name { font-size: 15px; font-weight: 750; margin: 0; color: var(--text); }
.module-desc { font-size: 11px; color: var(--text-muted); line-height: 1.5; margin: 2px 0 0; }
.module-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.module-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  background: var(--control-bg);
  border-radius: 12px;
}
.module-item-icon {
  width: 30px;
  height: 30px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.mi-wallpaper { background: #f0fdf4; color: #16a34a; }
.mi-avatar { background: #eff6ff; color: #3b82f6; }
.mi-nickname { background: #fffbeb; color: #f59e0b; }
.mi-feedback { background: #fef2f2; color: #ef4444; }
.module-item-text {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
  flex: 1;
}
.module-item-name { font-size: 13px; font-weight: 650; color: var(--text); }
.module-item-desc { font-size: 11px; color: var(--text-muted); }

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
  background: var(--white);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
  transition: transform 0.25s;
}
.switch input:checked + .track { background: #EC4141; }
.switch input:checked + .track .thumb { transform: translateX(18px); }

/* ===== 通知设置 ===== */
.notify-perm-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.notify-perm-ok {
  padding: 8px 10px;
  border-radius: 10px;
  background: rgba(34, 197, 94, 0.08);
}
.notify-hint {
  font-size: 11px;
  color: var(--text-muted);
  line-height: 1.5;
}
.notify-tip {
  margin-top: 10px;
  font-size: 11px;
  color: var(--text-muted);
  line-height: 1.5;
}
.notify-tip.warn {
  padding: 8px 10px;
  border-radius: 10px;
  background: rgba(236, 65, 65, 0.08);
  color: #EC4141;
}
.notify-settings-btn {
  display: inline-block;
  margin-top: 8px;
  padding: 6px 12px;
  border: none;
  border-radius: 8px;
  background: #EC4141;
  color: #fff;
  font-size: 12px;
  cursor: pointer;
}
.notify-settings-btn:active {
  opacity: 0.85;
}
.notify-divider {
  height: 1px;
  margin: 10px 0;
  background: var(--border);
}
.notify-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 5px 0;
}
.notify-row-label {
  display: inline-flex;
  flex-direction: column;
  font-size: 13px;
  font-weight: 600;
  color: var(--text);
}
.notify-row-desc {
  font-size: 11px;
  font-weight: 400;
  color: var(--text-muted);
}
.settings-modules {
  display: flex;
  flex-direction: column;
}

/* 统计 */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 10px;
}
.stat-chip {
  border: 1px solid var(--border);
  border-radius: 16px;
  background: var(--card);
  padding: 12px;
  display: flex;
  align-items: center;
  gap: 10px;
  box-shadow: var(--shadow-soft);
}
.stat-icon {
  width: 34px;
  height: 34px;
  border-radius: 9px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.stat-icon-total { background: #eff6ff; color: #3b82f6; }
.stat-icon-active { background: #f0fdf4; color: #16a34a; }
.stat-icon-disabled { background: #fef2f2; color: #ef4444; }
.stat-body { display: flex; flex-direction: column; min-width: 0; }
.stat-label { font-size: 10px; color: var(--text-muted); }
.stat-num { font-size: 18px; font-weight: 850; line-height: 1.2; color: var(--text); }

/* 列表 */
.mobile-item.is-disabled { opacity: 0.6; }
.mobile-admin-left {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}
.notify-avatar {
  width: 40px;
  height: 40px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.avatar-active { background: #eff6ff; color: #3b82f6; }
.avatar-disabled { background: #f3f4f6; color: #9ca3af; }

.notify-modules {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 10px;
}
.notify-module-tag {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 4px 10px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
  user-select: none;
  transition: transform 0.15s;
}
.notify-module-tag:active { transform: scale(0.94); }
.notify-module-tag .tag-dot { width: 6px; height: 6px; border-radius: 50%; }
.notify-module-tag.tag-on { background: rgba(34, 197, 94, 0.12); color: #16a34a; }
.notify-module-tag.tag-on .tag-dot { background: #16a34a; }
.notify-module-tag.tag-off { background: var(--control-bg); color: var(--text-muted); }
.notify-module-tag.tag-off .tag-dot { background: var(--text-muted); }

.empty-title { font-size: 14px; font-weight: 700; color: var(--text-light); margin-bottom: 4px; }
.empty-sub { font-size: 12px; color: var(--text-muted); }

/* ===== 表单 ===== */
.form-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-top: 12px;
}
.form-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 10px 12px;
  background: var(--control-bg);
  border-radius: 12px;
}
.form-row-text { display: flex; flex-direction: column; gap: 2px; min-width: 0; }
.form-row-inline {
  display: flex;
  gap: 10px;
  align-items: flex-end;
}
.form-field { display: flex; flex-direction: column; gap: 6px; }
.form-field-half { flex: 1; min-width: 0; }
.form-label { font-size: 12px; font-weight: 700; color: var(--text-light); }
.form-hint { font-size: 11px; color: var(--text-muted); }
.form-field input,
.form-field select,
.form-field textarea {
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 10px 12px;
  font-size: 14px;
  font-family: inherit;
  outline: none;
  background: var(--control-bg);
  color: var(--text);
  transition: border-color 0.18s, box-shadow 0.18s;
  width: 100%;
  box-sizing: border-box;
}
.form-field input:focus,
.form-field select:focus,
.form-field textarea:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 3px var(--accent-soft);
}
.form-field input:disabled { opacity: 0.6; }
.form-field textarea { resize: vertical; line-height: 1.5; }
.form-actions {
  display: flex;
  gap: 10px;
}
.form-actions .mobile-btn { flex: 1; }
.mobile-btn.full { width: 100%; padding: 11px; font-size: 14px; }

/* ===== 状态徽章 ===== */
.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 4px 10px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 750;
  flex-shrink: 0;
}
.status-badge .status-dot { width: 6px; height: 6px; border-radius: 50%; }
.status-badge.on { background: rgba(34, 197, 94, 0.12); color: #16a34a; }
.status-badge.on .status-dot { background: #16a34a; }
.status-badge.off { background: var(--control-bg); color: var(--text-muted); }
.status-badge.off .status-dot { background: var(--text-muted); }

/* ===== 信息行 ===== */
.info-line {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 12px;
  color: var(--text-light);
  word-break: break-all;
}
.info-line code {
  font-family: monospace;
  font-size: 11px;
  background: var(--control-bg);
  padding: 2px 6px;
  border-radius: 6px;
  color: var(--text);
}

/* ===== 鉴权面板 ===== */
.input-with-action {
  display: flex;
  gap: 8px;
  align-items: center;
}
.input-with-action input { flex: 1; min-width: 0; }
.input-with-action .mobile-btn {
  flex-shrink: 0;
  padding: 9px 14px;
  font-size: 12px;
}
.form-hint-text {
  font-size: 11px;
  line-height: 1.6;
  color: var(--text-muted);
  margin-top: 2px;
}
.form-hint-text code {
  font-family: monospace;
  font-size: 10px;
  background: var(--control-bg);
  padding: 1px 6px;
  border-radius: 5px;
  color: var(--accent);
}
.hint-codes {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.hint-code-row {
  display: flex;
  flex-direction: column;
  gap: 3px;
  padding: 9px 11px;
  background: var(--control-bg);
  border-radius: 10px;
}
.hint-code-label {
  font-size: 12px;
  font-weight: 700;
  color: var(--text);
}
.hint-code-row code {
  font-family: monospace;
  font-size: 11px;
  color: var(--accent);
  word-break: break-all;
}
.info-hint { margin: 8px 0 0; font-size: 11px; color: var(--text-muted); }

/* ===== 结果盒子 ===== */
.result-box {
  margin-top: 14px;
  border: 1px solid var(--border);
  border-radius: 14px;
  overflow: hidden;
  background: var(--control-bg);
}
.result-head {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  padding: 10px 12px;
  font-size: 12px;
  font-weight: 750;
  color: var(--text);
  border-bottom: 1px solid var(--border);
}
.result-elapsed { margin-left: auto; font-size: 11px; color: var(--text-muted); font-weight: 600; }
.result-body { padding: 10px 12px; }
.result-label { margin: 0 0 6px; font-size: 11px; color: var(--text-muted); }
.result-body pre {
  margin: 0 0 10px;
  font-family: monospace;
  font-size: 11px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-all;
  color: var(--text-light);
  background: var(--card);
  padding: 8px 10px;
  border-radius: 8px;
  border: 1px solid var(--border);
}
.result-body pre:last-child { margin-bottom: 0; }

/* ===== 日志 ===== */
.empty-mini {
  padding: 18px 0;
  text-align: center;
  font-size: 12px;
  color: var(--text-muted);
}
.log-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 340px;
  overflow-y: auto;
}
.log-item {
  border: 1px solid var(--border);
  border-radius: 12px;
  background: var(--control-bg);
  padding: 8px 10px;
}
.log-item-head {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.log-method {
  font-size: 10px;
  font-weight: 850;
  padding: 2px 8px;
  border-radius: 999px;
  background: var(--control-bg);
}
.log-method.GET { background: rgba(34, 197, 94, 0.12); color: #16a34a; }
.log-method.POST { background: rgba(59, 130, 246, 0.12); color: #3b82f6; }
.log-method.PUT { background: rgba(245, 158, 11, 0.12); color: #f59e0b; }
.log-method.DELETE { background: rgba(239, 68, 68, 0.12); color: #ef4444; }
.log-path {
  font-size: 11px;
  color: var(--text);
  font-weight: 650;
  word-break: break-all;
  flex: 1;
  min-width: 0;
}
.log-time { font-size: 10px; color: var(--text-muted); }
.log-direction {
  font-size: 10px;
  font-weight: 850;
  padding: 2px 8px;
  border-radius: 999px;
}
.log-direction.in { background: rgba(34, 197, 94, 0.12); color: #16a34a; }
.log-direction.out { background: rgba(59, 130, 246, 0.12); color: #3b82f6; }
.log-body {
  margin: 6px 0 0;
  font-family: monospace;
  font-size: 11px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-all;
  color: var(--text-light);
}

/* ===== 客户端列表 ===== */
.client-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.client-item {
  border: 1px solid var(--border);
  border-radius: 12px;
  background: var(--control-bg);
  padding: 10px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.client-info {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  flex-wrap: wrap;
}
.client-id {
  font-family: monospace;
  font-size: 12px;
  font-weight: 750;
  color: var(--text);
}
.client-time { font-size: 10px; color: var(--text-muted); }
.client-events {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  width: 100%;
}
.client-event-tag {
  font-size: 10px;
  padding: 1px 8px;
  border-radius: 6px;
  background: #eff6ff;
  color: #3b82f6;
}
.client-event-tag.tag-all {
  background: #f0fdf4;
  color: #16a34a;
}
.client-row {
  display: flex;
  gap: 8px;
}
.client-row input {
  flex: 1;
  min-width: 0;
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 9px 11px;
  font-size: 13px;
  font-family: inherit;
  outline: none;
  background: var(--card);
  color: var(--text);
}
.client-row input:focus { border-color: var(--accent); }

/* 弹窗 */
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
  max-width: 340px;
  border-radius: 22px;
  background: var(--card-solid, var(--card));
  box-shadow: 0 24px 60px rgba(15, 23, 42, 0.22);
  overflow: hidden;
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
}
.modal-field { display: flex; flex-direction: column; gap: 6px; }
.modal-field span { font-size: 12px; font-weight: 700; color: var(--text-light); }
.modal-field span.required::after { content: '*'; color: #ef4444; margin-left: 2px; }
.modal-field input {
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 11px 12px;
  font-size: 14px;
  font-family: inherit;
  outline: none;
  background: var(--control-bg);
  color: var(--text);
  transition: border-color 0.18s, box-shadow 0.18s;
  width: 100%;
  box-sizing: border-box;
}
.modal-field input:focus { border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-soft); }
.modal-modules {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}
.modal-module-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 9px 11px;
  border: 1px solid var(--border);
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.18s;
  background: transparent;
}
.modal-module-item input { display: none; }
.modal-module-check {
  width: 18px;
  height: 18px;
  border-radius: 5px;
  border: 1.5px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: center;
  color: transparent;
  flex-shrink: 0;
  transition: all 0.18s;
}
.modal-module-item.on .modal-module-check {
  background: #EC4141;
  border-color: #EC4141;
  color: #fff;
}
.modal-module-name { font-size: 12px; font-weight: 650; color: var(--text); }
.modal-tip { margin: 0; font-size: 11px; color: var(--text-muted); line-height: 1.5; }
.modal-foot {
  display: flex;
  gap: 10px;
  padding: 14px 20px 18px;
}
.modal-btn {
  flex: 1;
  padding: 11px;
  border-radius: 12px;
  font-size: 14px;
  font-weight: 750;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  transition: all 0.18s;
}
.modal-btn.cancel { border: 1px solid var(--border); background: transparent; color: var(--text-muted); }
.modal-btn.cancel:active { background: var(--control-bg); }
.modal-btn.save { border: none; background: #EC4141; color: #fff; }
.modal-btn.save:disabled { opacity: 0.55; }

/* 过渡动画 */
.modal-enter-active, .modal-leave-active { transition: opacity 0.24s var(--motion, cubic-bezier(0.16, 1, 0.3, 1)); }
.modal-enter-from, .modal-leave-to { opacity: 0; }
.modal-enter-active .modal-dialog { animation: modalIn 0.24s cubic-bezier(0.16, 1, 0.3, 1) forwards; }
.modal-leave-active .modal-dialog { animation: modalOut 0.2s ease forwards; }
@keyframes modalIn {
  from { opacity: 0; transform: scale(0.94); }
  to { opacity: 1; transform: scale(1); }
}
@keyframes modalOut {
  from { opacity: 1; transform: scale(1); }
  to { opacity: 0; transform: scale(0.96); }
}
</style>