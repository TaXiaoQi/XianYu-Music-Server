<template>
  <div class="mobile-page">
    <!-- 页面头部 -->
    <section class="mobile-page-head">
      <div class="mobile-head-info">
        <div class="mobile-head-title">
          反馈与建议
          <span v-if="stats.pending > 0" class="pending-badge">{{ stats.pending }} 项待处理</span>
        </div>
        <div class="mobile-head-desc">查看用户提交的反馈与建议，将问题标记为已解决或已拒绝。</div>
      </div>
      <div class="mobile-head-actions">
        <button class="mobile-btn" @click="openStats">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 20V10"/><path d="M12 20V4"/><path d="M6 20v-6"/></svg>
          统计
        </button>
        <button class="mobile-btn primary" @click="openCreate">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
          新建
        </button>
      </div>
    </section>

    <!-- 提交限制配置 -->
    <section class="mobile-card limit-panel">
      <div class="limit-info">
        <div class="limit-icon">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/><path d="M12 8v4"/><path d="M12 16h.01"/></svg>
        </div>
        <div class="limit-text">
          <h3>每日反馈提交上限</h3>
          <p>每个用户每天最多可提交 {{ limitInput === 0 ? '不限' : `${limitInput} 条` }}，修改后立即生效。</p>
        </div>
      </div>
      <div class="limit-actions">
        <input v-model.number="limitInput" class="mobile-input" type="number" min="0" max="10000" step="1" :disabled="limitSaving" @keyup.enter="saveLimit" />
        <button class="mobile-btn primary" :disabled="limitSaving" @click="saveLimit">{{ limitSaving ? '保存中...' : '保存上限' }}</button>
        <button class="mobile-btn" @click="openRecycleBin">
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/><line x1="10" y1="11" x2="10" y2="17"/><line x1="14" y1="11" x2="14" y2="17"/></svg>
          回收站
        </button>
      </div>
    </section>

    <!-- 统计卡片 -->
    <section class="stats-row">
      <div class="stat-chip" :class="{ active: status === '' }" @click="setStatus('')">
        <div class="stat-icon stat-icon-total"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg></div>
        <span class="stat-num">{{ stats.total }}</span><span class="stat-label">全部</span>
      </div>
      <div class="stat-chip" :class="{ active: status === 'pending' }" @click="setStatus('pending')">
        <div class="stat-icon stat-icon-pending"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg></div>
        <span class="stat-num">{{ stats.pending }}</span><span class="stat-label">待处理</span>
      </div>
      <div class="stat-chip" :class="{ active: status === 'processing' }" @click="setStatus('processing')">
        <div class="stat-icon stat-icon-processing"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2v4"/><path d="M12 18v4"/><path d="M4.93 4.93l2.83 2.83"/><path d="M16.24 16.24l2.83 2.83"/><circle cx="12" cy="12" r="4"/></svg></div>
        <span class="stat-num">{{ stats.processing }}</span><span class="stat-label">处理中</span>
      </div>
      <div class="stat-chip" :class="{ active: status === 'resolved' }" @click="setStatus('resolved')">
        <div class="stat-icon stat-icon-resolved"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg></div>
        <span class="stat-num">{{ stats.resolved }}</span><span class="stat-label">已解决</span>
      </div>
      <div class="stat-chip" :class="{ active: status === 'rejected' }" @click="setStatus('rejected')">
        <div class="stat-icon stat-icon-rejected"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg></div>
        <span class="stat-num">{{ stats.rejected }}</span><span class="stat-label">已拒绝</span>
      </div>
    </section>

    <!-- 工具条：类型筛选 + 排序 + 批量 -->
    <section class="toolbar">
      <div class="toolbar-group">
        <button class="tool-btn" :class="{ active: type === '' }" @click="type = ''">全部类型</button>
        <button class="tool-btn" :class="{ active: type === 'problem' }" @click="type = 'problem'">问题反馈</button>
        <button class="tool-btn" :class="{ active: type === 'suggestion' }" @click="type = 'suggestion'">功能建议</button>
        <button class="tool-btn" :class="{ active: type === 'appeal' }" @click="type = 'appeal'">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" style="vertical-align:-2px;margin-right:2px;"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
          封禁申诉
        </button>
      </div>
      <div class="toolbar-right">
        <button class="sort-btn" @click="openSortMenu">
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 5h10"/><path d="M11 9h7"/><path d="M11 13h4"/><path d="M3 17l3 3 3-3"/><path d="M6 18V4"/></svg>
          {{ sortLabel }}
        </button>
        <div class="batch-slot">
          <Transition name="batch-slide">
            <div v-if="batchMode" key="batch" class="batch-bar batch-abs">
              <label class="batch-select-all">
                <input type="checkbox" :checked="allSelected" @change="toggleSelectAll" />
                <span>全选</span>
              </label>
              <span class="batch-count">已选 {{ selectedIds.size }} 项</span>
              <button class="mobile-btn primary" :disabled="selectedIds.size === 0" @click="confirmBatchDelete">删除所选</button>
              <button class="mobile-btn" @click="exitBatchMode">退出</button>
            </div>
            <button v-else key="enter" class="mobile-btn batch-abs" @click="enterBatchMode">
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/></svg>
              批量
            </button>
          </Transition>
        </div>
      </div>
    </section>

    <div v-if="loading" class="mobile-empty"><span class="loader"></span> 加载中...</div>
    <div v-else-if="filteredList.length === 0" class="mobile-empty">
      <p class="empty-title">{{ status === '' ? '暂无反馈记录' : '该状态下暂无反馈' }}</p>
      <p class="empty-sub">用户提交的反馈与建议将显示在这里</p>
    </div>
    <div v-else class="mobile-list">
      <div v-for="f in filteredList" :key="f.id" class="mobile-item mfb-item" :class="{ 'batch-selected': selectedIds.has(f.id) }">
        <!-- 批量选择 -->
        <div v-if="batchMode" class="card-checkbox" @click.stop="toggleSelect(f.id)">
          <input type="checkbox" :checked="selectedIds.has(f.id)" />
        </div>
        <!-- 卡片头部 -->
        <div class="mfb-head">
          <div class="mfb-user">
            <img
              v-if="f.avatar_url"
              :src="f.avatar_url"
              alt="头像"
              class="user-avatar avatar-img"
              @click.stop="openViewer([f.avatar_url], 0)"
            />
            <div v-else class="user-avatar" :class="f.category === 'appeal' ? 'avatar-appeal' : (f.feedback_type === 'suggestion' ? 'avatar-suggestion' : 'avatar-problem')">
              <svg v-if="f.category === 'appeal'" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/><path d="M12 8v4"/><path d="M12 16h.01"/></svg>
              <svg v-else-if="f.feedback_type === 'suggestion'" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/></svg>
              <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
            </div>
            <div class="user-info">
              <span class="user-name">{{ f.nickname || '匿名用户' }}</span>
              <span class="user-id">{{ f.ciyuanxi_id || '后台创建' }}</span>
            </div>
          </div>
          <div class="mfb-head-right">
            <span v-if="f.category === 'appeal'" class="type-badge type-appeal">封禁申诉</span>
            <span v-else-if="f.feedback_type" class="type-badge" :class="f.feedback_type === 'suggestion' ? 'type-suggestion' : 'type-problem'">
              {{ f.feedback_type === 'suggestion' ? '功能建议' : '问题反馈' }}
            </span>
            <span class="status-badge" :class="`badge-${f.status}`">
              <svg v-if="f.status === 'resolved'" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><polyline points="20 6 9 17 4 12"/></svg>
              <svg v-else-if="f.status === 'rejected'" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
              <svg v-else-if="f.status === 'processing'" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
              {{ statusLabel(f.status) }}
            </span>
          </div>
        </div>

        <!-- 主体 -->
        <div class="mfb-main">
          <div class="mfb-left">
            <h3 class="mfb-title">{{ f.title || '无标题' }}</h3>
            <p class="mfb-content">{{ f.content || '无内容' }}</p>
            <div class="detail-more">
              <div v-if="hasErrorLogs(f) || hasAllLogs(f)" class="log-summary">
                <span v-if="hasErrorLogs(f)" class="log-chip">错误日志 {{ formatLogSize(f.error_logs_chars) }}</span>
                <span v-if="hasAllLogs(f)" class="log-chip">全量日志 {{ formatLogSize(f.all_logs_chars) }}</span>
              </div>
              <div v-if="f.assignee" class="assignee-row">
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
                <span>{{ f.status === 'rejected' ? '拒绝人' : '认领人' }}：{{ f.assignee }}</span>
              </div>
              <div v-if="f.status === 'resolved' && f.resolve_note" class="resolve-note">
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
                <div class="resolve-text"><span class="resolve-label">完成说明</span><span>{{ f.resolve_note }}</span></div>
              </div>
            </div>
          </div>
          <!-- 图片堆叠（仅显示第一张） -->
          <div v-if="imagesOf(f).length > 0" class="img-stack" @click="openViewer(imagesOf(f), 0)">
            <img
              v-for="(img, i) in imagesOf(f)"
              :key="i"
              :src="img"
              class="stack-thumb"
              :style="stackThumbStyle(i, imagesOf(f).length)"
              alt="反馈图片"
            />
            <span v-if="imagesOf(f).length > 1" class="stack-count">{{ imagesOf(f).length }}</span>
          </div>
        </div>

        <!-- 卡片底部 -->
        <div class="mfb-foot">
          <div class="foot-meta">
            <span>发布时间：{{ fmtTime(f.created_at) }}</span>
            <span v-if="f.claimed_at"> | 认领时间：{{ fmtTime(f.claimed_at) }}</span>
            <span v-if="f.status === 'resolved' && f.resolved_at"> | 完成时间：{{ fmtTime(f.resolved_at) }}</span>
          </div>
          <div class="foot-actions">
            <button v-if="hasErrorLogs(f) || hasAllLogs(f)" class="act-btn act-log" @click="openLogModal(f)">
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="8" y1="13" x2="16" y2="13"/><line x1="8" y1="17" x2="16" y2="17"/></svg>
              日志
            </button>
            <button v-if="f.status === 'pending' || (f.status === 'processing' && !isMineFeedback(f))" class="act-btn act-claim" @click="claim(f)">
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
              认领
            </button>
            <button v-if="f.status === 'processing' && isMineFeedback(f)" class="act-btn act-resolve" @click="openResolve(f)">
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12"/></svg>
              完成
            </button>
            <button v-if="(f.status === 'pending' || f.status === 'processing') && (f.status === 'pending' || isMineFeedback(f))" class="act-btn act-reject" @click="setStatusOf(f, 'rejected')">
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
              拒绝
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- 完成说明弹窗 -->
    <Transition name="mobile-fade">
    <div v-if="resolveVisible" class="mobile-dialog-overlay" @click.self="closeResolve">
      <div class="mobile-dialog" style="display:flex;flex-direction:column;max-width:400px;">
        <div class="mobile-dialog-title">完成反馈</div>
        <div class="resolve-target-info" v-if="resolveTarget">
          <strong>{{ resolveTarget.title || '无标题' }}</strong>
          <span>{{ resolveTarget.nickname || '匿名用户' }} · {{ resolveTarget.ciyuanxi_id || '-' }}</span>
        </div>
        <textarea v-model="resolveNote" class="mobile-dialog-input" rows="4" placeholder="请填写完成说明（必填）" style="min-height:90px;resize:vertical;"></textarea>
        <div class="mobile-dialog-actions">
          <button class="mobile-dialog-btn cancel" @click="closeResolve">取消</button>
          <button class="mobile-dialog-btn confirm" :disabled="!resolveNote.trim()" @click="confirmResolve">{{ resolveSaving ? '提交中...' : '确认完成' }}</button>
        </div>
      </div>
    </div>
    </Transition>

    <!-- 日志弹窗 -->
    <Transition name="mobile-fade">
    <div v-if="logVisible" class="mobile-dialog-overlay" @click.self="closeLog">
      <div class="mobile-dialog" style="display:flex;flex-direction:column;max-width:440px;max-height:88vh;">
        <div class="mobile-dialog-title">反馈日志</div>
        <div class="log-tabs">
          <button class="log-tab" :class="{ active: activeLogTab === 'error' }" :disabled="!logTarget?.error_logs" @click="activeLogTab = 'error'">错误日志</button>
          <button class="log-tab" :class="{ active: activeLogTab === 'all' }" :disabled="!logTarget?.all_logs" @click="activeLogTab = 'all'">全量日志</button>
        </div>
        <div class="log-body">
          <div v-if="logLoading" class="mobile-empty">加载中...</div>
          <pre v-else class="log-content">{{ currentLogText || '暂无日志内容' }}</pre>
        </div>
        <div class="mobile-dialog-actions">
          <button class="mobile-dialog-btn cancel" @click="closeLog">关闭</button>
        </div>
      </div>
    </div>
    </Transition>

    <!-- 新建事项弹窗 -->
    <Transition name="mobile-fade">
    <div v-if="createVisible" class="mobile-dialog-overlay" @click.self="closeCreate">
      <div class="mobile-dialog" style="display:flex;flex-direction:column;max-width:360px;max-height:88vh;">
        <div class="mobile-dialog-title">新建事项</div>
        <div class="mfb-create-body">
          <div class="mfb-type-row">
            <button class="mfb-type-btn" :class="{ active: createType === 'problem' }" @click="createType = 'problem'">问题反馈</button>
            <button class="mfb-type-btn" :class="{ active: createType === 'suggestion' }" @click="createType = 'suggestion'">功能建议</button>
          </div>
          <textarea v-model="createContent" class="mobile-dialog-input" rows="4" placeholder="请输入内容描述（最多 1000 字）" maxlength="1000" style="min-height:90px;resize:vertical;"></textarea>
          <div class="mfb-dropzone" :class="{ dragging: createDragging, has: createImages.length > 0 }" @dragover.prevent="createDragging = true" @dragleave.prevent="createDragging = false" @drop.prevent="onDrop" @click="fileInput?.click()">
            <input ref="fileInput" type="file" accept="image/*" multiple hidden @change="onFileChange" />
            <div class="mfb-dropzone-text">
              <strong>{{ createImages.length > 0 ? `${createImages.length} 张图片已选择` : '点击或拖拽图片到此处' }}</strong>
              <span>支持 JPG / PNG / GIF，单张不超过 8MB</span>
            </div>
          </div>
          <div v-if="createImages.length > 0" class="mfb-preview">
            <div v-for="(img, i) in createImages" :key="i" class="mfb-preview-item">
              <img :src="img" class="mfb-preview-img" @click.stop="openViewer(createImages, i)" />
              <button class="mfb-preview-remove" @click.stop="createImages.splice(i, 1)">×</button>
            </div>
          </div>
          <label class="mfb-notify">
            <input v-model="createNotify" type="checkbox" />
            <span class="mfb-notify-box" :class="{ checked: createNotify }">✓</span>
            <span class="mfb-notify-text">
              <strong>外部同步通知</strong>
              <span>发布后主动向「外部通知」配置中启用的邮箱发送邮件提醒</span>
            </span>
          </label>
        </div>
        <div class="mobile-dialog-actions">
          <button class="mobile-dialog-btn cancel" @click="closeCreate">取消</button>
          <button class="mobile-dialog-btn confirm" :disabled="createSaving || !createContent.trim()" @click="submitCreate">{{ createSaving ? '发布中...' : '发布' }}</button>
        </div>
      </div>
    </div>
    </Transition>

    <!-- 统计弹窗 -->
    <Transition name="mobile-fade">
    <div v-if="statsVisible" class="mobile-dialog-overlay" @click.self="statsVisible = false">
      <div class="mobile-dialog mfb-stats" style="display:flex;flex-direction:column;max-width:420px;max-height:88vh;">
        <div class="mobile-dialog-title">管理员处理统计</div>
        <div class="mfb-stats-body">
          <div v-if="statsLoading" class="mobile-empty">加载中...</div>
          <template v-else>
            <div class="mfb-stats-total">
              <strong>{{ statsGrand }}</strong>
              <span>累计处理反馈总量</span>
            </div>
            <div v-if="statsList.length === 0" class="mobile-empty">暂无统计数据</div>
            <div v-else class="mfb-stats-list">
              <div v-for="(row, i) in statsList" :key="i" class="mfb-stats-row">
                <div class="mfb-stats-admin"><span class="mfb-dot" :class="row.admin_name === '未认领' ? 'dot-unclaimed' : `dot-${i % 4}`"></span>{{ row.admin_name }}</div>
                <div class="mfb-stats-nums">
                  <span>总量 <b>{{ row.total }}</b></span>
                  <span><i class="dot-pending"></i>待处理 <b class="c-pending">{{ row.pending }}</b></span>
                  <span><i class="dot-processing"></i>处理中 <b class="c-processing">{{ row.processing }}</b></span>
                  <span><i class="dot-resolved"></i>已解决 <b class="c-resolved">{{ row.resolved }}</b></span>
                  <span><i class="dot-rejected"></i>已拒绝 <b class="c-rejected">{{ row.rejected }}</b></span>
                </div>
              </div>
            </div>
          </template>
        </div>
        <div class="mobile-dialog-actions">
          <button class="mobile-dialog-btn cancel" @click="statsVisible = false">关闭</button>
        </div>
      </div>
    </div>
    </Transition>

    <!-- 回收站弹窗 -->
    <Transition name="mobile-fade">
    <div v-if="recycleVisible" class="mobile-dialog-overlay" @click.self="closeRecycle">
      <div class="mobile-dialog" style="display:flex;flex-direction:column;max-width:420px;max-height:88vh;">
        <div class="mobile-dialog-title">回收站</div>
        <div class="recycle-body">
          <div class="recycle-tip">已删除的记录将保留 14 天，超期自动永久清除。可点击「恢复」将记录还原。</div>
          <div v-if="recycleLoading" class="mobile-empty">加载中...</div>
          <div v-else-if="recycleList.length === 0" class="mobile-empty">回收站为空</div>
          <div v-else class="recycle-list">
            <div v-for="item in recycleList" :key="item.id" class="recycle-item">
              <div class="recycle-item-main">
                <div class="recycle-item-title">{{ item.title || '无标题' }}</div>
                <div class="recycle-item-meta">
                  <span v-if="item.category === 'appeal'" class="type-badge type-appeal small">封禁申诉</span>
                  <span v-else-if="item.feedback_type === 'suggestion'" class="type-badge type-suggestion small">功能建议</span>
                  <span v-else class="type-badge type-problem small">问题反馈</span>
                  <span class="recycle-user">{{ item.nickname || '匿名' }}</span>
                  <span>删除人：{{ item.deleted_by || '-' }}</span>
                  <span class="recycle-remaining" :class="{ urgent: item.remaining_hours < 48 }">剩余 {{ Math.floor((item.remaining_hours || 0) / 24) }} 天 {{ (item.remaining_hours || 0) % 24 }} 小时</span>
                </div>
              </div>
              <button class="mobile-btn" @click="restoreItem(item.id)">恢复</button>
            </div>
          </div>
        </div>
        <div class="mobile-dialog-actions">
          <button class="mobile-dialog-btn cancel" @click="closeRecycle">关闭</button>
        </div>
      </div>
    </div>
    </Transition>

    <!-- 图片查看器 -->
    <Transition name="mobile-fade">
    <div v-if="viewerVisible" class="mobile-dialog-overlay mfb-viewer" @click.self="viewerVisible = false">
      <button class="mfb-viewer-close" @click="viewerVisible = false">×</button>
      <button v-if="viewerList.length > 1" class="mfb-viewer-nav prev" @click="viewerPrev">‹</button>
      <img v-if="viewerList[viewerIndex]" :src="viewerList[viewerIndex]" class="mfb-viewer-img" />
      <button v-if="viewerList.length > 1" class="mfb-viewer-nav next" @click="viewerNext">›</button>
      <div v-if="viewerList.length > 1" class="mfb-viewer-counter">{{ viewerIndex + 1 }} / {{ viewerList.length }}</div>
    </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref, computed } from 'vue'
import { adminApi, showToast, getAdminUser } from '@/api/client'
import { mobileConfirm, mobileActionMenu } from '@/utils/mobileDialog'
import './MobilePage.css'

// 当前登录管理员用户名（用于判断反馈是否由本人认领）
const currentAdminName = getAdminUser()?.username || ''
function isMineFeedback(f: any): boolean {
  return !!f.assignee && f.assignee === currentAdminName
}

const loading = ref(false)
const status = ref('')
const type = ref('')
const sortMode = ref('post_time_desc')
const list = ref<any[]>([])
const limitInput = ref(20)
const limitSaving = ref(false)

const sortOptions = [
  { key: 'post_time_desc', label: '最新提交' },
  { key: 'post_time_asc', label: '最早提交' },
  { key: 'update_desc', label: '最近更新' },
]
const sortLabel = computed(() => sortOptions.find(o => o.key === sortMode.value)?.label || '排序')

async function openSortMenu() {
  const key = await mobileActionMenu('排序方式', sortOptions.map(o => ({ key: o.key, label: o.label })))
  if (key) {
    sortMode.value = key
    loadList()
  }
}

const statusMap: Record<string, string> = { pending: '待处理', processing: '处理中', resolved: '已解决', rejected: '已拒绝' }
const statusLabel = (s: string) => statusMap[s] || s

const stats = ref({ total: 0, pending: 0, processing: 0, resolved: 0, rejected: 0 })

const filteredList = computed(() => {
  let arr = list.value
  if (status.value !== '') arr = arr.filter(f => f.status === status.value)
  if (type.value === 'appeal') arr = arr.filter(f => f.category === 'appeal')
  else if (type.value !== '') arr = arr.filter(f => f.feedback_type === type.value && f.category !== 'appeal')
  return arr
})

function setStatus(s: string) {
  status.value = s
  loadList()
}

// ===== 图片处理 =====
function normalizeImgUrl(u: string): string {
  if (u.startsWith('http://') || u.startsWith('https://')) {
    try {
      const parsed = new URL(u, window.location.origin)
      if (parsed.origin !== window.location.origin) {
        return window.location.origin + parsed.pathname + parsed.search
      }
      return u
    } catch { return u }
  }
  return u
}
function imagesOf(f: any): string[] {
  if (!f.images) return []
  try {
    const arr = JSON.parse(f.images)
    return Array.isArray(arr) ? arr.filter((u: string) => typeof u === 'string' && (u.startsWith('http') || u.startsWith('/'))).map(normalizeImgUrl) : []
  } catch { return [] }
}
function stackThumbStyle(i: number, total: number): Record<string, string> {
  if (total <= 1) return {}
  const offset = Math.min(i, 3) * 5
  return { left: `${offset}px`, top: `${offset}px`, zIndex: String(total - i) }
}

// ===== 日志 =====
const logVisible = ref(false)
const logTarget = ref<any>(null)
const logLoading = ref(false)
const activeLogTab = ref<'error' | 'all'>('error')
const currentLogText = computed(() => (logTarget.value ? (activeLogTab.value === 'error' ? (logTarget.value.error_logs || '') : (logTarget.value.all_logs || '')) : ''))
function formatLogSize(chars?: number | string): string {
  const n = Number(chars || 0)
  if (n <= 0) return ''
  if (n < 1024) return `${n} 字`
  return `${(n / 1024).toFixed(1)}K 字`
}
function truthyFlag(value: unknown): boolean { return value === true || value === 1 || value === '1' }
function hasErrorLogs(item: any): boolean { return truthyFlag(item.has_error_logs) || !!item.error_logs }
function hasAllLogs(item: any): boolean { return truthyFlag(item.has_all_logs) || !!item.all_logs }
async function openLogModal(item: any) {
  logVisible.value = true
  logTarget.value = item
  activeLogTab.value = hasErrorLogs(item) ? 'error' : 'all'
  logLoading.value = true
  const res = await adminApi<any>('get_feedback_detail', { id: item.id })
  logLoading.value = false
  if (res.code === 200 && res.data) {
    logTarget.value = res.data
    activeLogTab.value = res.data.error_logs ? 'error' : 'all'
  } else { showToast(res.msg || '日志加载失败') }
}
function closeLog() { if (!logLoading.value) { logVisible.value = false; logTarget.value = null } }

// ===== 时间戳 =====
function fmtTime(v: any): string {
  if (!v) return ''
  const m = String(v).match(/(\d{4})[-/](\d{1,2})[-/](\d{1,2})[T ]+(\d{1,2}):(\d{1,2})/)
  if (!m) return String(v)
  const [, y, mo, d, h, mi] = m.map(Number)
  return `${y}年${mo}月${d}日 ${h}时${String(mi).padStart(2, '0')}分`
}

async function loadList() {
  loading.value = true
  const res = await adminApi<any>('list_feedback', { status_filter: status.value === '' ? '' : status.value, sort: sortMode.value })
  if (res.code === 200 && res.data) {
    list.value = res.data.list || []
    if (res.data.stats) stats.value = res.data.stats
  } else { list.value = [] }
  loading.value = false
}

// ===== 限制 =====
async function loadLimit() {
  const res = await adminApi<any>('get_feedback_limit')
  if (res.code === 200 && res.data) limitInput.value = Number(res.data.feedback_daily_limit ?? 20)
}
async function saveLimit() {
  const v = Number(limitInput.value)
  if (!Number.isInteger(v) || v < 0 || v > 10000) return showToast('上限需为 0 到 10000 的整数')
  limitSaving.value = true
  const res = await adminApi('update_feedback_limit', { feedback_daily_limit: v })
  limitSaving.value = false
  if (res.code === 200) showToast('限制已保存', 'success'); else showToast(res.msg || '保存失败')
}

// ===== 认领 =====
async function claim(f: any) {
  const isTransfer = f.status === 'processing'
  const ok = await mobileConfirm(isTransfer ? '确认将该反馈转认领到自己名下？认领后问题将转移到您的名下。' : '确认认领该反馈？认领后将自动划入您的名下并移入处理中。', { title: '认领反馈', confirmText: '认领' })
  if (!ok) return
  const res = await adminApi('claim_feedback', { id: f.id })
  if (res.code === 200) {
    const old = f.status
    f.status = 'processing'
    f.assignee = res.data?.assignee || ''
    if (old !== 'processing') {
      if (stats.value[old as keyof typeof stats.value] !== undefined) (stats.value as any)[old]--
      stats.value.processing++
    }
    showToast(isTransfer ? '已转认领到自己名下' : '认领成功，已置为处理中', 'success')
  } else { showToast(res.msg || '认领失败') }
}

// ===== 状态变更 =====
async function setStatusOf(f: any, s: string) {
  const ok = await mobileConfirm(s === 'rejected' ? '确认拒绝此反馈？' : '确认更新状态？', { title: '更新反馈状态', confirmText: '确认', danger: s === 'rejected' })
  if (!ok) return
  const res = await adminApi('update_feedback_status', { id: f.id, status: s })
  if (res.code === 200) {
    const old = f.status
    f.status = s
    if (stats.value[old as keyof typeof stats.value] !== undefined) (stats.value as any)[old]--
    if (stats.value[s as keyof typeof stats.value] !== undefined) (stats.value as any)[s]++
    showToast('状态已更新', 'success')
  } else { showToast(res.msg || '操作失败') }
}

// ===== 完成弹窗 =====
const resolveVisible = ref(false)
const resolveTarget = ref<any>(null)
const resolveNote = ref('')
const resolveSaving = ref(false)
function openResolve(f: any) { resolveTarget.value = f; resolveNote.value = ''; resolveSaving.value = false; resolveVisible.value = true }
function closeResolve() { if (!resolveSaving.value) { resolveVisible.value = false; resolveTarget.value = null } }
async function confirmResolve() {
  if (!resolveTarget.value || !resolveNote.value.trim()) return
  resolveSaving.value = true
  const res = await adminApi('resolve_feedback', { id: resolveTarget.value.id, note: resolveNote.value.trim() })
  resolveSaving.value = false
  if (res.code === 200) {
    resolveTarget.value.status = 'resolved'
    resolveTarget.value.resolve_note = resolveNote.value.trim()
    closeResolve()
    showToast('已标记为已完成', 'success')
  } else { showToast(res.msg || '操作失败') }
}

// ===== 批量管理 =====
const batchMode = ref(false)
const selectedIds = ref<Set<number>>(new Set())
const allSelected = computed(() => filteredList.value.length > 0 && filteredList.value.every(f => selectedIds.value.has(f.id)))
function enterBatchMode() { batchMode.value = true; selectedIds.value.clear() }
function exitBatchMode() { batchMode.value = false; selectedIds.value.clear() }
function toggleSelect(id: number) {
  if (selectedIds.value.has(id)) selectedIds.value.delete(id)
  else selectedIds.value.add(id)
  selectedIds.value = new Set(selectedIds.value)
}
function toggleSelectAll() {
  filteredList.value.forEach(f => {
    if (allSelected.value) selectedIds.value.delete(f.id)
    else selectedIds.value.add(f.id)
  })
  selectedIds.value = new Set(selectedIds.value)
}
async function confirmBatchDelete() {
  if (selectedIds.value.size === 0) return
  const ok = await mobileConfirm(`确认将选中的 ${selectedIds.value.size} 条记录移入回收站？14 天内可恢复。`, { title: '批量删除', confirmText: '删除', danger: true })
  if (!ok) return
  const ids = Array.from(selectedIds.value)
  const res = await adminApi('batch_delete_feedback', { ids })
  if (res.code === 200) {
    showToast(`已删除 ${res.data?.deleted ?? ids.length} 条记录`, 'success')
    exitBatchMode()
    await loadList()
  } else { showToast(res.msg || '删除失败') }
}

// ===== 回收站 =====
const recycleVisible = ref(false)
const recycleLoading = ref(false)
const recycleList = ref<any[]>([])
async function openRecycleBin() {
  recycleVisible.value = true
  await loadRecycleBin()
}
function closeRecycle() { if (!recycleLoading.value) recycleVisible.value = false }
async function loadRecycleBin() {
  recycleLoading.value = true
  const res = await adminApi<any>('list_recycle_bin')
  recycleLoading.value = false
  if (res.code === 200 && res.data) recycleList.value = res.data.list || []
  else { recycleList.value = []; showToast(res.msg || '回收站加载失败') }
}
async function restoreItem(id: number) {
  const res = await adminApi('restore_feedback', { id })
  if (res.code === 200) {
    showToast('恢复成功', 'success')
    recycleList.value = recycleList.value.filter(r => r.id !== id)
    await loadList()
  } else { showToast(res.msg || '恢复失败') }
}

// ===== 新建事项 =====
const createVisible = ref(false)
const createType = ref<'problem' | 'suggestion'>('problem')
const createContent = ref('')
const createImages = ref<string[]>([])
const createNotify = ref(false)
const createDragging = ref(false)
const createSaving = ref(false)
const fileInput = ref<HTMLInputElement | null>(null)
function openCreate() {
  createType.value = 'problem'; createContent.value = ''
  createImages.value = []; createNotify.value = false; createDragging.value = false; createSaving.value = false
  createVisible.value = true
}
function closeCreate() { if (!createSaving.value) createVisible.value = false }
function onFileChange(e: Event) {
  const input = e.target as HTMLInputElement
  if (input.files) handleFiles(Array.from(input.files))
  input.value = ''
}
function onDrop(e: DragEvent) {
  e.preventDefault(); createDragging.value = false
  const files = e.dataTransfer?.files
  if (files) handleFiles(Array.from(files))
}
function handleFiles(files: File[]) {
  const remaining = 6 - createImages.value.length
  if (remaining <= 0) return showToast('最多上传 6 张图片')
  for (const file of files.slice(0, remaining)) {
    if (!file.type.startsWith('image/')) continue
    if (file.size > 8 * 1024 * 1024) { showToast(`图片 ${file.name} 超过 8MB，已跳过`); continue }
    const reader = new FileReader()
    reader.onload = () => createImages.value.push(reader.result as string)
    reader.readAsDataURL(file)
  }
}
async function submitCreate() {
  if (!createContent.value.trim()) return showToast('请填写内容')
  createSaving.value = true
  const res = await adminApi('create_feedback', {
    feedback_type: createType.value,
    title: createType.value === 'suggestion' ? '功能建议' : '问题反馈',
    content: createContent.value.trim(),
    images: createImages.value,
    notify_external: createNotify.value ? 1 : 0,
  })
  createSaving.value = false
  if (res.code === 200) { showToast('创建成功', 'success'); closeCreate(); loadList() } else { showToast(res.msg || '创建失败') }
}

// ===== 统计 =====
const statsVisible = ref(false)
const statsLoading = ref(false)
const statsList = ref<any[]>([])
const statsGrand = ref(0)
async function openStats() {
  statsVisible.value = true
  statsLoading.value = true
  const res = await adminApi<any>('feedback_admin_stats')
  statsLoading.value = false
  if (res.code === 200 && res.data) {
    statsList.value = res.data.list || []
    statsGrand.value = Number(res.data.grand_total ?? 0)
  } else { statsList.value = []; statsGrand.value = 0 }
}

// ===== 图片查看器 =====
const viewerVisible = ref(false)
const viewerList = ref<string[]>([])
const viewerIndex = ref(0)
function openViewer(imgs: string[], i: number) { viewerList.value = imgs; viewerIndex.value = i; viewerVisible.value = true }
function viewerPrev() { viewerIndex.value = (viewerIndex.value - 1 + viewerList.value.length) % viewerList.value.length }
function viewerNext() { viewerIndex.value = (viewerIndex.value + 1) % viewerList.value.length }

onMounted(() => { loadLimit(); loadList() })
</script>

<style scoped>
/* 页面头部 */
.mobile-page-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
}
.mobile-head-title {
  font-size: 18px;
  font-weight: 850;
  color: var(--text);
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.pending-badge {
  font-size: 11px;
  font-weight: 850;
  padding: 2px 9px;
  border-radius: 999px;
  background: rgba(236, 65, 65, 0.10);
  color: #EC4141;
}
.mobile-head-desc { margin-top: 4px; font-size: 12px; color: var(--text-muted); line-height: 1.5; }
.mobile-head-actions { display: flex; gap: 8px; flex: 0 0 auto; }

/* 限制面板 */
.limit-panel { display: flex; flex-direction: column; gap: 12px; }
.limit-info { display: flex; align-items: center; gap: 12px; }
.limit-icon {
  width: 40px; height: 40px; flex-shrink: 0;
  border-radius: 12px;
  display: flex; align-items: center; justify-content: center;
  background: rgba(236, 65, 65, 0.08); color: #EC4141;
}
.limit-text h3 { margin: 0; font-size: 14px; font-weight: 850; }
.limit-text p { margin: 3px 0 0; font-size: 12px; color: var(--text-muted); line-height: 1.5; }
.limit-actions { display: flex; gap: 8px; align-items: center; }
.limit-actions .mobile-input { flex: 1; min-width: 0; }

/* 统计卡片 */
.stats-row {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 8px;
}
.stat-chip {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 3px;
  padding: 10px 4px;
  border: 1px solid var(--border);
  border-radius: 16px;
  background: var(--card);
  cursor: pointer;
  transition: border-color 0.16s, transform 0.16s, box-shadow 0.16s;
}
.stat-chip:active { transform: scale(0.95); }
.stat-chip.active { border-color: #EC4141; box-shadow: 0 0 0 3px rgba(236, 65, 65, 0.10); }
.stat-icon {
  width: 28px; height: 28px;
  border-radius: 9px;
  display: flex; align-items: center; justify-content: center;
}
.stat-icon-total { background: rgba(59, 130, 246, 0.10); color: #3b82f6; }
.stat-icon-pending { background: rgba(245, 158, 11, 0.12); color: #f59e0b; }
.stat-icon-processing { background: rgba(139, 92, 246, 0.12); color: #8b5cf6; }
.stat-icon-resolved { background: rgba(34, 197, 94, 0.12); color: #16a34a; }
.stat-icon-rejected { background: rgba(236, 65, 65, 0.10); color: #EC4141; }
.stat-num { font-size: 16px; font-weight: 850; line-height: 1; }
.stat-label { font-size: 10px; color: var(--text-muted); }

/* 工具条 */
.toolbar { display: flex; flex-direction: column; gap: 8px; }
.toolbar-group { display: flex; gap: 6px; justify-content: center; }
.tool-btn {
  flex: 0 0 auto;
  border: 1px solid var(--border);
  border-radius: 999px;
  padding: 6px 11px;
  background: var(--control-bg);
  color: var(--text-light);
  font-size: 12px;
  font-weight: 800;
  transition: background 0.16s, color 0.16s, border-color 0.16s;
}
.tool-btn.active { border-color: #EC4141; background: #EC4141; color: #fff; }
.toolbar-right { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
.sort-btn {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 7px 12px;
  height: 38px;
  border: 1px solid var(--border);
  border-radius: 12px;
  background: var(--control-bg);
  color: var(--text-light);
  cursor: pointer;
  font-size: 12px;
  font-weight: 700;
  white-space: nowrap;
  flex-shrink: 0;
  transition: transform 0.16s var(--motion), color 0.16s, background 0.16s, border-color 0.16s;
}
.sort-btn:active {
  transform: scale(0.94);
  color: var(--accent);
  background: var(--accent-soft);
  border-color: var(--accent);
}
.sort-btn svg { flex-shrink: 0; }
.batch-slot {
  position: relative;
  min-height: 38px;
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: flex-end;
}
.batch-abs {
  position: absolute;
  right: 0;
  top: 50%;
  transform: translateY(-50%);
}
.batch-bar { display: flex; align-items: center; gap: 8px; justify-content: flex-end; white-space: nowrap; }
.batch-bar .mobile-btn { flex: 0 0 auto; }
/* 批量菜单向左弹出/收回动画 */
.batch-slide-enter-active,
.batch-slide-leave-active {
  transition: opacity 0.22s cubic-bezier(0.16, 1, 0.3, 1),
              transform 0.22s cubic-bezier(0.16, 1, 0.3, 1);
}
.batch-slide-enter-from {
  opacity: 0;
  transform: translateY(-50%) translateX(30px);
}
.batch-slide-leave-to {
  opacity: 0;
  transform: translateY(-50%) translateX(30px);
}
.batch-select-all { display: flex; align-items: center; gap: 5px; font-size: 12px; color: var(--text-light); flex-shrink: 0; }
.batch-count { font-size: 12px; color: var(--text-muted); flex-shrink: 0; }

/* 反馈卡片 */
.mfb-item { padding: 14px; }
.mfb-item.batch-selected { border-color: #EC4141; }
.card-checkbox { margin-bottom: 8px; }
.card-checkbox input { width: 16px; height: 16px; }
.mfb-head { display: flex; align-items: flex-start; justify-content: space-between; gap: 8px; }
.mfb-user { display: flex; align-items: center; gap: 8px; min-width: 0; }
.user-avatar {
  width: 34px; height: 34px; flex-shrink: 0;
  border-radius: 11px;
  display: flex; align-items: center; justify-content: center;
}
.user-avatar.avatar-img { object-fit: cover; cursor: zoom-in; }
.avatar-problem { background: rgba(236, 65, 65, 0.10); color: #EC4141; }
.avatar-suggestion { background: rgba(79, 70, 229, 0.10); color: #4f46e5; }
.avatar-appeal { background: rgba(245, 158, 11, 0.12); color: #f59e0b; }
.user-info { display: flex; flex-direction: column; min-width: 0; }
.user-name { font-size: 13px; font-weight: 850; color: var(--text); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.user-id { font-size: 11px; color: var(--text-muted); }
.mfb-head-right { display: flex; align-items: center; gap: 5px; flex: 0 0 auto; }
.type-badge {
  font-size: 10px; font-weight: 850;
  padding: 2px 7px; border-radius: 999px;
  white-space: nowrap;
}
.type-problem { background: #fef2f2; color: #dc2626; }
.type-suggestion { background: #eef2ff; color: #4f46e5; }
.type-appeal { background: #fffbeb; color: #d97706; }
.status-badge {
  display: inline-flex; align-items: center; gap: 3px;
  font-size: 10px; font-weight: 850;
  padding: 2px 7px; border-radius: 999px;
  white-space: nowrap;
}
.badge-pending { background: rgba(245, 158, 11, 0.12); color: #f59e0b; }
.badge-processing { background: rgba(59, 130, 246, 0.10); color: #3b82f6; }
.badge-resolved { background: rgba(34, 197, 94, 0.12); color: #16a34a; }
.badge-rejected { background: rgba(236, 65, 65, 0.10); color: #EC4141; }

/* 主体 */
.mfb-main { display: flex; gap: 10px; align-items: flex-start; margin-top: 10px; }
.mfb-left { flex: 1; min-width: 0; }
.mfb-title { margin: 0; font-size: 14px; font-weight: 850; color: var(--text); word-break: break-word; }
.mfb-content {
  margin: 4px 0 0;
  font-size: 11px; color: var(--text-muted);
  line-height: 1.6; word-break: break-word;
  display: -webkit-box; -webkit-line-clamp: 3; -webkit-box-orient: vertical; overflow: hidden;
}
.detail-more { margin-top: 6px; display: flex; flex-wrap: wrap; gap: 4px 10px; }
.log-summary { display: flex; gap: 6px; flex-wrap: wrap; }
.log-chip {
  font-size: 10px; font-weight: 800;
  padding: 2px 8px; border-radius: 999px;
  background: var(--control-bg); color: var(--text-light);
  border: 1px solid var(--border);
}
.assignee-row { display: flex; align-items: center; gap: 4px; font-size: 11px; color: var(--text-muted); }
.resolve-note { display: flex; align-items: flex-start; gap: 4px; font-size: 11px; color: #16a34a; width: 100%; }
.resolve-text { display: flex; flex-direction: column; gap: 1px; min-width: 0; }
.resolve-label { font-weight: 800; }
.resolve-text span { word-break: break-word; }

/* 图片堆叠 */
.img-stack {
  position: relative;
  width: 76px; height: 76px;
  flex-shrink: 0;
  cursor: zoom-in;
}
.stack-thumb {
  position: absolute; width: 76px; height: 76px;
  object-fit: cover; border-radius: 12px;
  border: 1px solid var(--border);
  background: var(--control-bg);
}
.stack-count {
  position: absolute; top: 5px; right: 5px;
  min-width: 20px; height: 20px; padding: 0 5px;
  border-radius: 999px;
  background: rgba(15, 23, 42, 0.6); color: #fff;
  font-size: 11px; font-weight: 850;
  display: flex; align-items: center; justify-content: center;
  z-index: 20;
}

/* 卡片底部 */
.mfb-foot { margin-top: 10px; padding-top: 10px; border-top: 1px dashed var(--border); }
.foot-meta { font-size: 11px; color: var(--text-muted); line-height: 1.6; }
.foot-actions { display: flex; flex-wrap: wrap; gap: 6px; margin-top: 8px; }
.act-btn {
  border: 1px solid var(--border);
  border-radius: 999px;
  padding: 5px 11px;
  background: var(--control-bg);
  color: var(--text-light);
  font-size: 11px; font-weight: 800;
  display: inline-flex; align-items: center; gap: 4px;
}
.act-btn:active { transform: scale(0.95); }
.act-claim { color: #3b82f6; border-color: rgba(59, 130, 246, 0.3); }
.act-resolve { color: #16a34a; border-color: rgba(34, 197, 94, 0.3); }
.act-reject { color: #EC4141; border-color: rgba(236, 65, 65, 0.3); }

/* 弹窗通用补充 */
.resolve-target-info {
  display: flex; flex-direction: column; gap: 2px;
  margin: 10px 20px 0; padding: 8px 12px;
  border-radius: 12px; background: var(--control-bg);
  font-size: 12px;
}
.resolve-target-info strong { color: var(--text); }
.resolve-target-info span { color: var(--text-muted); font-size: 11px; }
.log-tabs { display: flex; gap: 6px; padding: 12px 20px 0; }
.log-tab {
  border: 1px solid var(--border); border-radius: 999px;
  padding: 5px 12px; background: var(--control-bg);
  color: var(--text-light); font-size: 12px; font-weight: 800;
}
.log-tab.active { border-color: #EC4141; background: #EC4141; color: #fff; }
.log-tab:disabled { opacity: 0.4; }
.log-body { padding: 10px 20px 14px; overflow-y: auto; }
.log-content {
  margin: 0; padding: 12px; border-radius: 12px;
  background: var(--control-bg); color: var(--text-light);
  font-size: 11px; line-height: 1.6; white-space: pre;
  max-height: 50vh; overflow: auto;
}
.recycle-body { padding: 10px 20px 14px; overflow-y: auto; display: flex; flex-direction: column; gap: 10px; }
.recycle-tip {
  padding: 10px 12px; border-radius: 12px;
  background: rgba(245, 158, 11, 0.08); color: #d97706;
  font-size: 12px; line-height: 1.5;
}
.recycle-list { display: flex; flex-direction: column; gap: 8px; }
.recycle-item {
  display: flex; align-items: center; justify-content: space-between; gap: 10px;
  padding: 10px 12px; border: 1px solid var(--border); border-radius: 12px;
}
.recycle-item-main { min-width: 0; }
.recycle-item-title { font-size: 13px; font-weight: 850; color: var(--text); word-break: break-word; }
.recycle-item-meta { display: flex; flex-wrap: wrap; gap: 4px 10px; font-size: 11px; color: var(--text-muted); margin-top: 4px; align-items: center; }
.recycle-remaining { color: #d97706; }
.recycle-remaining.urgent { color: #EC4141; font-weight: 800; }
.type-badge.small { font-size: 9px; padding: 1px 6px; }

/* 新建弹窗 */
.mfb-create-body { padding: 10px 20px 12px; overflow-y: auto; display: flex; flex-direction: column; gap: 10px; }
.mfb-create-body .mobile-dialog-input {
  width: 100%;
  margin: 0;
  box-sizing: border-box;
}
.mfb-type-row { display: flex; gap: 8px; }
.mfb-type-btn {
  flex: 1;
  border: 1px solid var(--border);
  border-radius: 999px;
  padding: 8px 0;
  background: var(--control-bg);
  color: var(--text-light);
  font-size: 12px; font-weight: 800;
}
.mfb-type-btn.active { border-color: #EC4141; background: #EC4141; color: #fff; }
.mfb-dropzone {
  display: flex; align-items: center; justify-content: center;
  padding: 16px; border: 1.5px dashed var(--border);
  border-radius: 14px; text-align: center; cursor: pointer;
  transition: border-color 0.2s, background 0.2s;
}
.mfb-dropzone.dragging, .mfb-dropzone.has { border-color: #EC4141; background: rgba(236, 65, 65, 0.05); }
.mfb-dropzone-text { display: flex; flex-direction: column; gap: 2px; }
.mfb-dropzone-text strong { font-size: 13px; color: var(--text); }
.mfb-dropzone-text span { font-size: 11px; color: var(--text-muted); }
.mfb-preview { display: flex; flex-wrap: wrap; gap: 8px; }
.mfb-preview-item { position: relative; }
.mfb-preview-img { width: 72px; height: 72px; object-fit: cover; border-radius: 10px; border: 1px solid var(--border); }
.mfb-preview-remove {
  position: absolute; top: -6px; right: -6px;
  width: 20px; height: 20px; border: none; border-radius: 50%;
  background: #dc2626; color: #fff; font-size: 14px; line-height: 1;
  display: flex; align-items: center; justify-content: center; cursor: pointer;
}
.mfb-notify { display: flex; align-items: flex-start; gap: 10px; padding: 10px 12px; border-radius: 12px; background: var(--control-bg); cursor: pointer; }
.mfb-notify input { display: none; }
.mfb-notify-box {
  width: 20px; height: 20px; border-radius: 6px;
  border: 1.5px solid var(--border); background: var(--card); color: transparent;
  display: flex; align-items: center; justify-content: center;
  font-size: 13px; font-weight: 900; flex-shrink: 0; margin-top: 1px;
}
.mfb-notify-box.checked { background: #EC4141; border-color: #EC4141; color: #fff; }
.mfb-notify-text { display: flex; flex-direction: column; gap: 2px; }
.mfb-notify-text strong { font-size: 13px; color: var(--text); }
.mfb-notify-text span { font-size: 11px; color: var(--text-muted); line-height: 1.5; }

/* 统计弹窗 */
.mfb-stats-body { padding: 10px 20px 12px; overflow-y: auto; display: flex; flex-direction: column; gap: 10px; }
.mfb-stats-total { display: flex; flex-direction: column; align-items: center; gap: 3px; padding: 14px; border-radius: 14px; background: var(--control-bg); }
.mfb-stats-total strong { font-size: 30px; line-height: 1; }
.mfb-stats-total span { font-size: 12px; color: var(--text-muted); }
.mfb-stats-list { display: flex; flex-direction: column; gap: 8px; }
.mfb-stats-row { display: flex; flex-direction: column; gap: 6px; padding: 10px 12px; border: 1px solid var(--border); border-radius: 12px; }
.mfb-stats-admin { display: flex; align-items: center; gap: 8px; font-size: 13px; font-weight: 850; }
.mfb-dot { width: 10px; height: 10px; border-radius: 50%; }
.dot-unclaimed { background: #a1a1aa; }
.dot-0 { background: #3b82f6; }
.dot-1 { background: #16a34a; }
.dot-2 { background: #f59e0b; }
.dot-3 { background: #8b5cf6; }
.mfb-stats-nums { display: flex; flex-wrap: wrap; gap: 6px 12px; }
.mfb-stats-nums span { display: inline-flex; align-items: center; gap: 4px; font-size: 12px; color: var(--text-muted); }
.mfb-stats-nums b { font-size: 13px; }
.mfb-stats-nums i { width: 8px; height: 8px; border-radius: 50%; display: inline-block; }
.dot-pending { background: #f59e0b; }
.dot-processing { background: #3b82f6; }
.dot-resolved { background: #16a34a; }
.dot-rejected { background: #dc2626; }
.c-processing { color: #3b82f6; }
.c-resolved { color: #16a34a; }
.c-rejected { color: #dc2626; }
.c-pending { color: #f59e0b; }

/* 图片查看器 */
.mfb-viewer { background: rgba(0, 0, 0, 0.9) !important; padding: 0; }
.mfb-viewer-img { max-width: 92vw; max-height: 88vh; object-fit: contain; border-radius: 8px; }
.mfb-viewer-close {
  position: absolute; top: 18px; right: 18px;
  width: 40px; height: 40px; border: none; border-radius: 50%;
  background: rgba(255, 255, 255, 0.15); color: #fff;
  font-size: 22px; line-height: 1;
  display: flex; align-items: center; justify-content: center; cursor: pointer;
}
.mfb-viewer-nav {
  position: absolute; top: 50%; transform: translateY(-50%);
  width: 42px; height: 42px; border: none; border-radius: 50%;
  background: rgba(255, 255, 255, 0.15); color: #fff;
  font-size: 26px; line-height: 1;
  display: flex; align-items: center; justify-content: center; cursor: pointer;
}
.mfb-viewer-nav.prev { left: 14px; }
.mfb-viewer-nav.next { right: 14px; }
.mfb-viewer-counter {
  position: absolute; bottom: 20px; left: 50%; transform: translateX(-50%);
  padding: 5px 14px; border-radius: 999px;
  background: rgba(255, 255, 255, 0.16); color: #fff; font-size: 13px; font-weight: 800;
}
</style>