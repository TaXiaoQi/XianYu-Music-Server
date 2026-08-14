<template>
  <div class="fb-page">
    <!-- 页面头部 -->
    <Transition name="fade-down" appear>
      <div class="page-header">
        <div class="header-info">
          <h2 class="page-title">
            反馈与建议
            <span v-if="stats.pending > 0" class="pending-badge">{{ stats.pending }} 项待处理</span>
          </h2>
          <p class="page-desc">查看用户提交的反馈与建议，将问题标记为已解决或已拒绝。也可从后台直接创建新事项。</p>
        </div>
        <div class="header-actions">
          <button class="btn-stats" @click="openStats">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 20V10"/><path d="M12 20V4"/><path d="M6 20v-6"/></svg>
            处理统计
          </button>
          <button class="btn-create" @click="openCreateModal">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
            </svg>
            新建事项
          </button>
          <button class="btn-refresh" @click="loadList" :disabled="loading">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" :class="{ spinning: loading }">
              <polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
            </svg>
            刷新
          </button>
        </div>
      </div>
    </Transition>

    <!-- 提交限制配置 -->
    <Transition name="fade-up" appear>
      <div class="limit-panel">
        <div class="limit-info">
          <div class="limit-icon">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
              <path d="M12 8v4"/>
              <path d="M12 16h.01"/>
            </svg>
          </div>
          <div>
            <h3>每日反馈提交上限</h3>
            <p>当前每个用户每天最多可提交 {{ feedbackDailyLimit === 0 ? '不限' : `${feedbackDailyLimit} 条` }}反馈，修改后立即生效。</p>
          </div>
        </div>
        <div class="limit-actions">
          <input
            v-model.number="feedbackLimitInput"
            class="limit-input"
            type="number"
            min="0"
            max="10000"
            step="1"
            :disabled="limitLoading || limitSaving"
            @keyup.enter="saveFeedbackLimit"
          />
          <button class="btn-save-limit" :disabled="limitLoading || limitSaving" @click="saveFeedbackLimit">
            <span v-if="limitSaving" class="btn-spinner dark"></span>
            {{ limitSaving ? '保存中...' : '保存上限' }}
          </button>
          <button class="btn-recycle" @click="openRecycleBin">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/><line x1="10" y1="11" x2="10" y2="17"/><line x1="14" y1="11" x2="14" y2="17"/></svg>
            回收站
          </button>
        </div>
      </div>
    </Transition>

    <!-- 统计卡片 -->
    <Transition name="fade-up" appear>
      <div class="stats-row">
        <div class="stat-chip" :class="{ active: activeFilter === 'all' }" @click="activeFilter = 'all'">
          <div class="stat-icon stat-icon-total"><svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg></div>
          <div class="stat-body"><span class="stat-num">{{ stats.total }}</span><span class="stat-label">全部</span></div>
        </div>
        <div class="stat-chip" :class="{ active: activeFilter === 'pending' }" @click="activeFilter = 'pending'">
          <div class="stat-icon stat-icon-pending"><svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg></div>
          <div class="stat-body"><span class="stat-num">{{ stats.pending }}</span><span class="stat-label">待处理</span></div>
        </div>
        <div class="stat-chip" :class="{ active: activeFilter === 'processing' }" @click="activeFilter = 'processing'">
          <div class="stat-icon stat-icon-processing"><svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2v4"/><path d="M12 18v4"/><path d="M4.93 4.93l2.83 2.83"/><path d="M16.24 16.24l2.83 2.83"/><circle cx="12" cy="12" r="4"/></svg></div>
          <div class="stat-body"><span class="stat-num">{{ stats.processing }}</span><span class="stat-label">处理中</span></div>
        </div>
        <div class="stat-chip" :class="{ active: activeFilter === 'resolved' }" @click="activeFilter = 'resolved'">
          <div class="stat-icon stat-icon-resolved"><svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg></div>
          <div class="stat-body"><span class="stat-num">{{ stats.resolved }}</span><span class="stat-label">已解决</span></div>
        </div>
        <div class="stat-chip" :class="{ active: activeFilter === 'rejected' }" @click="activeFilter = 'rejected'">
          <div class="stat-icon stat-icon-rejected"><svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg></div>
          <div class="stat-body"><span class="stat-num">{{ stats.rejected }}</span><span class="stat-label">已拒绝</span></div>
        </div>
      </div>
    </Transition>

    <!-- 工具条：类型筛选 + 排序 + 批量操作 -->
    <div class="toolbar">
      <div class="toolbar-group">
        <button class="tool-btn" :class="{ active: typeFilter === 'all' }" @click="typeFilter = 'all'">全部类型</button>
        <button class="tool-btn" :class="{ active: typeFilter === 'problem' }" @click="typeFilter = 'problem'">问题反馈</button>
        <button class="tool-btn" :class="{ active: typeFilter === 'suggestion' }" @click="typeFilter = 'suggestion'">功能建议</button>
        <button class="tool-btn" :class="{ active: typeFilter === 'appeal' }" @click="typeFilter = 'appeal'">
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" style="vertical-align: -2px; margin-right: 3px;"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
          封禁申诉
        </button>
      </div>
      <div class="toolbar-right">
        <div class="batch-slot">
          <Transition name="batch-slide">
            <div v-if="batchMode" key="batch" class="batch-bar batch-abs">
              <button class="btn-batch-select" @click="toggleSelectAll">
                <span class="checkbox-badge" :class="{ checked: allSelected }">
                  <svg v-if="allSelected" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
                </span>
                {{ allSelected ? '取消全选' : '全选' }}
              </button>
              <span class="batch-count">已选 {{ selectedIds.size }} 项</span>
              <button class="btn-batch-delete" :disabled="selectedIds.size === 0" @click="confirmBatchDelete">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
                删除所选
              </button>
              <button class="btn-batch-exit" @click="exitBatchMode">退出</button>
            </div>
            <button v-else key="enter" class="btn-batch-enter batch-abs" @click="enterBatchMode">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/></svg>
              批量管理
            </button>
          </Transition>
        </div>
        <button class="sort-btn" @click="openSortMenu">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 5h10"/><path d="M11 9h7"/><path d="M11 13h4"/><path d="M3 17l3 3 3-3"/><path d="M6 18V4"/></svg>
          {{ sortLabel }}
          <svg class="sort-chevron" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"/></svg>
        </button>
      </div>
    </div>

    <!-- 加载中 -->
    <div v-if="loading" class="state-box">
      <div class="spinner"></div>
      <span>加载中...</span>
    </div>

    <template v-else>
      <!-- 空状态 -->
      <Transition name="fade-up" appear v-if="filteredList.length === 0">
        <div class="state-box state-empty">
          <div class="empty-icon">
            <svg width="56" height="56" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>
          </div>
          <p class="empty-title">{{ activeFilter === 'all' ? '暂无反馈记录' : '该状态下暂无反馈' }}</p>
          <p class="empty-sub">用户提交的反馈与建议将显示在这里</p>
        </div>
      </Transition>

      <!-- 反馈卡片列表 -->
      <div v-else class="fb-list">
        <TransitionGroup name="fb-card">
          <div
            v-for="(item, idx) in filteredList"
            :key="item.id"
            class="fb-card"
            :class="[`st-${item.status}`, { 'batch-selected': selectedIds.has(item.id) }]"
            :style="{ animationDelay: `${idx * 60}ms` }"
          >
            <!-- 批量选择复选框 -->
            <div v-if="batchMode" class="card-checkbox" @click.stop="toggleSelect(item.id)">
              <input type="checkbox" :checked="selectedIds.has(item.id)" />
            </div>
            <!-- 卡片头部 -->
            <div class="card-top">
              <div class="card-user">
                <img
                  v-if="item.avatar_url"
                  :src="item.avatar_url"
                  alt="头像"
                  class="user-avatar avatar-img"
                  @click.stop="openImageViewer([item.avatar_url], 0)"
                />
                <div v-else class="user-avatar" :class="item.category === 'appeal' ? 'avatar-appeal' : (item.feedback_type === 'suggestion' ? 'avatar-suggestion' : 'avatar-problem')">
                  <svg v-if="item.category === 'appeal'" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/><path d="M12 8v4"/><path d="M12 16h.01"/></svg>
                  <svg v-else-if="item.feedback_type === 'suggestion'" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/></svg>
                  <svg v-else width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
                </div>
                <div class="user-info">
                  <span class="user-name">{{ item.nickname || '匿名用户' }}</span>
                  <span class="user-id">{{ item.ciyuanxi_id || '后台创建' }}</span>
                </div>
              </div>
              <div class="card-top-right">
                <span v-if="item.category === 'appeal'" class="type-badge type-appeal">封禁申诉</span>
                <span v-else-if="item.feedback_type" class="type-badge" :class="item.feedback_type === 'suggestion' ? 'type-suggestion' : 'type-problem'">
                  {{ item.feedback_type === 'suggestion' ? '功能建议' : '问题反馈' }}
                </span>
                <span class="status-badge" :class="`badge-${item.status}`">
                  <svg v-if="item.status === 'resolved'" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><polyline points="20 6 9 17 4 12"/></svg>
                  <svg v-else-if="item.status === 'rejected'" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
                  <svg v-else-if="item.status === 'processing'" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
                  {{ statusLabel(item.status) }}
                </span>
              </div>
            </div>

            <!-- 主体：左内容，右图片（默认展开） -->
            <div class="card-body">
              <div class="card-main">
                <p class="fb-content fb-content-main">{{ item.content || '无内容' }}</p>
                <div class="detail-more">
                  <div v-if="hasErrorLogs(item) || hasAllLogs(item)" class="log-summary">
                    <span v-if="hasErrorLogs(item)" class="log-chip">错误日志 {{ formatLogSize(item.error_logs_chars) }}</span>
                    <span v-if="hasAllLogs(item)" class="log-chip">全量日志 {{ formatLogSize(item.all_logs_chars) }}</span>
                  </div>
                  <div v-if="item.assignee" class="assignee-row">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
                    <span>{{ item.status === 'rejected' ? '拒绝人' : '认领人' }}：{{ item.assignee }}</span>
                  </div>
                  <div v-if="item.status === 'resolved' && item.resolve_note" class="resolve-note">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
                    <div class="resolve-text"><span class="resolve-label">完成说明</span><span>{{ item.resolve_note }}</span></div>
                  </div>
                </div>
              </div>
              <!-- 图片缩略图（堆叠，仅显示第一张，点击查看详情） -->
              <div v-if="itemImages(item).length > 0" class="img-stack" @click.stop="openImageViewer(itemImages(item), 0)">
                <img
                  v-for="(img, i) in itemImages(item)"
                  :key="i"
                  :src="img"
                  class="stack-thumb"
                  :style="stackThumbStyle(i, itemImages(item).length)"
                  alt="反馈图片"
                />
                <span v-if="itemImages(item).length > 1" class="stack-count">{{ itemImages(item).length }}</span>
              </div>
            </div>

            <!-- 卡片底部 -->
            <div class="card-foot" @click.stop>
              <div class="foot-meta">
                <span class="meta-time">发布时间：{{ fmtTime(item.created_at) }}</span>
                <span v-if="item.claimed_at" class="meta-time"> | 认领时间：{{ fmtTime(item.claimed_at) }}</span>
                <span v-if="item.status === 'resolved' && item.resolved_at" class="meta-time"> | 完成时间：{{ fmtTime(item.resolved_at) }}</span>
              </div>
              <div class="foot-actions">
                <button v-if="hasErrorLogs(item) || hasAllLogs(item)" class="act-btn act-log" @click="openLogModal(item)">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="8" y1="13" x2="16" y2="13"/><line x1="8" y1="17" x2="16" y2="17"/></svg>
                  日志
                </button>
                <button v-if="item.status === 'pending' || (item.status === 'processing' && isMineFeedback(item))" class="act-btn act-claim" @click="claimFeedback(item.id)">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
                  认领
                </button>
                <button v-if="item.status === 'processing' && isMineFeedback(item)" class="act-btn act-resolve" @click="openResolveModal(item)">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12"/></svg>
                  完成
                </button>
                <button v-if="(item.status === 'pending' || item.status === 'processing') && (item.status === 'pending' || isMineFeedback(item))" class="act-btn act-reject" @click="changeStatus(item.id, 'rejected')">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
                  拒绝
                </button>
              </div>
            </div>
          </div>
        </TransitionGroup>
      </div>
    </template>

    <!-- 完成说明弹窗 -->
    <Transition name="modal">
      <div v-if="resolveModalVisible" class="modal-backdrop">
        <div class="modal-dialog resolve-dialog">
          <div class="modal-head">
            <h3>完成反馈</h3>
            <button class="modal-close" @click="closeResolveModal">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
          <div class="modal-body">
            <div v-if="resolveTarget" class="resolve-target-info">
              <strong>{{ resolveTarget.title || '无标题' }}</strong>
              <span>{{ resolveTarget.nickname || '匿名用户' }} · {{ resolveTarget.ciyuanxi_id || '-' }}</span>
            </div>
            <label class="resolve-field">
              <span class="resolve-field-label">完成说明 <em>*</em></span>
              <textarea
                v-model="resolveNote"
                class="resolve-textarea"
                rows="5"
                maxlength="1000"
                placeholder="请填写本次处理的完成说明（必填），该说明将展示给提交反馈的用户"
              ></textarea>
              <span class="resolve-count">{{ resolveNote.length }}/1000</span>
            </label>
          </div>
          <div class="modal-foot">
            <button class="btn-cancel" :disabled="resolveSaving" @click="closeResolveModal">取消</button>
            <button class="btn-confirm" :disabled="resolveSaving || !resolveNote.trim()" @click="confirmResolve">
              {{ resolveSaving ? '提交中...' : '确认完成' }}
            </button>
          </div>
        </div>
      </div>
    </Transition>

    <!-- 日志弹窗 -->
    <Transition name="modal">
      <div v-if="logModalVisible" class="modal-backdrop">
        <div class="modal-dialog log-dialog">
          <div class="modal-head">
            <h3>反馈日志</h3>
            <button class="modal-close" @click="closeLogModal">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
          <div class="modal-body">
            <div v-if="logTarget" class="log-target-info">
              <strong>{{ logTarget.title || '无标题' }}</strong>
              <span>{{ logTarget.nickname || '匿名用户' }} · {{ logTarget.ciyuanxi_id || '-' }}</span>
            </div>
            <div class="log-tabs">
              <button
                class="log-tab"
                :class="{ active: activeLogTab === 'error' }"
                :disabled="!logTarget?.error_logs"
                @click="activeLogTab = 'error'"
              >
                错误日志
              </button>
              <button
                class="log-tab"
                :class="{ active: activeLogTab === 'all' }"
                :disabled="!logTarget?.all_logs"
                @click="activeLogTab = 'all'"
              >
                全量日志
              </button>
            </div>
            <div v-if="logLoading" class="state-box compact">
              <div class="spinner"></div>
              <span>正在加载日志...</span>
            </div>
            <pre v-else class="log-content">{{ currentLogText || '暂无日志内容' }}</pre>
          </div>
          <div class="modal-foot">
            <button class="btn-cancel" @click="closeLogModal">关闭</button>
          </div>
        </div>
      </div>
    </Transition>

    <!-- 新建事项弹窗 -->
    <Transition name="modal">
      <div v-if="createModalVisible" class="modal-backdrop">
        <div class="modal-dialog create-dialog">
          <div class="modal-head">
            <h3>新建事项</h3>
            <button class="modal-close" @click="closeCreateModal">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
          <div class="modal-body">
            <div class="create-type-row">
              <button class="create-type-btn" :class="{ active: createType === 'problem' }" @click="createType = 'problem'">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
                问题反馈
              </button>
              <button class="create-type-btn" :class="{ active: createType === 'suggestion' }" @click="createType = 'suggestion'">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/></svg>
                功能建议
              </button>
            </div>
            <label class="create-field">
              <span class="create-field-label">内容 <em>*</em></span>
              <textarea v-model="createContent" class="create-textarea" rows="4" maxlength="1000" placeholder="请输入内容描述（最多 1000 字）"></textarea>
              <span class="create-count">{{ createContent.length }}/1000</span>
            </label>
            <div class="create-field">
              <span class="create-field-label">图片 <span class="optional">（可选，最多 6 张）</span></span>
              <div
                class="create-dropzone"
                :class="{ dragging: createDragging, has: createImages.length > 0 }"
                @dragover.prevent="onCreateDragOver"
                @dragleave.prevent="onCreateDragLeave"
                @drop.prevent="onCreateDrop"
                @click="createFileInput?.click()"
              >
                <input ref="createFileInput" type="file" accept="image/*" multiple hidden @change="onCreateFileChange" />
                <div class="dropzone-icon">
                  <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>
                </div>
                <div class="dropzone-text">
                  <strong>点击或拖拽图片到此处</strong>
                  <span>支持 JPG / PNG / GIF，单张不超过 8MB</span>
                </div>
              </div>
              <div v-if="createImages.length > 0" class="create-preview">
                <div v-for="(img, i) in createImages" :key="i" class="preview-item">
                  <img :src="img" class="preview-img" @click.stop="openImageViewer(createImages, i)" />
                  <button class="preview-remove" @click.stop="removeCreateImage(i)">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
                  </button>
                </div>
              </div>
            </div>
            <label class="create-notify">
              <input v-model="createNotifyExternal" type="checkbox" class="notify-check" />
              <span class="notify-box">
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12"/></svg>
              </span>
              <span class="notify-text">
                <strong>外部同步通知</strong>
                <span>发布后主动向「外部通知」配置中启用的邮箱发送邮件提醒</span>
              </span>
            </label>
          </div>
          <div class="modal-foot">
            <button class="btn-cancel" :disabled="createSaving" @click="closeCreateModal">取消</button>
            <button class="btn-confirm btn-create-submit" :disabled="createSaving || !createContent.trim()" @click="submitCreate">
              {{ createSaving ? '发布中...' : '发布' }}
            </button>
          </div>
        </div>
      </div>
    </Transition>

    <!-- 处理统计弹窗 -->
    <Transition name="modal">
      <div v-if="statsModalVisible" class="modal-backdrop">
        <div class="modal-dialog stats-dialog">
          <div class="modal-head">
            <h3>管理员处理统计</h3>
            <button class="modal-close" @click="closeStats">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
          <div class="modal-body">
            <div v-if="statsLoading" class="state-box compact">
              <div class="spinner"></div>
              <span>正在加载统计...</span>
            </div>
            <template v-else>
              <div class="stats-total">
                <span class="stats-total-num">{{ statsGrandTotal }}</span>
                <span class="stats-total-label">累计处理反馈总量</span>
              </div>
              <div v-if="statsList.length === 0" class="state-box compact">
                <span>暂无统计数据</span>
              </div>
              <div v-else class="stats-table-wrap">
                <table class="stats-table">
                  <thead>
                    <tr>
                      <th>管理账号</th>
                      <th>总量</th>
                      <th>处理中</th>
                      <th>已解决</th>
                      <th>已拒绝</th>
                      <th>待处理</th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr v-for="(row, i) in statsList" :key="i">
                      <td class="stats-admin">
                        <span class="admin-dot" :class="row.admin_name === '未认领' ? 'dot-unclaimed' : `dot-${i % 4}`"></span>
                        {{ row.admin_name }}
                      </td>
                      <td class="stats-num-cell"><strong>{{ row.total }}</strong></td>
                      <td class="stats-num-cell processing">{{ row.processing }}</td>
                      <td class="stats-num-cell resolved">{{ row.resolved }}</td>
                      <td class="stats-num-cell rejected">{{ row.rejected }}</td>
                      <td class="stats-num-cell pending">{{ row.pending }}</td>
                    </tr>
                  </tbody>
                </table>
              </div>
            </template>
          </div>
          <div class="modal-foot">
            <button class="btn-cancel" @click="closeStats">关闭</button>
          </div>
        </div>
      </div>
    </Transition>

    <!-- 回收站弹窗 -->
    <Transition name="modal">
      <div v-if="recycleModalVisible" class="modal-backdrop">
        <div class="modal-dialog recycle-dialog">
          <div class="modal-head">
            <h3>回收站</h3>
            <button class="modal-close" @click="closeRecycleBin">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
          <div class="modal-body">
            <div class="recycle-tip">
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M12 8v4"/><path d="M12 16h.01"/></svg>
              <span>已删除的记录将保留 14 天，超期自动永久清除。可点击「恢复」将记录还原至反馈列表。</span>
            </div>
            <div v-if="recycleLoading" class="state-box compact">
              <div class="spinner"></div>
              <span>正在加载...</span>
            </div>
            <div v-else-if="recycleList.length === 0" class="state-box compact">
              <span>回收站为空</span>
            </div>
            <div v-else class="recycle-list">
              <div v-for="item in recycleList" :key="item.id" class="recycle-item">
                <div class="recycle-item-main">
                  <div class="recycle-item-title">{{ item.title || '无标题' }}</div>
                  <div class="recycle-item-meta">
                    <span v-if="item.category === 'appeal'" class="type-badge type-appeal small">封禁申诉</span>
                    <span v-else-if="item.feedback_type === 'suggestion'" class="type-badge type-suggestion small">功能建议</span>
                    <span v-else class="type-badge type-problem small">问题反馈</span>
                    <span class="recycle-user">{{ item.nickname || '匿名' }}</span>
                    <span class="recycle-del">删除人：{{ item.deleted_by || '-' }}</span>
                    <span class="recycle-remaining" :class="{ urgent: item.remaining_hours < 48 }">
                      剩余 {{ Math.floor((item.remaining_hours || 0) / 24) }} 天 {{ (item.remaining_hours || 0) % 24 }} 小时
                    </span>
                  </div>
                </div>
                <button class="btn-restore" @click="restoreItem(item.id)">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/></svg>
                  恢复
                </button>
              </div>
            </div>
          </div>
          <div class="modal-foot">
            <button class="btn-cancel" @click="closeRecycleBin">关闭</button>
          </div>
        </div>
      </div>
    </Transition>

    <!-- 图片查看器 -->
    <Transition name="modal">
      <div v-if="imageViewerVisible" class="viewer-backdrop">
        <button class="viewer-close" @click="closeImageViewer">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
        </button>
        <button v-if="imageViewerList.length > 1" class="viewer-nav viewer-prev" @click="prevImage">
          <svg width="26" height="26" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="15 18 9 12 15 6"/></svg>
        </button>
        <img
          v-if="imageViewerList[imageViewerIndex]"
          :src="imageViewerList[imageViewerIndex]"
          class="viewer-img"
          :class="{ ready: imageViewerReady }"
          @load="imageViewerReady = true"
        />
        <button v-if="imageViewerList.length > 1" class="viewer-nav viewer-next" @click="nextImage">
          <svg width="26" height="26" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="9 18 15 12 9 6"/></svg>
        </button>
        <div v-if="imageViewerList.length > 1" class="viewer-counter">{{ imageViewerIndex + 1 }} / {{ imageViewerList.length }}</div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { adminApi, showToast, getAdminUser } from '@/api/client'
import { webConfirm } from '@/utils/webDialog'
import { webActionMenu } from '@/utils/webDialog'

// 当前登录管理员用户名（用于判断反馈是否由本人认领）
const currentAdminName = getAdminUser()?.username || ''
function isMineFeedback(item: Feedback): boolean {
  return !!item.assignee && item.assignee === currentAdminName
}

interface Feedback {
  id: number
  ciyuanxi_id: string
  nickname: string
  title: string
  content: string
  status: string
  admin_reply: string | null
  error_logs?: string | null
  all_logs?: string | null
  log_meta?: string | null
  error_logs_chars?: number
  all_logs_chars?: number
  has_error_logs?: number | string | boolean
  has_all_logs?: number | string | boolean
  replied_at: string | null
  replied_by: string
  assignee: string
  resolve_note: string | null
  ip: string
  created_at: string
  updated_at: string
  [key: string]: any
}

interface FbStats {
  total: number
  pending: number
  processing: number
  resolved: number
  rejected: number
}

interface FeedbackLimit {
  feedback_daily_limit: number
}

// ===== 状态 =====
const loading = ref(true)
const feedbackList = ref<Feedback[]>([])
const activeFilter = ref('all')
const stats = ref<FbStats>({ total: 0, pending: 0, processing: 0, resolved: 0, rejected: 0 })
const limitLoading = ref(false)
const limitSaving = ref(false)
const feedbackDailyLimit = ref(20)
const feedbackLimitInput = ref(20)

const statusMap: Record<string, string> = {
  pending: '待处理',
  processing: '处理中',
  resolved: '已解决',
  rejected: '已拒绝',
}

function statusLabel(s: string): string {
  return statusMap[s] || s
}

// ===== 类型筛选 + 排序 =====
const typeFilter = ref('all')
const sortMode = ref('post_time_desc')
const sortOptions = [
  { key: 'post_time_desc', label: '最新提交' },
  { key: 'post_time_asc', label: '最早提交' },
  { key: 'update_desc', label: '最近更新' },
]
const sortLabel = computed(() => sortOptions.find(o => o.key === sortMode.value)?.label || '排序')
async function openSortMenu() {
  const key = await webActionMenu('排序方式', sortOptions.map(o => ({ key: o.key, label: o.label })))
  if (key && key !== sortMode.value) {
    sortMode.value = key
    loadList()
  }
}

// ===== 时间戳格式化 =====
// 按数据库字面时间显示（兼容服务器/客户端），如 2026年12月12日 21时25分
function fmtTime(v: any): string {
  if (!v) return ''
  const m = String(v).match(/(\d{4})[-/](\d{1,2})[-/](\d{1,2})[T ]+(\d{1,2}):(\d{1,2})/)
  if (!m) return String(v)
  const [, y, mo, d, h, mi] = m.map(Number)
  return `${y}年${mo}月${d}日 ${h}时${String(mi).padStart(2, '0')}分`
}

// ===== 图片处理 =====
// 客户端上传的图片可能存为内网 IP:端口 的完整 URL（如 http://47.80.58.50:8081/...），
// 后台通过 https 域名访问时会被浏览器混合内容策略拦截，这里将跨源地址改写为后台同源路径加载。
function normalizeImgUrl(u: string): string {
  if (u.startsWith('http://') || u.startsWith('https://')) {
    try {
      const parsed = new URL(u, window.location.origin)
      if (parsed.origin !== window.location.origin) {
        return window.location.origin + parsed.pathname + parsed.search
      }
      return u
    } catch {
      return u
    }
  }
  return u
}
function itemImages(item: Feedback): string[] {
  if (!item.images) return []
  try {
    const arr = JSON.parse(item.images)
    return Array.isArray(arr)
      ? arr.filter((u: string) => typeof u === 'string' && (u.startsWith('http') || u.startsWith('/'))).map(normalizeImgUrl)
      : []
  } catch {
    return []
  }
}
// 堆叠样式：仅第一张完整显示，其余向右下偏移并置于底层，视觉上"只显示一张，其余叠压其后"
function stackThumbStyle(i: number, total: number): Record<string, string> {
  if (total <= 1) return {}
  const offset = Math.min(i, 3) * 5
  return {
    left: `${offset}px`,
    top: `${offset}px`,
    zIndex: String(total - i),
  }
}

// ===== 图片查看器 =====
const imageViewerVisible = ref(false)
const imageViewerList = ref<string[]>([])
const imageViewerIndex = ref(0)
const imageViewerReady = ref(false)

function openImageViewer(imgs: string[], index: number) {
  imageViewerList.value = imgs
  imageViewerIndex.value = index
  imageViewerReady.value = false
  imageViewerVisible.value = true
}
function closeImageViewer() {
  imageViewerVisible.value = false
  imageViewerList.value = []
}
function nextImage() {
  if (imageViewerList.value.length === 0) return
  imageViewerIndex.value = (imageViewerIndex.value + 1) % imageViewerList.value.length
  imageViewerReady.value = false
}
function prevImage() {
  if (imageViewerList.value.length === 0) return
  imageViewerIndex.value = (imageViewerIndex.value - 1 + imageViewerList.value.length) % imageViewerList.value.length
  imageViewerReady.value = false
}
function onViewerKeydown(e: KeyboardEvent) {
  if (!imageViewerVisible.value) return
  if (e.key === 'Escape') closeImageViewer()
  else if (e.key === 'ArrowRight') nextImage()
  else if (e.key === 'ArrowLeft') prevImage()
}

// ===== 处理统计弹窗 =====
const statsModalVisible = ref(false)
const statsLoading = ref(false)
const statsList = ref<Array<{ admin_name: string; total: number; processing: number; resolved: number; rejected: number; pending: number }>>([])
const statsGrandTotal = ref(0)

async function openStats() {
  statsModalVisible.value = true
  await loadStats()
}
async function loadStats() {
  statsLoading.value = true
  const res = await adminApi<{ list: Array<{ admin_name: string; total: number; processing: number; resolved: number; rejected: number; pending: number }>; grand_total: number }>('feedback_admin_stats')
  statsLoading.value = false
  if (res.code === 200 && res.data) {
    statsList.value = res.data.list || []
    statsGrandTotal.value = Number(res.data.grand_total ?? 0)
  } else {
    statsList.value = []
    statsGrandTotal.value = 0
    showToast(res.msg || '统计加载失败')
  }
}
function closeStats() {
  if (statsLoading.value) return
  statsModalVisible.value = false
}

// ===== 新建事项弹窗 =====
const createModalVisible = ref(false)
const createType = ref<'problem' | 'suggestion'>('problem')
// 标题默认取所选类型，无需手动填写
const createTitle = computed(() => {
  if (createType.value === 'suggestion') return '功能建议'
  return '问题反馈'
})
const createContent = ref('')
const createImages = ref<string[]>([])
const createDragging = ref(false)
const createNotifyExternal = ref(false)
const createSaving = ref(false)
const createFileInput = ref<HTMLInputElement | null>(null)

function openCreateModal() {
  createType.value = 'problem'
  createContent.value = ''
  createImages.value = []
  createNotifyExternal.value = false
  createSaving.value = false
  createDragging.value = false
  createModalVisible.value = true
}
function closeCreateModal() {
  if (createSaving.value) return
  createModalVisible.value = false
}
function onCreateDragOver(e: DragEvent) {
  e.preventDefault()
  createDragging.value = true
}
function onCreateDragLeave() {
  createDragging.value = false
}
function onCreateDrop(e: DragEvent) {
  e.preventDefault()
  createDragging.value = false
  const files = e.dataTransfer?.files
  if (files && files.length > 0) {
    handleCreateFiles(Array.from(files))
  }
}
function onCreateFileChange(e: Event) {
  const input = e.target as HTMLInputElement
  if (!input.files) return
  handleCreateFiles(Array.from(input.files))
  input.value = ''
}
function handleCreateFiles(files: File[]) {
  const remaining = 6 - createImages.value.length
  if (remaining <= 0) {
    showToast('最多上传 6 张图片')
    return
  }
  const accepted = files.slice(0, remaining)
  for (const file of accepted) {
    if (!file.type.startsWith('image/')) continue
    if (file.size > 8 * 1024 * 1024) {
      showToast(`图片 ${file.name} 超过 8MB，已跳过`)
      continue
    }
    const reader = new FileReader()
    reader.onload = () => {
      createImages.value.push(reader.result as string)
    }
    reader.onerror = () => showToast(`图片 ${file.name} 读取失败`)
    reader.readAsDataURL(file)
  }
}
function removeCreateImage(index: number) {
  createImages.value.splice(index, 1)
}
async function submitCreate() {
  if (createSaving.value) return
  if (!createContent.value.trim()) {
    showToast('请填写内容')
    return
  }
  createSaving.value = true
  const res = await adminApi('create_feedback', {
    feedback_type: createType.value,
    title: createTitle.value.trim(),
    content: createContent.value.trim(),
    images: createImages.value,
    notify_external: createNotifyExternal.value ? 1 : 0,
  })
  createSaving.value = false
  if (res.code === 200) {
    showToast('创建成功', 'success')
    closeCreateModal()
    await loadList()
  } else {
    showToast(res.msg || '创建失败')
  }
}

// 仅待处理/处理中可执行完成或拒绝操作，终态（已解决/已拒绝）不再显示操作按钮

// ===== 认领功能 =====
async function claimFeedback(id: number) {
  const item = feedbackList.value.find(f => f.id === id)
  const isTransfer = item?.status === 'processing'
  const ok = await webConfirm(isTransfer ? '确认将该反馈转认领到自己名下？认领后问题将转移到您的名下。' : '确认认领该反馈？认领后将自动划入您的名下并移入处理中。', {
    title: '认领反馈',
    confirmText: '认领',
  })
  if (!ok) return
  const res = await adminApi('claim_feedback', { id })
  if (res.code === 200) {
    showToast(isTransfer ? '已转认领到自己名下' : '认领成功，已置为处理中', 'success')
    // 本地更新，不刷新页面
    const item2 = feedbackList.value.find(f => f.id === id)
    if (item2) {
      const oldStatus = item2.status
      item2.status = 'processing'
      item2.assignee = res.data?.assignee || ''
      if (oldStatus !== 'processing') {
        if (stats.value[oldStatus as keyof FbStats] !== undefined) {
          stats.value[oldStatus as keyof FbStats]--
        }
        if (stats.value.processing !== undefined) {
          stats.value.processing++
        }
      }
    }
  } else {
    showToast(res.msg || '认领失败')
  }
}

// ===== 完成弹窗 =====
const resolveModalVisible = ref(false)
const resolveTarget = ref<Feedback | null>(null)
const resolveNote = ref('')
const resolveSaving = ref(false)

function openResolveModal(item: Feedback) {
  resolveTarget.value = item
  resolveNote.value = ''
  resolveSaving.value = false
  resolveModalVisible.value = true
}

function closeResolveModal() {
  if (resolveSaving.value) return
  resolveModalVisible.value = false
  resolveTarget.value = null
  resolveNote.value = ''
}

async function confirmResolve() {
  if (!resolveTarget.value || !resolveNote.value.trim()) return
  resolveSaving.value = true
  const res = await adminApi('resolve_feedback', {
    id: resolveTarget.value.id,
    note: resolveNote.value.trim(),
  })
  resolveSaving.value = false
  if (res.code === 200) {
    showToast('已标记为已完成', 'success')
    closeResolveModal()
    await loadList()
  } else {
    showToast(res.msg || '操作失败')
  }
}

function formatLogSize(chars?: number | string): string {
  const n = Number(chars || 0)
  if (n <= 0) return ''
  if (n < 1024) return `${n} 字`
  return `${(n / 1024).toFixed(1)}K 字`
}

function truthyFlag(value: unknown): boolean {
  return value === true || value === 1 || value === '1'
}

function hasErrorLogs(item: Feedback): boolean {
  return truthyFlag(item.has_error_logs) || !!item.error_logs
}

function hasAllLogs(item: Feedback): boolean {
  return truthyFlag(item.has_all_logs) || !!item.all_logs
}

const filteredList = computed(() => {
  let arr = feedbackList.value
  if (activeFilter.value !== 'all') {
    arr = arr.filter(f => f.status === activeFilter.value)
  }
  if (typeFilter.value === 'appeal') {
    arr = arr.filter(f => f.category === 'appeal')
  } else if (typeFilter.value !== 'all') {
    arr = arr.filter(f => f.feedback_type === typeFilter.value && f.category !== 'appeal')
  }
  return arr
})

// ===== 加载数据 =====
async function loadList() {
  loading.value = true
  const res = await adminApi<{ list: Feedback[]; stats: FbStats }>('list_feedback', {
    status_filter: activeFilter.value === 'all' ? '' : activeFilter.value,
    sort: sortMode.value,
  })
  if (res.code === 200 && res.data) {
    feedbackList.value = res.data.list || []
    if (res.data.stats) {
      stats.value = res.data.stats
    }
  } else {
    feedbackList.value = []
  }
  loading.value = false
}

async function loadFeedbackLimit() {
  limitLoading.value = true
  const res = await adminApi<FeedbackLimit>('get_feedback_limit')
  if (res.code === 200 && res.data) {
    const limit = Number(res.data.feedback_daily_limit ?? 20)
    feedbackDailyLimit.value = Number.isFinite(limit) ? limit : 20
    feedbackLimitInput.value = feedbackDailyLimit.value
  } else {
    showToast(res.msg || '反馈上限加载失败')
  }
  limitLoading.value = false
}

async function saveFeedbackLimit() {
  const limit = Number(feedbackLimitInput.value)
  if (!Number.isInteger(limit) || limit < 0 || limit > 10000) {
    showToast('每日上限需为 0 到 10000 的整数')
    return
  }
  limitSaving.value = true
  const res = await adminApi<FeedbackLimit>('update_feedback_limit', {
    feedback_daily_limit: limit,
  })
  limitSaving.value = false
  if (res.code === 200) {
    feedbackDailyLimit.value = Number(res.data?.feedback_daily_limit ?? limit)
    feedbackLimitInput.value = feedbackDailyLimit.value
    showToast('反馈提交上限已保存', 'success')
  } else {
    showToast(res.msg || '保存失败')
  }
}

// ===== 状态变更 =====
async function changeStatus(id: number, status: string) {
  const tips: Record<string, string> = {
    resolved: '确认将此反馈标记为已解决？',
    rejected: '确认拒绝此反馈？',
  }
  if (tips[status]) {
    const ok = await webConfirm(tips[status], { title: '更新反馈状态', confirmText: '确认' })
    if (!ok) return
  }
  const res = await adminApi('update_feedback_status', { id, status })
  if (res.code === 200) {
    showToast('状态已更新', 'success')
    // 本地更新
    const item = feedbackList.value.find(f => f.id === id)
    if (item) {
      const oldStatus = item.status
      item.status = status
      // 更新统计
      if (stats.value[oldStatus as keyof FbStats] !== undefined) {
        stats.value[oldStatus as keyof FbStats]--
      }
      if (stats.value[status as keyof FbStats] !== undefined) {
        stats.value[status as keyof FbStats]++
      }
    }
  } else {
    showToast(res.msg || '操作失败')
  }
}

// ===== 日志弹窗 =====
const logModalVisible = ref(false)
const logTarget = ref<Feedback | null>(null)
const logLoading = ref(false)
const activeLogTab = ref<'error' | 'all'>('error')

const currentLogText = computed(() => {
  if (!logTarget.value) return ''
  return activeLogTab.value === 'error'
    ? (logTarget.value.error_logs || '')
    : (logTarget.value.all_logs || '')
})

async function openLogModal(item: Feedback) {
  logModalVisible.value = true
  logTarget.value = item
  activeLogTab.value = hasErrorLogs(item) ? 'error' : 'all'
  logLoading.value = true
  const res = await adminApi<Feedback>('get_feedback_detail', { id: item.id })
  logLoading.value = false
  if (res.code === 200 && res.data) {
    logTarget.value = res.data
    activeLogTab.value = res.data.error_logs ? 'error' : 'all'
  } else {
    showToast(res.msg || '日志加载失败')
  }
}

function closeLogModal() {
  if (logLoading.value) return
  logModalVisible.value = false
  logTarget.value = null
}

// ===== 批量管理 =====
const batchMode = ref(false)
const selectedIds = ref<Set<number>>(new Set())

const allSelected = computed(() => {
  return filteredList.value.length > 0 && filteredList.value.every(f => selectedIds.value.has(f.id))
})

function enterBatchMode() {
  batchMode.value = true
  selectedIds.value.clear()
}

function exitBatchMode() {
  batchMode.value = false
  selectedIds.value.clear()
}

function toggleSelect(id: number) {
  if (selectedIds.value.has(id)) {
    selectedIds.value.delete(id)
  } else {
    selectedIds.value.add(id)
  }
  // 触发响应式更新
  selectedIds.value = new Set(selectedIds.value)
}

function toggleSelectAll() {
  if (allSelected.value) {
    // 取消全选（仅取消当前列表的选中）
    filteredList.value.forEach(f => selectedIds.value.delete(f.id))
  } else {
    filteredList.value.forEach(f => selectedIds.value.add(f.id))
  }
  selectedIds.value = new Set(selectedIds.value)
}

async function confirmBatchDelete() {
  if (selectedIds.value.size === 0) return
  const ok = await webConfirm(`确认将选中的 ${selectedIds.value.size} 条记录移入回收站？14 天内可恢复。`, {
    title: '批量删除',
    confirmText: '删除',
  })
  if (!ok) return
  const ids = Array.from(selectedIds.value)
  const res = await adminApi('batch_delete_feedback', { ids })
  if (res.code === 200) {
    showToast(`已删除 ${res.data?.deleted ?? ids.length} 条记录`, 'success')
    exitBatchMode()
    await loadList()
  } else {
    showToast(res.msg || '删除失败')
  }
}

// ===== 回收站 =====
const recycleModalVisible = ref(false)
const recycleLoading = ref(false)
const recycleList = ref<any[]>([])

async function openRecycleBin() {
  recycleModalVisible.value = true
  await loadRecycleBin()
}

function closeRecycleBin() {
  if (recycleLoading.value) return
  recycleModalVisible.value = false
}

async function loadRecycleBin() {
  recycleLoading.value = true
  const res = await adminApi<{ list: any[] }>('list_recycle_bin')
  recycleLoading.value = false
  if (res.code === 200 && res.data) {
    recycleList.value = res.data.list || []
  } else {
    recycleList.value = []
    showToast(res.msg || '回收站加载失败')
  }
}

async function restoreItem(id: number) {
  const res = await adminApi('restore_feedback', { id })
  if (res.code === 200) {
    showToast('恢复成功', 'success')
    // 从回收站列表移除
    recycleList.value = recycleList.value.filter(r => r.id !== id)
    // 刷新主列表
    await loadList()
  } else {
    showToast(res.msg || '恢复失败')
  }
}

onMounted(() => {
  loadFeedbackLimit()
  loadList()
  window.addEventListener('keydown', onViewerKeydown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', onViewerKeydown)
})
</script>

<style scoped>
.fb-page {
  max-width: 920px;
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
  display: flex;
  align-items: center;
  gap: 10px;
}
.pending-badge {
  font-size: 12px;
  font-weight: 600;
  padding: 3px 10px;
  border-radius: 20px;
  background: #fffbeb;
  color: #f59e0b;
  animation: badgePulse 2s ease-in-out infinite;
}
@keyframes badgePulse {
  0%, 100% { box-shadow: 0 0 0 0 rgba(245, 158, 11, 0.3); }
  50% { box-shadow: 0 0 0 6px rgba(245, 158, 11, 0); }
}
.page-desc {
  font-size: 13px;
  color: var(--text-muted);
  line-height: 1.6;
  margin: 0;
  max-width: 560px;
}
.btn-refresh {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 10px 18px;
  border-radius: 10px;
  border: 1px solid var(--border);
  background: var(--white);
  color: var(--text);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}
.btn-refresh:hover { border-color: var(--accent); transform: translateY(-1px); }
.btn-refresh:active { transform: scale(0.96); }
.btn-refresh:disabled { opacity: 0.5; cursor: not-allowed; }
.spinning { animation: spin 0.8s linear infinite; }
@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }

.header-actions { display: flex; align-items: center; gap: 10px; flex-shrink: 0; }
.btn-stats {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 10px 18px;
  border-radius: 10px;
  border: 1px solid var(--border);
  background: var(--white);
  color: var(--text);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}
.btn-stats:hover { border-color: var(--accent); transform: translateY(-1px); }
.btn-stats:active { transform: scale(0.96); }
.btn-create {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 10px 18px;
  border-radius: 10px;
  border: none;
  background: var(--accent);
  color: #fff;
  font-size: 14px;
  font-weight: 700;
  cursor: pointer;
  white-space: nowrap;
  box-shadow: 0 4px 14px rgba(26, 26, 26, 0.18);
  transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}
.btn-create:hover { transform: translateY(-1px); opacity: 0.9; }
.btn-create:active { transform: scale(0.96); }

/* ===== 工具条：类型筛选 + 排序 ===== */
.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
  flex-wrap: wrap;
}
.toolbar-group { display: flex; gap: 6px; }
.tool-btn {
  padding: 7px 14px;
  border-radius: 9px;
  border: 1px solid var(--border);
  background: var(--white);
  color: var(--text-light);
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}
.tool-btn:hover { border-color: var(--accent); color: var(--text); }
.tool-btn.active { background: var(--accent); color: #fff; border-color: var(--accent); }
.sort-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 32px;
  padding: 0 12px;
  border: 1px solid var(--border);
  border-radius: 9px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text);
  background: var(--white);
  cursor: pointer;
  white-space: nowrap;
  transition: border-color 0.2s, color 0.2s, background 0.2s, transform 0.16s cubic-bezier(0.16, 1, 0.3, 1);
}
.sort-btn:hover {
  border-color: var(--accent);
  color: var(--accent);
  background: var(--accent-soft);
}
.sort-btn:active { transform: scale(0.97); }
.sort-btn svg { flex-shrink: 0; }
.sort-chevron { opacity: 0.6; }

/* ===== 类型徽标 ===== */
.type-badge {
  display: inline-flex;
  align-items: center;
  padding: 3px 10px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 700;
  white-space: nowrap;
}
.type-suggestion { background: #eef2ff; color: #4f46e5; }
.type-problem { background: #fef2f2; color: #dc2626; }
.card-top-right { display: flex; align-items: center; gap: 8px; }

/* ===== 图片缩略图（堆叠，仅显示第一张） ===== */
.img-stack {
  position: relative;
  width: 72px;
  height: 72px;
  flex-shrink: 0;
  cursor: zoom-in;
}
.stack-thumb {
  position: absolute;
  left: 0;
  top: 0;
  width: 72px;
  height: 72px;
  object-fit: cover;
  border-radius: 10px;
  border: 1px solid var(--border);
  background: var(--card);
  transition: transform 0.25s cubic-bezier(0.16, 1, 0.3, 1), box-shadow 0.25s;
}
.img-stack:hover .stack-thumb:first-child {
  transform: scale(1.05);
  box-shadow: 0 6px 16px rgba(0, 0, 0, 0.12);
}
.stack-count {
  position: absolute;
  right: -6px;
  bottom: -6px;
  min-width: 20px;
  height: 20px;
  padding: 0 6px;
  border-radius: 10px;
  background: rgba(0, 0, 0, 0.65);
  color: #fff;
  font-size: 11px;
  font-weight: 600;
  line-height: 20px;
  text-align: center;
  z-index: 20;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.25);
}

/* 主体布局：主内容靠左，图片堆叠于右侧 */
.fb-card .card-body {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: start;
  gap: 16px;
}
.fb-card .card-main { min-width: 0; }
.fb-card .img-stack { margin-top: 0; }

/* ===== 提交限制配置 ===== */
.limit-panel {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
  background: linear-gradient(135deg, #ffffff 0%, #f8fafc 100%);
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 16px 18px;
  margin-bottom: 20px;
  box-shadow: 0 6px 20px rgba(15, 23, 42, 0.04);
}
.limit-info {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}
.limit-icon {
  width: 42px;
  height: 42px;
  border-radius: 12px;
  background: #eef2ff;
  color: #4f46e5;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.limit-info h3 {
  font-size: 15px;
  font-weight: 800;
  color: var(--text);
  margin: 0 0 4px 0;
}
.limit-info p {
  font-size: 12px;
  color: var(--text-muted);
  margin: 0;
  line-height: 1.5;
}
.limit-actions {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-shrink: 0;
}
.limit-input {
  width: 110px;
  height: 38px;
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 0 12px;
  font-size: 14px;
  font-weight: 700;
  color: var(--text);
  outline: none;
  transition: border-color 0.2s, box-shadow 0.2s;
}
.limit-input:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 3px rgba(26, 26, 26, 0.08);
}
.limit-input:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
.btn-save-limit {
  height: 38px;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 0 16px;
  border-radius: 10px;
  border: none;
  background: var(--accent);
  color: #fff;
  font-size: 13px;
  font-weight: 700;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.2s;
}
.btn-save-limit:hover { opacity: 0.86; transform: translateY(-1px); }
.btn-save-limit:active { transform: scale(0.96); }
.btn-save-limit:disabled { opacity: 0.55; cursor: not-allowed; transform: none; }

/* ===== 统计卡片 ===== */
.stats-row {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 12px;
  margin-bottom: 20px;
}
.stat-chip {
  background: var(--white);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 14px 16px;
  display: flex;
  align-items: center;
  gap: 12px;
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}
.stat-chip:hover { transform: translateY(-2px); box-shadow: 0 6px 20px rgba(0, 0, 0, 0.06); }
.stat-chip.active { border-color: var(--accent); box-shadow: 0 0 0 2px rgba(26, 26, 26, 0.08); }
.stat-icon {
  width: 38px; height: 38px;
  border-radius: 10px;
  display: flex; align-items: center; justify-content: center;
  flex-shrink: 0;
}
.stat-icon-total { background: #f0f0f0; color: #1a1a1a; }
.stat-icon-pending { background: #fffbeb; color: #f59e0b; }
.stat-icon-processing { background: #eff6ff; color: #3b82f6; }
.stat-icon-resolved { background: #f0fdf4; color: #16a34a; }
.stat-icon-rejected { background: #fef2f2; color: #dc2626; }
.stat-body { display: flex; flex-direction: column; }
.stat-num { font-size: 22px; font-weight: 800; line-height: 1.1; color: var(--text); }
.stat-label { font-size: 11px; color: var(--text-muted); margin-top: 2px; }

/* ===== 反馈卡片 ===== */
.fb-list { display: flex; flex-direction: column; gap: 14px; }
.fb-card {
  position: relative;
  background: var(--white);
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 18px 20px;
  transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
  animation: cardIn 0.5s cubic-bezier(0.16, 1, 0.3, 1) both;
  border-left: 3px solid transparent;
}
.fb-card:hover { box-shadow: 0 8px 24px rgba(0, 0, 0, 0.06); transform: translateY(-2px); }
.fb-card.st-pending { border-left-color: #f59e0b; }
.fb-card.st-processing { border-left-color: #3b82f6; }
.fb-card.st-resolved { border-left-color: #16a34a; }
.fb-card.st-rejected { border-left-color: #dc2626; }
@keyframes cardIn {
  from { opacity: 0; transform: translateY(16px); }
  to { opacity: 1; transform: translateY(0); }
}

.card-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}
.card-user { display: flex; align-items: center; gap: 10px; }
.user-avatar {
  width: 36px; height: 36px;
  border-radius: 50%;
  background: #f0f0f0;
  display: flex; align-items: center; justify-content: center;
  color: #999;
  flex-shrink: 0;
}
.user-avatar.avatar-img { object-fit: cover; cursor: zoom-in; }
.user-info { display: flex; flex-direction: column; gap: 1px; }
.user-name { font-size: 14px; font-weight: 600; color: var(--text); }
.user-id { font-size: 11px; color: var(--text-muted); }

.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 12px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 600;
}
.badge-pending { background: #fffbeb; color: #f59e0b; }
.badge-processing { background: #eff6ff; color: #3b82f6; }
.badge-resolved { background: #f0fdf4; color: #16a34a; }
.badge-rejected { background: #fef2f2; color: #dc2626; }

.card-body { margin-bottom: 12px; }
.fb-content {
  font-size: 11px;
  color: var(--text-muted);
  line-height: 1.6;
  margin: 0;
  white-space: pre-wrap;
  word-break: break-word;
}
.fb-content-main {
  font-size: 14px;
  font-weight: 700;
  color: var(--text);
  margin: 0 0 6px 0;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
.log-summary {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 10px;
}
.log-chip {
  display: inline-flex;
  align-items: center;
  padding: 3px 8px;
  border-radius: 999px;
  background: #f4f4f5;
  color: #52525b;
  font-size: 11px;
  font-weight: 600;
}

/* 卡片底部 */
.card-foot {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}
.foot-meta { display: flex; align-items: center; gap: 12px; }
.meta-ip, .meta-time {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: var(--text-muted);
}
.foot-actions { display: flex; gap: 8px; }

.act-btn {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 7px 14px;
  border-radius: 8px;
  border: none;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}
.act-btn:active { transform: scale(0.95); }
.act-log { background: #f4f4f5; color: #52525b; }
.act-log:hover { background: #e4e4e7; }
.act-resolve { background: #f0fdf4; color: #16a34a; }
.act-resolve:hover { background: #dcfce7; }
.act-reject { background: #fef2f2; color: #dc2626; }
.act-reject:hover { background: #fee2e2; }
.act-claim { background: #eff6ff; color: #3b82f6; }
.act-claim:hover { background: #dbeafe; }

/* ===== 认领人 / 完成说明 ===== */
.assignee-row {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  margin-top: 10px;
  padding: 4px 10px;
  border-radius: 8px;
  background: #eff6ff;
  color: #3b82f6;
  font-size: 12px;
  font-weight: 600;
}
.resolve-note {
  display: flex;
  align-items: flex-start;
  gap: 7px;
  margin-top: 10px;
  padding: 10px 12px;
  border-radius: 10px;
  background: #f0fdf4;
  color: #16a34a;
  font-size: 12px;
}
.resolve-note svg { flex-shrink: 0; margin-top: 1px; }
.resolve-text { display: flex; flex-direction: column; gap: 2px; min-width: 0; }
.resolve-label { font-weight: 700; }
.resolve-text span:last-child { color: #15803d; white-space: pre-wrap; word-break: break-word; }

/* ===== 完成说明弹窗 ===== */
.resolve-dialog { max-width: 520px; }
.resolve-target-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 12px 14px;
  background: #f8f9fc;
  border-radius: 10px;
  margin-bottom: 16px;
}
.resolve-target-info strong { font-size: 14px; color: var(--text); }
.resolve-target-info span { font-size: 12px; color: var(--text-muted); }
.resolve-field { position: relative; display: block; }
.resolve-field-label {
  display: block;
  font-size: 13px;
  font-weight: 600;
  color: var(--text);
  margin-bottom: 8px;
}
.resolve-field-label em { color: #dc2626; font-style: normal; }
.resolve-textarea {
  width: 100%;
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 12px;
  font-size: 13px;
  font-family: inherit;
  resize: vertical;
  min-height: 120px;
  outline: none;
  transition: border-color 0.2s, box-shadow 0.2s;
  box-sizing: border-box;
  background: var(--white);
  color: var(--text);
}
.resolve-textarea:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 3px rgba(26, 26, 26, 0.08);
}
.resolve-count {
  position: absolute;
  right: 12px;
  bottom: 10px;
  font-size: 11px;
  color: var(--text-muted);
  pointer-events: none;
}
.btn-confirm {
  padding: 9px 20px;
  border-radius: 10px;
  border: none;
  background: #16a34a;
  color: #fff;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}
.btn-confirm:hover { opacity: 0.88; }
.btn-confirm:disabled { opacity: 0.5; cursor: not-allowed; }

/* ===== 回复弹窗 ===== */
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
  overflow-y: auto;
}
.modal-dialog {
  background: var(--white);
  border-radius: 16px;
  width: 100%;
  max-width: 480px;
  margin: auto;
  overflow: hidden;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.15);
}
.log-dialog {
  max-width: min(920px, calc(100vw - 40px));
}
.modal-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 18px 20px;
  border-bottom: 1px solid var(--border);
}
.modal-head h3 { font-size: 16px; font-weight: 700; margin: 0; }
.modal-close {
  width: 32px; height: 32px;
  border: none;
  background: #f5f5f5;
  border-radius: 8px;
  cursor: pointer;
  display: flex; align-items: center; justify-content: center;
  color: var(--text-muted);
  transition: all 0.2s;
}
.modal-close:hover { background: #e5e5e5; color: var(--text); }
.modal-body { padding: 20px; }
.log-target-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 12px 14px;
  background: #f8f9fc;
  border-radius: 10px;
  margin-bottom: 16px;
}
.log-target-info strong { font-size: 14px; color: var(--text); }
.log-target-info span { font-size: 12px; color: var(--text-muted); }
.field { margin-bottom: 0; }
.field label {
  display: block;
  font-size: 13px;
  font-weight: 600;
  color: var(--text);
  margin-bottom: 8px;
}
.field textarea {
  width: 100%;
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 12px;
  font-size: 13px;
  font-family: inherit;
  resize: vertical;
  min-height: 100px;
  outline: none;
  transition: border-color 0.2s;
  box-sizing: border-box;
}
.field textarea:focus { border-color: var(--accent); }
.log-tabs {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
}
.log-tab {
  border: 1px solid var(--border);
  background: var(--white);
  color: var(--text-light);
  border-radius: 8px;
  padding: 7px 12px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
}
.log-tab.active {
  background: var(--accent);
  color: #fff;
  border-color: var(--accent);
}
.log-tab:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}
.log-content {
  max-height: 520px;
  overflow: auto;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: #0f172a;
  color: #e5e7eb;
  padding: 14px;
  font-size: 12px;
  line-height: 1.6;
  white-space: pre;
}
.modal-foot {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  padding: 16px 20px;
  border-top: 1px solid var(--border);
}
.btn-cancel {
  padding: 9px 18px;
  border-radius: 10px;
  border: 1px solid var(--border);
  background: var(--white);
  color: var(--text-light);
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}
.btn-cancel:hover { background: #f5f5f5; }
.btn-save {
  padding: 9px 20px;
  border-radius: 10px;
  border: none;
  background: var(--accent);
  color: #fff;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  transition: all 0.2s;
}
.btn-save:hover { opacity: 0.85; }
.btn-save:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-spinner {
  width: 14px; height: 14px;
  border: 2px solid rgba(255,255,255,0.3);
  border-top-color: #fff;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}
.btn-spinner.dark {
  border-color: rgba(255,255,255,0.35);
  border-top-color: #fff;
}

/* ===== 新建事项弹窗 ===== */
.create-dialog {
  max-width: 520px;
  max-height: calc(100vh - 40px);
  display: flex;
  flex-direction: column;
}
.create-dialog .modal-body { overflow-y: auto; }
.create-type-row { display: flex; gap: 8px; margin-bottom: 16px; }
.create-type-btn {
  flex: 1;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 10px 12px;
  border-radius: 10px;
  border: 1px solid var(--border);
  background: var(--white);
  color: var(--text-light);
  font-size: 13px;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}
.create-type-btn:hover { border-color: var(--accent); }
.create-type-btn.active { background: var(--accent); color: #fff; border-color: var(--accent); }
.create-field { display: block; margin-bottom: 16px; }
.create-field-label {
  display: block;
  font-size: 13px;
  font-weight: 600;
  color: var(--text);
  margin-bottom: 8px;
}
.create-field-label em { color: #dc2626; font-style: normal; }
.create-field-label .optional { color: var(--text-muted); font-weight: 500; font-size: 12px; }
.create-input {
  width: 100%;
  height: 40px;
  box-sizing: border-box;
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 0 12px;
  font-size: 13px;
  color: var(--text);
  background: var(--white);
  outline: none;
  transition: border-color 0.2s, box-shadow 0.2s;
}
.create-input:focus { border-color: var(--accent); box-shadow: 0 0 0 3px rgba(26, 26, 26, 0.08); }
.create-textarea {
  width: 100%;
  box-sizing: border-box;
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 12px;
  font-size: 13px;
  font-family: inherit;
  color: var(--text);
  background: var(--white);
  resize: vertical;
  outline: none;
  transition: border-color 0.2s, box-shadow 0.2s;
}
.create-textarea:focus { border-color: var(--accent); box-shadow: 0 0 0 3px rgba(26, 26, 26, 0.08); }
.create-count {
  display: block;
  text-align: right;
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 4px;
}
.create-dropzone {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 22px 16px;
  border: 1.5px dashed var(--border);
  border-radius: 12px;
  background: var(--white);
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}
.create-dropzone:hover, .create-dropzone.dragging {
  border-color: var(--accent);
  background: rgba(26, 26, 26, 0.03);
}
.create-dropzone.dragging { border-width: 2px; }
.create-dropzone.has { border-style: solid; border-color: #16a34a; background: #f0fdf4; }
.dropzone-icon {
  width: 42px; height: 42px;
  border-radius: 12px;
  background: #eef2ff;
  color: #4f46e5;
  display: flex; align-items: center; justify-content: center;
  flex-shrink: 0;
}
.dropzone-text { display: flex; flex-direction: column; gap: 2px; }
.dropzone-text strong { font-size: 13px; color: var(--text); }
.dropzone-text span { font-size: 11px; color: var(--text-muted); }
.create-preview { display: flex; flex-wrap: wrap; gap: 8px; margin-top: 10px; }
.preview-item { position: relative; }
.preview-img {
  width: 76px; height: 76px;
  object-fit: cover;
  border-radius: 10px;
  border: 1px solid var(--border);
  cursor: zoom-in;
}
.preview-remove {
  position: absolute;
  top: -6px; right: -6px;
  width: 20px; height: 20px;
  border: none;
  border-radius: 50%;
  background: #dc2626;
  color: #fff;
  display: flex; align-items: center; justify-content: center;
  cursor: pointer;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.2);
  transition: transform 0.2s;
}
.preview-remove:hover { transform: scale(1.1); }
.create-notify {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 12px 14px;
  border-radius: 12px;
  background: #f8f9fc;
  border: 1px solid var(--border);
  cursor: pointer;
}
.notify-check { display: none; }
.notify-box {
  width: 20px; height: 20px;
  border-radius: 6px;
  border: 1.5px solid var(--border);
  background: var(--white);
  display: flex; align-items: center; justify-content: center;
  color: transparent;
  flex-shrink: 0;
  margin-top: 1px;
  transition: all 0.2s;
}
.notify-check:checked + .notify-box { background: var(--accent); border-color: var(--accent); color: #fff; }
.notify-text { display: flex; flex-direction: column; gap: 2px; }
.notify-text strong { font-size: 13px; color: var(--text); }
.notify-text span { font-size: 11px; color: var(--text-muted); line-height: 1.5; }
.btn-create-submit { background: var(--accent); }

/* ===== 处理统计弹窗 ===== */
.stats-dialog { max-width: 560px; }
.stats-total {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  padding: 18px;
  margin-bottom: 16px;
  border-radius: 12px;
  background: linear-gradient(135deg, #ffffff 0%, #f0f4ff 100%);
  border: 1px solid var(--border);
}
.stats-total-num { font-size: 34px; font-weight: 800; color: var(--text); line-height: 1; }
.stats-total-label { font-size: 12px; color: var(--text-muted); }
.stats-table-wrap { overflow: auto; }
.stats-table { width: 100%; border-collapse: collapse; }
.stats-table th {
  text-align: left;
  font-size: 11px;
  font-weight: 700;
  color: var(--text-muted);
  padding: 9px 10px;
  border-bottom: 1px solid var(--border);
  white-space: nowrap;
}
.stats-table td {
  text-align: left;
  font-size: 13px;
  color: var(--text);
  padding: 11px 10px;
  border-bottom: 1px solid var(--border);
  white-space: nowrap;
}
.stats-table tbody tr:last-child td { border-bottom: none; }
.stats-table tbody tr { transition: background 0.2s; }
.stats-table tbody tr:hover { background: rgba(26, 26, 26, 0.03); }
.stats-admin { display: flex; align-items: center; gap: 8px; font-weight: 600; }
.admin-dot { width: 10px; height: 10px; border-radius: 50%; flex-shrink: 0; }
.dot-unclaimed { background: #a1a1aa; }
.dot-0 { background: #3b82f6; }
.dot-1 { background: #16a34a; }
.dot-2 { background: #f59e0b; }
.dot-3 { background: #8b5cf6; }
.stats-num-cell { font-weight: 600; }
.stats-num-cell strong { font-size: 15px; }
.stats-num-cell.processing { color: #3b82f6; }
.stats-num-cell.resolved { color: #16a34a; }
.stats-num-cell.rejected { color: #dc2626; }
.stats-num-cell.pending { color: #f59e0b; }

/* ===== 图片查看器 ===== */
.viewer-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.88);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
  padding: 40px;
  animation: viewerIn 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}
@keyframes viewerIn { from { opacity: 0; } to { opacity: 1; } }
.viewer-img {
  max-width: 90vw;
  max-height: 90vh;
  object-fit: contain;
  border-radius: 8px;
  opacity: 0;
  transform: scale(0.96);
  transition: opacity 0.25s, transform 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}
.viewer-img.ready { opacity: 1; transform: scale(1); }
.viewer-close {
  position: absolute;
  top: 20px; right: 20px;
  width: 42px; height: 42px;
  border: none;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.12);
  color: #fff;
  display: flex; align-items: center; justify-content: center;
  cursor: pointer;
  backdrop-filter: blur(6px);
  transition: background 0.2s;
}
.viewer-close:hover { background: rgba(255, 255, 255, 0.24); }
.viewer-nav {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  width: 46px; height: 46px;
  border: none;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.12);
  color: #fff;
  display: flex; align-items: center; justify-content: center;
  cursor: pointer;
  backdrop-filter: blur(6px);
  transition: background 0.2s;
}
.viewer-nav:hover { background: rgba(255, 255, 255, 0.24); }
.viewer-prev { left: 20px; }
.viewer-next { right: 20px; }
.viewer-counter {
  position: absolute;
  bottom: 22px;
  left: 50%;
  transform: translateX(-50%);
  padding: 6px 14px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.14);
  color: #fff;
  font-size: 13px;
  font-weight: 700;
  backdrop-filter: blur(6px);
}

/* ===== 空状态 / 加载 ===== */
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
.state-box.compact {
  padding: 24px 20px;
}
.state-empty { padding: 48px 20px; }
.empty-icon { color: #d0d0d0; margin-bottom: 4px; }
.empty-title { font-size: 15px; font-weight: 600; color: var(--text-light); margin: 0; }
.empty-sub { font-size: 13px; color: var(--text-muted); margin: 0; }

/* ===== Spinner ===== */
.spinner {
  width: 32px; height: 32px;
  border: 3px solid #e5e5e5;
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

/* ===== 过渡动画 ===== */
.fade-down-enter-active, .fade-down-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-down-enter-from { opacity: 0; transform: translateY(-12px); }

.fade-up-enter-active, .fade-up-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-up-enter-from { opacity: 0; transform: translateY(12px); }

.fb-card-enter-active { transition: all 0.5s cubic-bezier(0.16, 1, 0.3, 1); }
.fb-card-enter-from { opacity: 0; transform: translateY(16px); }
.fb-card-leave-active { transition: all 0.3s ease; }
.fb-card-leave-to { opacity: 0; transform: scale(0.95); }
.fb-card-move { transition: transform 0.4s cubic-bezier(0.16, 1, 0.3, 1); }

.modal-enter-active, .modal-leave-active { transition: opacity 0.3s ease; }
.modal-enter-from, .modal-leave-to { opacity: 0; }
.modal-enter-active .modal-dialog { animation: modalIn 0.3s cubic-bezier(0.16, 1, 0.3, 1); }
@keyframes modalIn {
  from { opacity: 0; transform: scale(0.92) translateY(20px); }
  to { opacity: 1; transform: scale(1) translateY(0); }
}

/* ===== 回收站按钮 ===== */
.btn-recycle {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border-radius: 10px;
  border: 1px solid var(--border-color);
  background: var(--card-bg);
  color: var(--text-muted);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}
.btn-recycle:hover {
  border-color: #ef4444;
  color: #ef4444;
  background: rgba(239, 68, 68, 0.06);
}

/* ===== 工具条右侧 ===== */
.toolbar-right {
  display: flex;
  align-items: center;
  gap: 10px;
}
.batch-slot {
  display: flex;
  align-items: center;
}
.batch-abs {
  display: flex;
  align-items: center;
}

/* ===== 批量管理 ===== */
.btn-batch-enter {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 6px 12px;
  border-radius: 8px;
  border: 1px solid var(--border-color);
  background: var(--card-bg);
  color: var(--text-secondary);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}
.btn-batch-enter:hover {
  border-color: var(--accent-color);
  color: var(--accent-color);
}
/* 批量菜单向左弹出/收回动效 */
.batch-slide-enter-active,
.batch-slide-leave-active {
  transition: opacity 0.22s cubic-bezier(0.16, 1, 0.3, 1),
              transform 0.22s cubic-bezier(0.16, 1, 0.3, 1);
}
.batch-slide-enter-from {
  opacity: 0;
  transform: translateX(30px);
}
.batch-slide-leave-to {
  opacity: 0;
  transform: translateX(30px);
}
.batch-bar {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 2px 10px;
  height: 32px;
  border-radius: 9px;
  background: var(--accent-soft);
  border: 1px solid var(--border);
}
.btn-batch-select {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 0;
  border: none;
  background: transparent;
  color: var(--text);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  white-space: nowrap;
}
.checkbox-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  border: 1.5px solid var(--border);
  border-radius: 5px;
  background: var(--white);
  cursor: pointer;
  transition: all 0.15s;
  vertical-align: -2px;
  margin-right: 4px;
}
.checkbox-badge.checked {
  background: var(--accent);
  border-color: var(--accent);
  color: #fff;
}
.checkbox-badge:hover { border-color: var(--accent); }
.batch-count {
  font-size: 12px;
  color: var(--accent);
  font-weight: 600;
  white-space: nowrap;
}
.btn-batch-delete {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 9px;
  border-radius: 6px;
  border: none;
  background: #ef4444;
  color: #fff;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}
.btn-batch-delete:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
.btn-batch-delete:not(:disabled):hover {
  background: #dc2626;
}
.btn-batch-exit {
  padding: 4px 9px;
  border-radius: 6px;
  border: 1px solid var(--border-color);
  background: var(--card-bg);
  color: var(--text-muted);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}
.btn-batch-exit:hover {
  color: var(--text-primary);
}

/* ===== 卡片复选框 ===== */
.card-checkbox {
  position: absolute;
  top: 14px;
  right: 14px;
  z-index: 2;
  width: 22px;
  height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}
.card-checkbox input {
  width: 18px;
  height: 18px;
  cursor: pointer;
  accent-color: var(--accent-color);
}
.fb-card.batch-selected {
  border-color: var(--accent-color) !important;
  box-shadow: 0 0 0 2px rgba(var(--accent-rgb, 99, 102, 241), 0.15);
}

/* ===== 封禁申诉类型样式 ===== */
.avatar-appeal {
  background: linear-gradient(135deg, #fef3c7, #fde68a);
  color: #92400e;
}
.type-appeal {
  background: #fef3c7 !important;
  color: #92400e !important;
}
.type-badge.small {
  font-size: 10px;
  padding: 2px 7px;
  border-radius: 4px;
  font-weight: 600;
}

/* ===== 回收站弹窗 ===== */
.recycle-dialog {
  max-width: 560px;
  width: 90%;
}
.recycle-tip {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 10px 14px;
  border-radius: 8px;
  background: rgba(245, 158, 11, 0.08);
  color: #92400e;
  font-size: 12px;
  line-height: 1.5;
  margin-bottom: 14px;
}
.recycle-tip svg {
  flex-shrink: 0;
  margin-top: 1px;
}
.recycle-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 400px;
  overflow-y: auto;
}
.recycle-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 12px 14px;
  border-radius: 10px;
  background: var(--hover-bg);
  border: 1px solid var(--border-color);
  transition: all 0.2s;
}
.recycle-item:hover {
  border-color: var(--accent-color);
}
.recycle-item-main {
  flex: 1;
  min-width: 0;
}
.recycle-item-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 4px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.recycle-item-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  font-size: 11px;
  color: var(--text-muted);
}
.recycle-user {
  font-weight: 500;
}
.recycle-del {
  color: var(--text-muted);
}
.recycle-remaining {
  color: #16a34a;
  font-weight: 500;
}
.recycle-remaining.urgent {
  color: #ef4444;
}
.btn-restore {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 6px 12px;
  border-radius: 8px;
  border: 1px solid #16a34a;
  background: rgba(22, 163, 74, 0.06);
  color: #16a34a;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
  flex-shrink: 0;
}
.btn-restore:hover {
  background: #16a34a;
  color: #fff;
}

/* ===== 响应式 ===== */
@media (max-width: 768px) {
  .stats-row { grid-template-columns: 1fr 1fr 1fr; gap: 8px; }
  .stat-chip { padding: 10px 12px; flex-direction: column; align-items: flex-start; gap: 6px; }
  .stat-num { font-size: 18px; }
  .stat-label { font-size: 10px; }
  .card-foot { flex-direction: column; align-items: stretch; }
  .foot-actions { justify-content: flex-end; }
  .fb-card { padding: 14px 16px; }
  .toolbar { flex-direction: column; gap: 10px; align-items: stretch; }
  .toolbar-right { flex-wrap: wrap; }
  .toolbar-group { flex-wrap: wrap; }
}
</style>
