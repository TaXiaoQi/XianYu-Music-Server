<template>
  <div class="notify-page">
    <!-- 页面头部 -->
    <Transition name="fade-down" appear>
      <div class="page-header">
        <div class="header-info">
          <h2 class="page-title">外部通知</h2>
          <p class="page-desc">管理用于接收后台状态通知的绑定邮箱，并可分别设置壁纸审核、头像审核、昵称审核、反馈更新等板块的通知开关。</p>
        </div>
      </div>
    </Transition>

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
    <div v-if="activeTab === 'settings'" class="tab-panel">
      <Transition name="fade-up" appear>
        <div class="settings-card">
          <div class="settings-card-head">
            <div class="module-title">
              <span class="module-icon">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 0 1-3.46 0"/></svg>
              </span>
              <div>
                <h3 class="module-name">通知设置</h3>
                <p class="module-desc">配置浏览器 / 系统通知的授权状态、总开关与各板块开关。</p>
              </div>
            </div>
          </div>

          <div v-if="!notify.supportedByBrowser" class="notify-tip warn">
            当前环境不支持系统通知，请改用最新版 Chrome / Edge / Firefox 或系统浏览器打开。
          </div>

          <template v-else>
            <div v-if="notify.permission === 'default'" class="notify-perm-row">
              <button class="btn btn-primary btn-sm" @click="handleRequestPermission">授权通知</button>
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

            <div class="settings-row">
              <span class="settings-row-label">启用通知</span>
              <span class="switch" :class="{ on: notify.enabled }" @click="notify.setEnabled(!notify.enabled)"><span class="switch-knob"></span></span>
            </div>

            <div class="settings-modules">
              <div v-for="m in notify.moduleList" :key="m.key" class="settings-row">
                <span class="settings-row-label">
                  {{ m.label }}
                  <span class="settings-row-desc">{{ m.desc }}</span>
                </span>
                <span class="switch" :class="{ on: notify.modules[m.key] }" @click="notify.toggleModule(m.key)"><span class="switch-knob"></span></span>
              </div>
            </div>

            <div class="notify-divider"></div>

            <button class="btn btn-sm" :disabled="!notify.granted" @click="notify.testNotification()">发送测试通知</button>
            <div class="notify-tip">有新反馈或待审核项时，将弹出系统通知提醒。</div>
          </template>
        </div>
      </Transition>
    </div>

    <!-- ==================== 通知邮箱 Tab ==================== -->
    <div v-if="activeTab === 'email'">
      <!-- 顶部操作区 -->
      <Transition name="fade-up" appear>
        <div class="email-actions">
          <button class="btn-ghost" @click="doImportAdmin" :disabled="importing">
            <span v-if="importing" class="btn-spinner btn-spinner-dark"></span>
            <svg v-else width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
            导入管理员邮箱
          </button>
          <button class="btn-add" @click="openAddModal">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
            </svg>
            新增通知邮箱
          </button>
        </div>
      </Transition>

      <!-- 通知板块设置 -->
      <Transition name="fade-up" appear>
        <div class="module-block">
          <div class="module-head">
            <div class="module-title">
              <span class="module-icon">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
              </span>
              <div>
                <h3 class="module-name">通知板块</h3>
                <p class="module-desc">全局开关，控制哪些板块默认发送通知。关闭后该板块所有邮箱将不再收到通知；开启后下方邮箱可单独关闭。</p>
              </div>
            </div>
          </div>
          <div class="module-grid">
            <div
              v-for="mod in moduleList"
              :key="mod.key"
              class="module-item"
            >
              <div class="module-item-info">
                <span class="module-item-icon" :class="'mi-' + mod.key">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M3 9h18"/><path d="M9 21V9"/></svg>
                </span>
                <div class="module-item-text">
                  <span class="module-item-name">{{ mod.label }}</span>
                  <span class="module-item-desc">{{ mod.desc }}</span>
                </div>
              </div>
              <button
                class="switch"
                :class="{ on: moduleSettings[mod.key] }"
                @click="toggleModule(mod.key)"
              >
                <span class="switch-knob"></span>
              </button>
            </div>
          </div>
        </div>
      </Transition>

      <!-- 统计卡片 -->
      <Transition name="fade-up" appear>
        <div class="stats-row">
          <div class="stat-chip">
            <div class="stat-icon stat-icon-total">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="4" width="20" height="16" rx="2"/><path d="M22 7l-10 6L2 7"/></svg>
            </div>
            <div class="stat-body"><span class="stat-num">{{ stats.total }}</span><span class="stat-label">总数</span></div>
          </div>
          <div class="stat-chip">
            <div class="stat-icon stat-icon-active">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
            </div>
            <div class="stat-body"><span class="stat-num">{{ stats.active }}</span><span class="stat-label">启用中</span></div>
          </div>
          <div class="stat-chip">
            <div class="stat-icon stat-icon-disabled">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="4.93" y1="4.93" x2="19.07" y2="19.07"/></svg>
            </div>
            <div class="stat-body"><span class="stat-num">{{ stats.disabled }}</span><span class="stat-label">已停用</span></div>
          </div>
        </div>
      </Transition>

      <!-- 加载中 -->
      <div v-if="loading" class="state-box">
        <div class="spinner"></div>
        <span>加载中...</span>
      </div>

      <template v-else>
        <!-- 空状态 -->
        <Transition name="fade-up" appear v-if="list.length === 0">
          <div class="state-box state-empty">
            <div class="empty-icon">
              <svg width="56" height="56" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2"><rect x="2" y="4" width="20" height="16" rx="2"/><path d="M22 7l-10 6L2 7"/></svg>
            </div>
            <p class="empty-title">暂无通知邮箱</p>
            <p class="empty-sub">点击右上角"新增通知邮箱"添加，用于接收后台状态通知</p>
          </div>
        </Transition>

        <!-- 通知邮箱列表 -->
        <div v-else class="notify-list">
          <TransitionGroup name="notify-card">
            <div
              v-for="(item, idx) in list"
              :key="item.id"
              class="notify-card"
              :class="{ 'is-disabled': item.status == 0 }"
              :style="{ animationDelay: `${idx * 60}ms` }"
            >
              <div class="notify-avatar" :class="item.status == 1 ? 'avatar-active' : 'avatar-disabled'">
                <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="4" width="20" height="16" rx="2"/><path d="M22 7l-10 6L2 7"/></svg>
              </div>
              <div class="notify-main">
                <div class="notify-info">
                  <div class="notify-row">
                    <span class="notify-email">{{ item.email }}</span>
                    <span class="notify-status" :class="item.status == 1 ? 'st-active' : 'st-disabled'">
                      <span class="status-dot"></span>{{ item.status == 1 ? '启用中' : '已停用' }}
                    </span>
                  </div>
                  <p class="notify-remark">{{ item.remark || '暂无备注' }}</p>
                  <p class="notify-time">创建时间：{{ fmtDateTime(item.created_at) || '-' }}</p>
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
                </div>
                <div class="notify-actions">
                  <button class="btn-action" @click="sendTest(item)">发送测试</button>
                  <button class="btn-action" @click="toggle(item)">{{ item.status == 1 ? '停用' : '启用' }}</button>
                  <button class="btn-action btn-danger" @click="remove(item)">删除</button>
                </div>
              </div>
            </div>
          </TransitionGroup>
        </div>
      </template>

      <!-- 新增通知邮箱弹窗 -->
      <Transition name="modal">
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
                    <input
                      type="checkbox"
                      v-model="form[mod.field]"
                      :true-value="1"
                      :false-value="0"
                    />
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
              <button class="btn-cancel" @click="closeAddModal">取消</button>
              <button class="btn-save" :disabled="saving" @click="doAdd">
                <span v-if="saving" class="btn-spinner"></span>
                {{ saving ? '添加中...' : '确认添加' }}
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </div>

    <!-- ==================== Webhook 通知 Tab ==================== -->
    <div v-if="activeTab === 'webhook'" class="tab-panel">
      <div class="module-block">
        <div class="module-head">
          <div class="module-title">
            <span class="module-icon mi-orange">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg>
            </span>
            <div>
              <h3 class="module-name">Webhook 配置</h3>
              <p class="module-desc">配置回调地址与请求模板，后台发生审核/反馈事件时自动推送通知到指定地址。</p>
            </div>
          </div>
        </div>
        <div class="form-section">
          <div class="form-row">
            <div class="form-row-text">
              <span class="form-label">启用 Webhook</span>
              <span class="form-hint">开启后才会向回调地址发送通知</span>
            </div>
            <button
              class="switch"
              :class="{ on: webhookForm.enabled }"
              @click="webhookForm.enabled = !webhookForm.enabled"
            >
              <span class="switch-knob"></span>
            </button>
          </div>
          <label class="form-field">
            <span class="form-label">回调地址</span>
            <input v-model="webhookForm.url" type="text" placeholder="https://example.com/webhook" />
          </label>
          <label class="form-field form-field-short">
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
      </div>

      <div class="module-block">
        <div class="module-head">
          <div class="module-title">
            <span class="module-icon mi-blue">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
            </span>
            <div>
              <h3 class="module-name">通知板块</h3>
              <p class="module-desc">选择哪些板块触发 Webhook 通知。开启后对应板块发生事件时即会推送。</p>
            </div>
          </div>
        </div>
        <div class="module-grid">
          <div
            v-for="mod in moduleList"
            :key="mod.key"
            class="module-item"
          >
            <div class="module-item-info">
              <span class="module-item-icon" :class="'mi-' + mod.key">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M3 9h18"/><path d="M9 21V9"/></svg>
              </span>
              <div class="module-item-text">
                <span class="module-item-name">{{ mod.label }}</span>
                <span class="module-item-desc">{{ mod.desc }}</span>
              </div>
            </div>
            <button
              class="switch"
              :class="{ on: webhookForm.modules['wh_' + mod.key] }"
              @click="toggleWebhookModule(mod.key)"
            >
              <span class="switch-knob"></span>
            </button>
          </div>
        </div>
      </div>

      <div class="module-block">
        <div class="form-actions">
          <button class="btn-ghost" :disabled="webhookTesting" @click="testWebhook">
            <span v-if="webhookTesting" class="btn-spinner btn-spinner-dark"></span>
            {{ webhookTesting ? '测试中...' : '发送测试' }}
          </button>
          <button class="btn-add" :disabled="webhookSaving" @click="saveWebhookConfig">
            <span v-if="webhookSaving" class="btn-spinner"></span>
            {{ webhookSaving ? '保存中...' : '保存配置' }}
          </button>
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
      </div>
    </div>

    <!-- ==================== 通信工具 Tab ==================== -->
    <div v-if="activeTab === 'commtool'" class="tab-panel">

      <!-- ====== 服务管理 ====== -->
      <Transition name="fade-up" appear>
        <div class="comm-service-card">
          <div class="service-head">
            <div class="service-title">
              <span class="service-icon mi-server">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="2" width="20" height="8" rx="2"/><rect x="2" y="14" width="20" height="8" rx="2"/></svg>
              </span>
              <div>
                <h3>服务管理</h3>
                <p>管理 HTTP/SSE/WebSocket 多协议通信服务的启用状态与端口配置</p>
              </div>
            </div>
            <button class="switch" :class="{ on: commService.enabled }" @click="toggleService">
              <span class="switch-knob"></span>
            </button>
          </div>

          <div class="service-config" v-if="commService.enabled">
            <div class="service-config-row">
              <label class="field">
                <span>监听端口</span>
                <input v-model="commService.port" type="number" min="1024" max="65535" />
              </label>
              <span class="service-status-badge" :class="commStatus.server_running ? 'on' : 'off'">
                <span class="status-dot"></span>{{ commStatus.server_running ? '运行中' : '未运行' }}
              </span>
            </div>
            <div class="service-actions">
              <button class="btn-add" :disabled="serviceSaving" @click="saveServiceConfig">
                <span v-if="serviceSaving" class="btn-spinner"></span>
                {{ serviceSaving ? '保存中...' : '保存配置' }}
              </button>
              <button class="btn-ghost" @click="openAuthDialog">
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
                连接鉴权
              </button>
            </div>
          </div>
        </div>
      </Transition>

      <!-- ====== 客户端管理 ====== -->
      <Transition name="fade-up" appear>
        <div class="module-block">
          <div class="module-head">
            <div class="module-title">
              <span class="module-icon mi-green">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg>
              </span>
              <div>
                <h3 class="module-name">客户端管理</h3>
                <p class="module-desc">管理外部通信客户端连接，支持 HTTP 回调查看、WebSocket 客户端自动重连、SSE 推送等。</p>
              </div>
            </div>
            <button class="btn-add" @click="openAddClientModal">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
              添加客户端
            </button>
          </div>

          <!-- 已添加客户端 -->
          <div v-if="commClients.length > 0" class="added-clients">
            <div class="added-clients-head">
              <span class="added-clients-title">已添加客户端</span>
              <span class="added-clients-count">{{ commClients.length }} 个</span>
            </div>
            <div class="added-client-list">
              <div v-for="c in commClients" :key="c.id" class="added-client-item" :class="{ disabled: !c.enabled }">
                <span class="added-client-type" :class="'t-' + c.type">{{ c.type.toUpperCase() }}</span>
                <div class="added-client-info">
                  <strong>{{ c.name }}</strong>
                  <span class="added-client-url">{{ c.url }}</span>
                  <span v-if="c.events" class="added-client-events">订阅：{{ c.events }}</span>
                </div>
                <button class="switch" :class="{ on: c.enabled }" @click="toggleClientEnabled(c)">
                  <span class="switch-knob"></span>
                </button>
                <button v-if="c.type === 'ws'" class="btn-ghost btn-small" @click.stop="connectClient(c)">
                  <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M5 12h14M12 5l7 7-7 7"/></svg>
                  连接
                </button>
                <button class="btn-ghost btn-small btn-danger" @click.stop="deleteClient(c)">
                  <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
                  删除
                </button>
              </div>
            </div>
          </div>
          <div v-else class="added-clients-empty">
            尚未添加客户端，点击右上角「添加客户端」创建外部连接。
          </div>

          <!-- 客户端列表 -->
          <div class="client-grid">
            <!-- HTTP 客户端 -->
            <div class="client-card" :class="{ expanded: expandedClient === 'http' }" @click="toggleClient('http')">
              <div class="client-card-head">
                <div class="client-card-info">
                  <span class="client-card-icon mi-blue">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 12h-4l-3 9L9 3l-3 9H2"/></svg>
                  </span>
                  <div>
                    <strong>HTTP 客户端</strong>
                    <span>发送 HTTP 请求调试接口与验证回调</span>
                  </div>
                </div>
                <span class="client-expand-icon">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="6 9 12 15 18 9"/></svg>
                </span>
              </div>
              <div class="client-card-body">
                <div class="form-section">
                  <div class="form-row-inline">
                    <label class="form-field form-field-grow">
                      <span class="form-label">请求地址</span>
                      <input v-model="httpClientUrl" type="text" placeholder="https://example.com/api" />
                    </label>
                    <label class="form-field form-field-method">
                      <span class="form-label">请求方法</span>
                      <select v-model="httpClientMethod">
                        <option value="GET">GET</option>
                        <option value="POST">POST</option>
                      </select>
                    </label>
                  </div>
                  <label class="form-field">
                    <span class="form-label">请求头（每行 Key: Value）</span>
                    <textarea v-model="httpClientHeaders" rows="2" placeholder="Authorization: Bearer xxx"></textarea>
                  </label>
                  <label class="form-field">
                    <span class="form-label">请求体</span>
                    <textarea v-model="httpClientBody" rows="3" placeholder='{"key": "value"}'></textarea>
                  </label>
                  <div class="form-actions">
                    <button class="btn-add" :disabled="httpClientSending" @click="sendHttpClient">
                      <span v-if="httpClientSending" class="btn-spinner"></span>
                      {{ httpClientSending ? '发送中...' : '发送请求' }}
                    </button>
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
                </div>
              </div>
            </div>

            <!-- WebSocket 客户端 -->
            <div class="client-card" :class="{ expanded: expandedClient === 'ws' }" @click="toggleClient('ws')">
              <div class="client-card-head">
                <div class="client-card-info">
                  <span class="client-card-icon mi-cyan">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>
                  </span>
                  <div>
                    <strong>WebSocket 客户端</strong>
                    <span>连接外部 WebSocket 服务，支持自动重连与心跳</span>
                  </div>
                </div>
                <span class="client-expand-icon">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="6 9 12 15 18 9"/></svg>
                </span>
              </div>
              <div class="client-card-body">
                <div class="form-section">
                  <div class="form-row">
                    <div class="form-row-text">
                      <span class="form-label">启用自动重连</span>
                      <span class="form-hint">开启后，后台定时检查并按配置自动连接/重连</span>
                    </div>
                    <button class="switch" :class="{ on: wsClientConfig.auto_reconnect }" @click="wsClientConfig.auto_reconnect = !wsClientConfig.auto_reconnect">
                      <span class="switch-knob"></span>
                    </button>
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
                  <div class="form-actions">
                    <button class="btn-add" :disabled="wsClientConfigSaving" @click="saveWsClientConfig">
                      <span v-if="wsClientConfigSaving" class="btn-spinner"></span>
                      {{ wsClientConfigSaving ? '保存中...' : '保存配置' }}
                    </button>
                    <button v-if="!wsClientConnected" class="btn-ghost" @click="manualConnectWs">
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M5 12h14M12 5l7 7-7 7"/></svg>
                      手动连接
                    </button>
                    <button v-else class="btn-ghost btn-danger" @click="disconnectWsClient">
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
                      断开连接
                    </button>
                  </div>
                  <div v-if="wsClientConnected" class="ws-client-row">
                    <div class="form-row-inline">
                      <label class="form-field form-field-grow">
                        <span class="form-label">发送消息</span>
                        <input v-model="wsClientMessage" type="text" placeholder="输入要发送的消息..." @keydown.enter="sendWsClientMessage" />
                      </label>
                      <button class="btn-add btn-small" style="margin-top:22px" @click="sendWsClientMessage">发送</button>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <!-- SSE 服务器 -->
            <div class="client-card" :class="{ expanded: expandedClient === 'sse' }" @click="toggleClient('sse')">
              <div class="client-card-head">
                <div class="client-card-info">
                  <span class="client-card-icon mi-purple">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/></svg>
                  </span>
                  <div>
                    <strong>SSE 推送</strong>
                    <span>服务端推送事件，连接的客户端可实时接收消息</span>
                  </div>
                </div>
                <span class="client-expand-icon">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="6 9 12 15 18 9"/></svg>
                </span>
              </div>
              <div class="client-card-body">
                <div class="info-line">
                  <span>SSE 地址：<code>http://服务器IP:{{ commStatus.server_port || '8090' }}/sse</code></span>
                </div>
                <div class="form-section">
                  <label class="form-field">
                    <span class="form-label">推送消息</span>
                    <textarea v-model="sseMessage" rows="3" placeholder="输入要推送的内容"></textarea>
                  </label>
                  <div class="form-actions">
                    <button class="btn-add" @click="sendSsePush">推送消息</button>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </Transition>

      <!-- ====== 连接日志 ====== -->
      <Transition name="fade-up" appear>
        <div class="module-block">
          <div class="module-head">
            <div class="module-title">
              <span class="module-icon mi-orange">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg>
              </span>
              <div>
                <h3 class="module-name">连接日志</h3>
                <p class="module-desc">HTTP 请求记录与 WebSocket 消息日志</p>
              </div>
            </div>
            <button class="btn-ghost btn-small" @click="clearAllLogs">清空日志</button>
          </div>
          <div v-if="httpLogs.length === 0 && wsLogs.length === 0" class="empty-mini">暂无日志记录</div>
          <div v-else class="log-merged">
            <div v-for="(log, i) in mergedLogs" :key="i" class="log-item">
              <div class="log-item-head">
                <span class="log-source" :class="log.source">{{ log.source === 'http' ? 'HTTP' : 'WS' }}</span>
                <span v-if="log.method" class="log-method" :class="log.method">{{ log.method }}</span>
                <span v-if="log.path" class="log-path">{{ log.path }}</span>
                <span v-if="log.direction" class="log-direction" :class="log.direction">{{ log.direction === 'in' ? '接收' : '发送' }}</span>
                <span class="log-time">{{ log.time }}</span>
              </div>
              <pre v-if="log.data || log.body" class="log-body">{{ log.data || log.body }}</pre>
            </div>
          </div>
        </div>
      </Transition>

      <!-- 连接鉴权弹窗 -->
      <Transition name="modal">
        <div v-if="authDialogVisible" class="modal-backdrop">
          <div class="modal-dialog">
            <div class="modal-head">
              <h3>连接鉴权</h3>
              <button class="modal-close" @click="closeAuthDialog">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
              </button>
            </div>
            <div class="modal-body">
              <div class="modal-field">
                <span>访问令牌 Token</span>
                <div class="input-with-action">
                  <input v-model="authForm.token" :type="authShowToken ? 'text' : 'password'" placeholder="留空表示不开启鉴权" />
                  <button class="btn-ghost btn-small" @click="authShowToken = !authShowToken">{{ authShowToken ? '隐藏' : '显示' }}</button>
                </div>
              </div>
              <p class="modal-tip">客户端连接时通过 <code>?token=xxx</code> 查询参数、<code>Authorization: Bearer xxx</code> 或 <code>X-Token</code> 请求头携带该令牌。留空则关闭鉴权。</p>
              <div class="hint-codes">
                <div class="hint-code-row">
                  <span class="hint-code-label">WebSocket：</span>
                  <code>ws://服务器IP:{{ commStatus.server_port || '8090' }}/ws?token=你的令牌</code>
                </div>
                <div class="hint-code-row">
                  <span class="hint-code-label">SSE：</span>
                  <code>http://服务器IP:{{ commStatus.server_port || '8090' }}/sse?token=你的令牌</code>
                </div>
                <div class="hint-code-row">
                  <span class="hint-code-label">HTTP：</span>
                  <code>http://服务器IP:{{ commStatus.server_port || '8090' }}/任意路径?token=你的令牌</code>
                </div>
              </div>
            </div>
            <div class="modal-foot">
              <button class="btn-cancel" @click="closeAuthDialog">取消</button>
              <button class="btn-save" :disabled="authSaving" @click="saveAuthConfig">
                <span v-if="authSaving" class="btn-spinner"></span>
                {{ authSaving ? '保存中...' : '保存配置' }}
              </button>
            </div>
          </div>
        </div>
      </Transition>

      <!-- 添加客户端弹窗 -->
      <Transition name="modal">
        <div v-if="addClientDialogVisible" class="modal-backdrop">
          <div class="modal-dialog">
            <div class="modal-head">
              <h3>添加客户端</h3>
              <button class="modal-close" @click="closeAddClientDialog">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
              </button>
            </div>
            <div class="modal-body">
              <div class="modal-field">
                <span class="required">名称</span>
                <input v-model="addClientForm.name" type="text" placeholder="例如：测试服务器" />
              </div>
              <div class="modal-field">
                <span class="required">连接地址</span>
                <input v-model="addClientForm.url" type="text" placeholder="ws://192.168.1.100/ws" />
              </div>
              <div class="modal-field">
                <span class="required">类型</span>
                <div class="type-picker">
                  <button
                    v-for="t in clientTypeOptions"
                    :key="t.value"
                    class="type-option"
                    :class="{ active: addClientForm.type === t.value }"
                    @click="addClientForm.type = t.value"
                  >
                    <span class="pick-dot"></span>{{ t.label }}
                  </button>
                </div>
              </div>
              <div class="modal-field">
                <span>订阅事件（逗号分隔，留空订阅全部）</span>
                <input v-model="addClientForm.events" type="text" placeholder="wallpaper,avatar,nickname,feedback" />
              </div>
              <p class="modal-tip">添加后，后台将尝试连接该地址，并按照配置自动重连。</p>
            </div>
            <div class="modal-foot">
              <button class="btn-cancel" @click="closeAddClientDialog">取消</button>
              <button class="btn-save" :disabled="addClientSaving" @click="doAddClient">
                <span v-if="addClientSaving" class="btn-spinner"></span>
                {{ addClientSaving ? '添加中...' : '确认添加' }}
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, reactive, watch, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { adminApi, showToast } from '@/api/client'
import { webConfirm } from '@/utils/webDialog'
import { useNotificationStore } from '@/stores/notification'
import { fmtDateTime } from '@/utils/time'

const notify = useNotificationStore()
const route = useRoute()

interface NotifyEmail {
  id: number
  email: string
  remark: string
  status: number
  created_at: string
  [key: string]: any
}

interface ModuleItem {
  key: string
  field: string
  label: string
  desc: string
}

// 通知板块定义
const moduleList: ModuleItem[] = [
  { key: 'wallpaper', field: 'notify_wallpaper', label: '壁纸审核', desc: '新壁纸提交待审核' },
  { key: 'avatar', field: 'notify_avatar', label: '头像审核', desc: '新头像提交待审核' },
  { key: 'nickname', field: 'notify_nickname', label: '昵称审核', desc: '新昵称提交待审核' },
  { key: 'feedback', field: 'notify_feedback', label: '反馈更新', desc: '用户提交新反馈' },
]

// ===== 标签页 =====
const activeTab = ref<'settings' | 'email' | 'webhook' | 'commtool'>('settings')

// ===== 状态 =====
const loading = ref(true)
const saving = ref(false)
const list = ref<NotifyEmail[]>([])
const moduleSettings = ref<Record<string, boolean>>({
  wallpaper: true,
  avatar: true,
  nickname: true,
  feedback: true,
})
const importing = ref(false)

const stats = computed(() => {
  const total = list.value.length
  const active = list.value.filter(i => i.status == 1).length
  return { total, active, disabled: total - active }
})

// ===== 加载数据 =====
async function loadList() {
  loading.value = true
  const [listRes, moduleRes] = await Promise.all([
    adminApi<NotifyEmail[]>('list_notification_emails'),
    adminApi<Record<string, boolean>>('get_notification_modules'),
  ])
  if (listRes.code === 200 && Array.isArray(listRes.data)) {
    list.value = listRes.data
  } else {
    list.value = []
  }
  if (moduleRes.code === 200 && moduleRes.data) {
    moduleSettings.value = { ...moduleSettings.value, ...moduleRes.data }
  }
  loading.value = false
}

// ===== 全局板块开关 =====
async function toggleModule(key: string) {
  moduleSettings.value[key] = !moduleSettings.value[key]
  const payload: Record<string, boolean> = {}
  moduleList.forEach(m => { payload[m.key] = moduleSettings.value[m.key] })
  const res = await adminApi('update_notification_modules', payload)
  if (res.code === 200) {
    showToast('已保存', 'success')
  } else {
    moduleSettings.value[key] = !moduleSettings.value[key]
    showToast(res.msg || '保存失败')
  }
}

// ===== 单邮箱板块开关 =====
async function toggleModuleOnEmail(item: NotifyEmail, key: string) {
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

// ===== 导入管理员邮箱 =====
async function doImportAdmin() {
  const ok = await webConfirm('导入后，后台所有已填写邮箱的管理员账号将自动加入通知邮箱列表（已存在的会跳过）。继续吗？', {
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

// ===== 新增 =====
const addModalVisible = ref(false)
interface AddForm {
  email: string
  remark: string
  notify_wallpaper: number
  notify_avatar: number
  notify_nickname: number
  notify_feedback: number
  [key: string]: any
}
const form = ref<AddForm>({
  email: '',
  remark: '',
  notify_wallpaper: 1,
  notify_avatar: 1,
  notify_nickname: 1,
  notify_feedback: 1,
})

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
  if (!email) {
    showToast('请输入邮箱地址')
    return
  }
  if (!isValidEmail(email)) {
    showToast('邮箱格式不正确')
    return
  }
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

// ===== 启用/停用 =====
async function toggle(item: NotifyEmail) {
  const res = await adminApi('toggle_notification_email', { id: item.id })
  if (res.code === 200) {
    item.status = item.status == 1 ? 0 : 1
    showToast('已更新', 'success')
  } else {
    showToast(res.msg || '操作失败')
  }
}

// ===== 删除 =====
async function remove(item: NotifyEmail) {
  const ok = await webConfirm(`确认删除通知邮箱 "${item.email}"？`, { title: '删除通知邮箱', confirmText: '确认删除' })
  if (!ok) return
  const res = await adminApi('delete_notification_email', { id: item.id })
  if (res.code === 200) {
    showToast('删除成功', 'success')
    list.value = list.value.filter(i => i.id !== item.id)
  } else {
    showToast(res.msg || '删除失败')
  }
}

// ===== 发送测试通知 =====
const testingId = ref<number | null>(null)
async function sendTest(item: NotifyEmail) {
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

function toggleWebhookModule(key: string) {
  webhookForm.modules['wh_' + key] = !webhookForm.modules['wh_' + key]
}

// ==================== 通信工具 ====================

interface CommStatus {
  server_running: boolean
  server_port: number
  server_enabled?: boolean
  server_port_config?: number
  ws_server_count: number
  sse_count?: number
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
const authConfigLoaded = ref(false)

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
    authConfigLoaded.value = true
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
    if (res.data.server_enabled != null) {
      commService.enabled = !!res.data.server_enabled
    }
    if (res.data.server_port_config) {
      commService.port = res.data.server_port_config
    }
    if (res.data.ws_client_config) {
      const cfg = res.data.ws_client_config
      wsClientConfig.url = cfg.url || ''
      wsClientConfig.auto_reconnect = !!cfg.auto_reconnect
      wsClientConfig.reconnect_interval = cfg.reconnect_interval || '10'
      wsClientConfig.heartbeat_interval = cfg.heartbeat_interval || '30'
    }
    if (res.data.token_enabled != null) {
      authConfigLoaded.value = true
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
  const ok = await webConfirm('确认清空所有 HTTP 请求日志？', { title: '清空日志', confirmText: '确认清空' })
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
  const ok = await webConfirm('确认断开 WebSocket 客户端连接？', { title: '断开连接', confirmText: '确认断开' })
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
  const ok = await webConfirm('确认清空所有 WebSocket 消息日志？', { title: '清空日志', confirmText: '确认清空' })
  if (!ok) return
  const res = await adminApi('comm_ws_clear')
  if (res.code === 200) {
    wsLogs.value = []
    showToast('已清空', 'success')
  } else {
    showToast(res.msg || '清空失败')
  }
}

// ===== 服务管理 =====
const commService = reactive({
  enabled: false,
  port: 8090,
})
const serviceSaving = ref(false)

async function toggleService() {
  commService.enabled = !commService.enabled
  await saveServiceConfig()
}

async function saveServiceConfig() {
  if (commService.port < 1024 || commService.port > 65535) {
    showToast('端口必须在1024-65535之间')
    return
  }
  serviceSaving.value = true
  const res = await adminApi('comm_service_config', {
    enabled: commService.enabled,
    port: commService.port,
  })
  serviceSaving.value = false
  if (res.code === 200) {
    showToast('服务配置已保存', 'success')
    loadCommStatus()
  } else {
    showToast(res.msg || '保存失败')
  }
}

// ===== 连接鉴权弹窗 =====
const authDialogVisible = ref(false)
function openAuthDialog() {
  loadAuthConfig()
  authDialogVisible.value = true
}
function closeAuthDialog() {
  if (authSaving.value) return
  authDialogVisible.value = false
}

// ===== 添加客户端弹窗 =====
const addClientDialogVisible = ref(false)
const addClientSaving = ref(false)
const clientTypeOptions = [
  { value: 'ws', label: 'WebSocket' },
  { value: 'http', label: 'HTTP' },
  { value: 'sse', label: 'SSE' },
]
const addClientForm = reactive({
  name: '',
  url: '',
  type: 'ws',
  events: '',
})
function openAddClientModal() {
  addClientForm.name = ''
  addClientForm.url = ''
  addClientForm.type = 'ws'
  addClientForm.events = ''
  addClientDialogVisible.value = true
}
function closeAddClientDialog() {
  if (addClientSaving.value) return
  addClientDialogVisible.value = false
}
async function doAddClient() {
  const name = addClientForm.name.trim()
  const url = addClientForm.url.trim()
  if (!name) {
    showToast('请输入名称')
    return
  }
  if (!url) {
    showToast('请输入连接地址')
    return
  }
  addClientSaving.value = true
  const res = await adminApi('comm_client_add', {
    name,
    url,
    type: addClientForm.type,
    events: addClientForm.events.trim(),
  })
  addClientSaving.value = false
  if (res.code === 200) {
    showToast('添加成功', 'success')
    addClientDialogVisible.value = false
    loadCommClients()
    loadCommStatus()
  } else {
    showToast(res.msg || '添加失败')
  }
}

// ===== 已添加客户端列表 =====
interface CommClient {
  id: number
  name: string
  type: string
  url: string
  events: string
  enabled: boolean
  created_at: string
}
const commClients = ref<CommClient[]>([])
async function loadCommClients() {
  const res = await adminApi<CommClient[]>('comm_client_list')
  if (res.code === 200 && Array.isArray(res.data)) {
    commClients.value = res.data
  }
}
async function deleteClient(item: CommClient) {
  const ok = await webConfirm(`确认删除客户端 "${item.name}"？`, { title: '删除客户端', confirmText: '确认删除' })
  if (!ok) return
  const res = await adminApi('comm_client_delete', { id: item.id })
  if (res.code === 200) {
    showToast('已删除', 'success')
    loadCommClients()
  } else {
    showToast(res.msg || '删除失败')
  }
}
async function toggleClientEnabled(item: CommClient) {
  const res = await adminApi('comm_client_toggle', { id: item.id, enabled: !item.enabled })
  if (res.code === 200) {
    item.enabled = !item.enabled
    showToast('已更新', 'success')
  } else {
    showToast(res.msg || '操作失败')
  }
}
async function connectClient(item: CommClient) {
  if (item.type !== 'ws') return
  const res = await adminApi('comm_ws_client_connect', { url: item.url })
  if (res.code === 200) {
    wsClientConnected.value = true
    showToast(res.msg || '连接成功', 'success')
    loadCommStatus()
  } else {
    showToast(res.msg || '连接失败')
  }
}

// ===== 展开卡片 =====
const expandedClient = ref<string | null>(null)
function toggleClient(key: string) {
  expandedClient.value = expandedClient.value === key ? null : key
}

// ===== 合并日志 =====
interface MergedLog {
  source: string
  time: string
  method?: string
  path?: string
  direction?: string
  data?: string
  body?: string
}
const mergedLogs = computed<MergedLog[]>(() => {
  const httpArr: MergedLog[] = httpLogs.value.map(l => ({ ...l, source: 'http' }))
  const wsArr: MergedLog[] = wsLogs.value.map(l => ({ ...l, source: 'ws' }))
  return [...httpArr, ...wsArr].sort((a, b) => (b.time || '').localeCompare(a.time || ''))
})

async function clearAllLogs() {
  const ok = await webConfirm('确认清空所有连接日志？', { title: '清空日志', confirmText: '确认清空' })
  if (!ok) return
  await Promise.all([adminApi('comm_http_clear'), adminApi('comm_ws_clear')])
  httpLogs.value = []
  wsLogs.value = []
  showToast('已清空', 'success')
}

// ===== 手动连接 WS 客户端 =====
async function manualConnectWs() {
  if (!wsClientConfig.url.trim()) {
    showToast('请先填写重连地址')
    return
  }
  const res = await adminApi('comm_ws_client_connect', { url: wsClientConfig.url.trim() })
  if (res.code === 200) {
    wsClientConnected.value = true
    showToast(res.msg || '连接成功', 'success')
    loadCommStatus()
  } else {
    showToast(res.msg || '连接失败')
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
    loadCommClients()
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
.notify-page {
  max-width: 860px;
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
  max-width: 560px;
}
.btn-add {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 10px 18px;
  border-radius: 10px;
  border: none;
  background: var(--accent);
  color: #fff;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}
.btn-add:hover { opacity: 0.85; transform: translateY(-1px); }
.btn-add:active { transform: scale(0.96); }
.btn-add:disabled { opacity: 0.5; cursor: not-allowed; transform: none; }

/* ===== 统计 ===== */
.stats-row {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 12px;
  margin-bottom: 20px;
}
.stat-chip {
  background: var(--card-solid);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 14px 16px;
  display: flex;
  align-items: center;
  gap: 12px;
  transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
  animation: cardIn 0.5s cubic-bezier(0.16, 1, 0.3, 1) backwards;
}
.stat-chip:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(0, 0, 0, 0.06);
}
.stat-icon {
  width: 38px;
  height: 38px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.stat-icon-total { background: #eff6ff; color: #3b82f6; }
.stat-icon-active { background: #f0fdf4; color: #16a34a; }
.stat-icon-disabled { background: rgba(236, 65, 65, 0.12); color: #ef4444; }
.stat-body {
  display: flex;
  flex-direction: column;
  min-width: 0;
}
.stat-label {
  font-size: 11px;
  color: var(--text-muted);
  margin-bottom: 3px;
}
.stat-num {
  font-size: 20px;
  font-weight: 800;
  line-height: 1.2;
  color: var(--text);
}
@keyframes cardIn {
  from { opacity: 0; transform: translateY(16px); }
  to { opacity: 1; transform: translateY(0); }
}

/* ===== 头部按钮组 ===== */
.header-actions {
  display: flex;
  gap: 10px;
  flex-shrink: 0;
}
/* 通知邮箱页顶部操作区 */
.email-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-bottom: 16px;
}
.btn-ghost {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 10px 18px;
  border-radius: 10px;
  border: 1px solid var(--border);
  background: transparent;
  color: var(--text-light);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}
.btn-ghost:hover:not(:disabled) { border-color: var(--accent); color: var(--accent); background: var(--accent-soft); }
.btn-ghost:disabled { opacity: 0.6; cursor: not-allowed; }
.btn-spinner-dark {
  border-color: rgba(0, 0, 0, 0.2);
  border-top-color: var(--accent);
}

/* ===== 通知板块设置 ===== */
.module-block {
  background: var(--card, var(--white));
  border: 1px solid var(--border);
  border-radius: 16px;
  padding: 18px;
  margin-bottom: 20px;
}
.module-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
}
.module-title { display: flex; align-items: center; gap: 12px; }
.module-icon {
  width: 38px; height: 38px;
  border-radius: 10px;
  display: flex; align-items: center; justify-content: center;
  flex-shrink: 0;
  background: #eff6ff; color: #3b82f6;
}
.module-name { font-size: 15px; font-weight: 700; margin: 0; color: var(--text); }
.module-desc { font-size: 12px; color: var(--text-muted); margin: 2px 0 0; max-width: 560px; line-height: 1.5; }
.module-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: 10px;
}
.module-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 12px 14px;
  background: var(--control-bg, #f7f7f8);
  border-radius: 12px;
  transition: all 0.2s;
}
.module-item-info { display: flex; align-items: center; gap: 10px; min-width: 0; }
.module-item-icon {
  width: 32px; height: 32px;
  border-radius: 8px;
  display: flex; align-items: center; justify-content: center;
  flex-shrink: 0;
}
.mi-wallpaper { background: #f0fdf4; color: #16a34a; }
.mi-avatar { background: #eff6ff; color: #3b82f6; }
.mi-nickname { background: rgba(245, 158, 11, 0.14); color: #f59e0b; }
.mi-feedback { background: rgba(236, 65, 65, 0.12); color: #ef4444; }
.module-item-text { display: flex; flex-direction: column; gap: 1px; min-width: 0; }
.module-item-name { font-size: 13px; font-weight: 600; color: var(--text); }
.module-item-desc { font-size: 11px; color: var(--text-muted); }

/* ===== 开关 ===== */
.switch {
  width: 40px; height: 22px;
  border-radius: 12px;
  border: none;
  background: #d1d5db;
  cursor: pointer;
  position: relative;
  flex-shrink: 0;
  transition: background 0.25s cubic-bezier(0.16, 1, 0.3, 1);
  padding: 0;
}
.switch.on { background: var(--accent); }
.switch-knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 18px; height: 18px;
  border-radius: 50%;
  background: #fff;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
  transition: transform 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}
.switch.on .switch-knob { transform: translateX(18px); }

/* ===== 通知设置 ===== */
.settings-card {
  background: var(--card-solid);
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 18px;
}
.settings-card-head {
  margin-bottom: 14px;
}
.notify-perm-row {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}
.notify-perm-ok {
  padding: 8px 12px;
  border-radius: 10px;
  background: rgba(34, 197, 94, 0.08);
}
.notify-hint {
  font-size: 12px;
  color: var(--text-muted);
  line-height: 1.5;
}
.notify-tip {
  margin-top: 10px;
  font-size: 12px;
  color: var(--text-muted);
  line-height: 1.5;
}
.notify-tip.warn {
  padding: 8px 12px;
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
.notify-divider {
  height: 1px;
  margin: 12px 0;
  background: var(--border);
}
.settings-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 6px 0;
}
.settings-row-label {
  display: inline-flex;
  flex-direction: column;
  font-size: 13px;
  font-weight: 600;
  color: var(--text);
}
.settings-row-desc {
  font-size: 11px;
  font-weight: 400;
  color: var(--text-muted);
}
.settings-modules {
  display: flex;
  flex-direction: column;
}

/* ===== 列表 ===== */
.notify-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.notify-card {
  display: flex;
  align-items: flex-start;
  gap: 14px;
  background: var(--card-solid);
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 16px;
  transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
  animation: cardIn 0.45s cubic-bezier(0.16, 1, 0.3, 1) backwards;
}
.notify-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.06);
}
.notify-card.is-disabled { opacity: 0.62; }
.notify-avatar {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.avatar-active { background: #eff6ff; color: #3b82f6; }
.avatar-disabled { background: #f3f4f6; color: #9ca3af; }
.notify-main {
  flex: 1;
  min-width: 0;
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
}
.notify-info { min-width: 0; }
.notify-row {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}
.notify-email {
  font-size: 15px;
  font-weight: 700;
  color: var(--text);
  word-break: break-all;
}
.notify-status {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 11px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 20px;
}
.notify-status .status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
}
.st-active { background: #f0fdf4; color: #16a34a; }
.st-active .status-dot { background: #16a34a; }
.st-disabled { background: #f3f4f6; color: #9ca3af; }
.st-disabled .status-dot { background: #9ca3af; }
.notify-modules {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 8px;
}
.notify-module-tag {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 3px 9px;
  border-radius: 20px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  user-select: none;
  transition: all 0.2s;
}
.notify-module-tag .tag-dot { width: 6px; height: 6px; border-radius: 50%; }
.notify-module-tag.tag-on { background: #f0fdf4; color: #16a34a; }
.notify-module-tag.tag-on .tag-dot { background: #16a34a; }
.notify-module-tag.tag-off { background: #f3f4f6; color: #9ca3af; }
.notify-module-tag.tag-off .tag-dot { background: #9ca3af; }
.notify-module-tag:hover { transform: translateY(-1px); }
.notify-remark {
  margin: 6px 0 2px;
  font-size: 13px;
  color: var(--text-light);
}
.notify-time {
  margin: 0;
  font-size: 11px;
  color: var(--text-muted);
}
.notify-actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}
.btn-action {
  border: 1px solid var(--border);
  border-radius: 8px;
  background: transparent;
  color: var(--text-light);
  padding: 6px 12px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}
.btn-action:hover { border-color: var(--accent); color: var(--accent); background: var(--accent-soft); }
.btn-action.btn-danger:hover { border-color: #ef4444; color: #ef4444; background: rgba(236, 65, 65, 0.12); }

/* ===== 弹窗 ===== */
.modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 1000;
  background: rgba(0, 0, 0, 0.4);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
}
.modal-dialog {
  width: 100%;
  max-width: 420px;
  background: var(--card-solid, var(--white));
  border-radius: 16px;
  box-shadow: 0 24px 60px rgba(0, 0, 0, 0.2);
  overflow: hidden;
}
.modal-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 18px 20px 0;
}
.modal-head h3 { margin: 0; font-size: 16px; font-weight: 800; color: var(--text); }
.modal-close {
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  padding: 4px;
  border-radius: 8px;
  display: flex;
}
.modal-close:hover { background: var(--control-bg); color: var(--text); }
.modal-body {
  padding: 16px 20px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.modal-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.modal-field span {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-light);
}
.modal-field span.required::after { content: '*'; color: #ef4444; margin-left: 2px; }
.modal-field input {
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 10px 12px;
  font-size: 14px;
  font-family: inherit;
  outline: none;
  background: var(--control-bg);
  color: var(--text);
  transition: border-color 0.2s;
  width: 100%;
  box-sizing: border-box;
}
.modal-field input:focus { border-color: var(--accent); }
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
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.2s;
  background: transparent;
}
.modal-module-item input { display: none; }
.modal-module-check {
  width: 18px; height: 18px;
  border-radius: 5px;
  border: 1.5px solid #d1d5db;
  display: flex; align-items: center; justify-content: center;
  color: transparent;
  flex-shrink: 0;
  transition: all 0.2s;
}
.modal-module-item.on .modal-module-check {
  background: var(--accent);
  border-color: var(--accent);
  color: #fff;
}
.modal-module-name { font-size: 12px; font-weight: 600; color: var(--text); }
.modal-tip {
  margin: 0;
  font-size: 12px;
  color: var(--text-muted);
  line-height: 1.5;
}
.modal-foot {
  display: flex;
  gap: 10px;
  padding: 14px 20px 18px;
}
.btn-cancel, .btn-save {
  flex: 1;
  padding: 10px;
  border-radius: 10px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  transition: all 0.2s;
}
.btn-cancel {
  border: 1px solid var(--border);
  background: transparent;
  color: var(--text-muted);
}
.btn-cancel:hover { background: var(--control-bg); }
.btn-save {
  border: none;
  background: var(--accent);
  color: #fff;
}
.btn-save:hover:not(:disabled) { opacity: 0.85; }
.btn-save:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: #fff;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

/* ===== 加载/空状态 ===== */
.state-box {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  color: var(--text-muted);
  gap: 12px;
  font-size: 14px;
}
.state-empty { padding: 80px 20px; }
.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid #e5e5e5;
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}
.empty-icon { color: var(--text-light); opacity: 0.5; }
.empty-title { font-size: 16px; font-weight: 700; color: var(--text-light); margin: 0; }
.empty-sub { font-size: 13px; color: var(--text-muted); margin: 0; }
@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }

/* ===== 过渡动画 ===== */
.fade-down-enter-active, .fade-down-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-down-enter-from { opacity: 0; transform: translateY(-12px); }
.fade-up-enter-active, .fade-up-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-up-enter-from { opacity: 0; transform: translateY(12px); }
.notify-card-enter-active, .notify-card-leave-active { transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1); }
.notify-card-enter-from { opacity: 0; transform: translateY(12px); }
.notify-card-leave-to { opacity: 0; transform: translateY(-8px); }
.modal-enter-active, .modal-leave-active { transition: opacity 0.3s ease; }
.modal-enter-from, .modal-leave-to { opacity: 0; }
.modal-enter-active .modal-dialog { animation: modalIn 0.3s cubic-bezier(0.16, 1, 0.3, 1); }
@keyframes modalIn {
  from { opacity: 0; transform: scale(0.92) translateY(20px); }
  to { opacity: 1; transform: scale(1) translateY(0); }
}

/* ==================== 标签栏 ==================== */
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

/* ==================== Tab 面板 ==================== */
.tab-panel {
  animation: fadeUp 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}
@keyframes fadeUp {
  from { opacity: 0; transform: translateY(8px); }
  to { opacity: 1; transform: translateY(0); }
}
.sub-panel { animation: fadeUp 0.25s cubic-bezier(0.16, 1, 0.3, 1); }

/* ==================== 表单 ==================== */
.form-section {
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.form-section-gap { margin-top: 16px; }
.form-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.form-field-short { max-width: 240px; }
.form-field-grow { flex: 1; min-width: 0; }
.form-field-method { width: 140px; flex-shrink: 0; }
.form-field-half { flex: 1; min-width: 0; }
.form-label {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-light);
}
.form-hint {
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 2px;
}
.form-field input,
.form-field select,
.form-field textarea {
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 10px 12px;
  font-size: 14px;
  font-family: inherit;
  outline: none;
  background: var(--control-bg);
  color: var(--text);
  transition: border-color 0.2s;
  width: 100%;
  box-sizing: border-box;
}
.form-field input:focus,
.form-field select:focus,
.form-field textarea:focus { border-color: var(--accent); }
.form-field input:disabled { opacity: 0.6; cursor: not-allowed; }
.form-field textarea { resize: vertical; min-height: 80px; line-height: 1.5; }
.form-field select { cursor: pointer; }
.form-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 12px 14px;
  background: var(--control-bg, #f7f7f8);
  border-radius: 12px;
}
.form-row-text { display: flex; flex-direction: column; gap: 2px; }
.form-row-inline {
  display: flex;
  gap: 12px;
  align-items: flex-end;
}
.form-actions {
  display: flex;
  gap: 10px;
  justify-content: flex-end;
  flex-wrap: wrap;
}

/* ==================== 通信工具：服务管理 ==================== */
.comm-service-card {
  background: var(--card, var(--white));
  border: 1px solid var(--border);
  border-radius: 16px;
  padding: 18px;
  margin-bottom: 20px;
}
.service-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
.service-title { display: flex; align-items: center; gap: 12px; }
.service-icon {
  width: 38px; height: 38px;
  border-radius: 10px;
  display: flex; align-items: center; justify-content: center;
  flex-shrink: 0;
  background: #eff6ff; color: #3b82f6;
}
.service-title h3 { font-size: 15px; font-weight: 700; margin: 0; color: var(--text); }
.service-title p { font-size: 12px; color: var(--text-muted); margin: 2px 0 0; max-width: 560px; line-height: 1.5; }
.service-config {
  margin-top: 16px;
  padding: 14px;
  background: var(--control-bg, #f7f7f8);
  border-radius: 12px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.service-config-row {
  display: flex;
  align-items: flex-end;
  gap: 12px;
}
.service-config-row .field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  width: 180px;
}
.service-config-row .field span {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-light);
}
.service-config-row .field input {
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 10px 12px;
  font-size: 14px;
  font-family: inherit;
  outline: none;
  background: var(--card-solid);
  color: var(--text);
  transition: border-color 0.2s;
  box-sizing: border-box;
}
.service-config-row .field input:focus { border-color: var(--accent); }
.service-status-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 600;
  white-space: nowrap;
  flex-shrink: 0;
}
.service-status-badge .status-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
}
.service-status-badge.on { background: #f0fdf4; color: #16a34a; }
.service-status-badge.on .status-dot { background: #16a34a; }
.service-status-badge.off { background: #f3f4f6; color: #9ca3af; }
.service-status-badge.off .status-dot { background: #9ca3af; }
.service-actions {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

/* ==================== 通信工具：已添加客户端 ==================== */
.added-clients {
  margin-bottom: 16px;
  border: 1px dashed var(--border);
  border-radius: 12px;
  padding: 12px 14px;
}
.added-clients-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 10px;
}
.added-clients-title {
  font-size: 13px;
  font-weight: 700;
  color: var(--text);
}
.added-clients-count {
  font-size: 11px;
  color: var(--text-muted);
  background: var(--control-bg, #f7f7f8);
  border-radius: 20px;
  padding: 2px 10px;
}
.added-client-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.added-client-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  background: var(--card-solid);
  border: 1px solid var(--border);
  border-radius: 10px;
  transition: all 0.2s;
}
.added-client-item:hover { border-color: var(--accent); }
.added-client-item.disabled { opacity: 0.55; }
.added-client-type {
  flex-shrink: 0;
  font-size: 10px;
  font-weight: 800;
  letter-spacing: 0.04em;
  padding: 3px 8px;
  border-radius: 6px;
}
.added-client-type.t-ws { background: #ecfeff; color: #06b6d4; }
.added-client-type.t-http { background: #eff6ff; color: #3b82f6; }
.added-client-type.t-sse { background: #f5f3ff; color: #8b5cf6; }
.added-client-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 1px;
}
.added-client-info strong {
  font-size: 13px;
  font-weight: 700;
  color: var(--text);
}
.added-client-url {
  font-size: 12px;
  color: var(--text-muted);
  word-break: break-all;
}
.added-client-events {
  font-size: 11px;
  color: var(--accent);
}
.added-clients-empty {
  margin-bottom: 16px;
  padding: 16px;
  text-align: center;
  font-size: 12px;
  color: var(--text-muted);
  background: var(--control-bg, #f7f7f8);
  border-radius: 10px;
}

/* ==================== 通信工具：客户端卡片 ==================== */
.client-grid {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.client-card {
  border: 1px solid var(--border);
  border-radius: 12px;
  background: var(--card-solid);
  overflow: hidden;
  transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}
.client-card:hover { border-color: var(--accent); }
.client-card-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 14px 16px;
  cursor: pointer;
  user-select: none;
}
.client-card-info {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}
.client-card-icon {
  width: 34px; height: 34px;
  border-radius: 9px;
  display: flex; align-items: center; justify-content: center;
  flex-shrink: 0;
}
.client-card-info strong {
  display: block;
  font-size: 14px;
  font-weight: 700;
  color: var(--text);
}
.client-card-info span {
  display: block;
  font-size: 12px;
  color: var(--text-muted);
  margin-top: 1px;
}
.client-expand-icon {
  flex-shrink: 0;
  color: var(--text-muted);
  transition: transform 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}
.client-card.expanded .client-expand-icon { transform: rotate(180deg); }
.client-card-body {
  max-height: 0;
  overflow: hidden;
  transition: max-height 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}
.client-card.expanded .client-card-body {
  max-height: 1200px;
}
.client-card-body .form-section {
  padding: 0 16px 16px;
}

/* ==================== WS 客户端连接行 ==================== */
.ws-client-row {
  margin-top: 4px;
}

/* ==================== 合并日志 ==================== */
.log-merged {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.log-source {
  flex-shrink: 0;
  font-size: 10px;
  font-weight: 800;
  padding: 2px 8px;
  border-radius: 6px;
}
.log-source.http { background: #eff6ff; color: #3b82f6; }
.log-source.ws { background: #ecfeff; color: #06b6d4; }

/* ==================== 类型选择器 ==================== */
.type-picker {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}
.type-option {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  padding: 8px 14px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: transparent;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-light);
  cursor: pointer;
  transition: all 0.2s;
}
.type-option:hover { border-color: var(--accent); color: var(--accent); }
.type-option.active { border-color: var(--accent); background: var(--accent-soft); color: var(--accent); }
.pick-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  border: 2px solid var(--border);
  transition: all 0.2s;
}
.type-option.active .pick-dot { border-color: var(--accent); background: var(--accent); }

/* ==================== 状态徽标 ==================== */
.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 12px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 600;
  white-space: nowrap;
  flex-shrink: 0;
}
.status-badge .status-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
}
.status-badge.on { background: #f0fdf4; color: #16a34a; }
.status-badge.on .status-dot { background: #16a34a; }
.status-badge.off { background: #f3f4f6; color: #9ca3af; }
.status-badge.off .status-dot { background: #9ca3af; }

/* ==================== 信息行 ==================== */
.info-line {
  display: flex;
  align-items: center;
  gap: 16px;
  flex-wrap: wrap;
  font-size: 13px;
  color: var(--text-light);
}
.info-line code {
  background: var(--control-bg);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 2px 8px;
  font-size: 12px;
  font-family: 'Consolas', 'Monaco', monospace;
  color: var(--accent);
  word-break: break-all;
}
.info-hint { font-size: 12px; color: var(--text-muted); }

/* ==================== 日志列表 ==================== */
.log-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 360px;
  overflow: auto;
}
.log-item {
  padding: 10px 12px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--card-solid);
  font-size: 12px;
}
.log-item-head {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-bottom: 4px;
  flex-wrap: wrap;
}
.log-method {
  padding: 2px 8px;
  border-radius: 6px;
  font-weight: 700;
  font-size: 11px;
  flex-shrink: 0;
}
.log-method.GET { background: #eff6ff; color: #3b82f6; }
.log-method.POST { background: #f0fdf4; color: #16a34a; }
.log-method.PUT { background: rgba(245, 158, 11, 0.14); color: #f59e0b; }
.log-method.DELETE { background: rgba(236, 65, 65, 0.12); color: #ef4444; }
.log-method.PATCH { background: #f5f3ff; color: #8b5cf6; }
.log-path {
  font-weight: 600;
  color: var(--text);
  word-break: break-all;
  min-width: 0;
}
.log-time {
  margin-left: auto;
  color: var(--text-muted);
  font-size: 11px;
  flex-shrink: 0;
}
.log-body {
  margin: 0;
  white-space: pre-wrap;
  word-break: break-all;
  color: var(--text-light);
  font-size: 11px;
  line-height: 1.5;
}
.log-direction {
  padding: 2px 8px;
  border-radius: 6px;
  font-size: 11px;
  font-weight: 700;
  flex-shrink: 0;
}
.log-direction.in { background: #eff6ff; color: #3b82f6; }
.log-direction.out { background: #f0fdf4; color: #16a34a; }

/* ==================== 客户端列表 ==================== */
.client-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.client-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--card-solid);
}
.client-info {
  display: flex;
  flex-direction: column;
  min-width: 0;
  flex-shrink: 0;
}
.client-id {
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 12px;
  font-weight: 700;
  color: var(--text);
}
.client-time { font-size: 11px; color: var(--text-muted); }
.client-item input {
  flex: 1;
  min-width: 120px;
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 7px 10px;
  font-size: 13px;
  font-family: inherit;
  outline: none;
  background: var(--control-bg);
  color: var(--text);
  transition: border-color 0.2s;
}
.client-item input:focus { border-color: var(--accent); }

/* ==================== 结果框 ==================== */
.result-box {
  margin-top: 16px;
  border: 1px solid var(--border);
  border-radius: 12px;
  overflow: hidden;
}
.result-head {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  font-size: 13px;
  font-weight: 700;
  background: var(--control-bg);
  border-bottom: 1px solid var(--border);
  flex-wrap: wrap;
}
.result-elapsed {
  margin-left: auto;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-muted);
}
.result-body {
  padding: 12px 14px;
  font-size: 13px;
  max-height: 320px;
  overflow: auto;
}
.result-label {
  margin: 8px 0 4px;
  font-size: 12px;
  font-weight: 700;
  color: var(--text-light);
}
.result-label:first-child { margin-top: 0; }
.result-body pre {
  margin: 0;
  white-space: pre-wrap;
  word-break: break-all;
  font-size: 12px;
  font-family: 'Consolas', 'Monaco', monospace;
  color: var(--text-light);
  background: var(--control-bg);
  border-radius: 8px;
  padding: 10px;
  line-height: 1.5;
}

/* ==================== 空状态（迷你） ==================== */
.empty-mini {
  padding: 24px;
  text-align: center;
  font-size: 13px;
  color: var(--text-muted);
  background: var(--control-bg);
  border-radius: 10px;
}

/* ==================== 小按钮 & 危险按钮 ==================== */
.btn-small {
  padding: 6px 12px;
  font-size: 12px;
  border-radius: 8px;
}
.btn-add.btn-small { padding: 7px 12px; font-size: 12px; border-radius: 8px; }
.btn-ghost.btn-danger:hover { border-color: #ef4444; color: #ef4444; background: rgba(236, 65, 65, 0.12); }

/* ==================== 图标配色 ==================== */
.mi-orange { background: rgba(245, 158, 11, 0.14); color: #f59e0b; }
.mi-blue { background: #eff6ff; color: #3b82f6; }
.mi-green { background: #f0fdf4; color: #16a34a; }
.mi-purple { background: #f5f3ff; color: #8b5cf6; }
.mi-pink { background: #fdf2f8; color: #ec4899; }
.mi-cyan { background: #ecfeff; color: #06b6d4; }
.mi-red { background: rgba(236, 65, 65, 0.12); color: #ef4444; }
.mi-server { background: #eff6ff; color: #3b82f6; }

/* ==================== 鉴权面板 ==================== */
.input-with-action {
  display: flex;
  gap: 8px;
  align-items: center;
}
.input-with-action input {
  flex: 1;
  min-width: 0;
}
.form-hint-text {
  font-size: 12px;
  line-height: 1.6;
  color: var(--text-muted);
  margin-top: 4px;
}
.form-hint-text code {
  background: var(--control-bg);
  border: 1px solid var(--border);
  border-radius: 5px;
  padding: 1px 6px;
  font-size: 11px;
  font-family: 'Consolas', 'Monaco', monospace;
  color: var(--accent);
}
.hint-codes {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.hint-code-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  background: var(--control-bg);
  border-radius: 8px;
  font-size: 13px;
  flex-wrap: wrap;
}
.hint-code-label {
  font-weight: 600;
  color: var(--text);
  flex-shrink: 0;
}
.hint-code-row code {
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 12px;
  color: var(--accent);
  word-break: break-all;
  flex: 1;
  min-width: 0;
}

/* ==================== 客户端订阅事件标签 ==================== */
.client-events {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-top: 4px;
}
.client-event-tag {
  font-size: 11px;
  padding: 1px 8px;
  border-radius: 6px;
  background: #eff6ff;
  color: #3b82f6;
}
.client-event-tag.tag-all {
  background: #f0fdf4;
  color: #16a34a;
}

/* ===== 响应式 ===== */
@media (max-width: 640px) {
  .stats-row { grid-template-columns: 1fr 1fr; gap: 8px; }
  .notify-main { flex-direction: column; }
  .page-header { flex-direction: column; }
  .header-actions { width: 100%; }
  .btn-add, .btn-ghost { flex: 1; justify-content: center; }
  .module-grid { grid-template-columns: 1fr; }
  .modal-modules { grid-template-columns: 1fr; }
  .form-row-inline { flex-direction: column; align-items: stretch; }
  .form-field-method { width: 100%; }
  .form-field-half { width: 100%; }
  .form-actions { justify-content: stretch; }
  .form-actions .btn-add, .form-actions .btn-ghost { flex: 1; }
  .tab-item { padding: 10px 14px; font-size: 13px; }
  .client-item { flex-wrap: wrap; }
  .client-item input { min-width: 100%; }
}
</style>
