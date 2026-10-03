<template>
  <div class="users-wrap">
    <Transition name="fade-down" appear>
    <div class="toolbar-row">
      <div class="search-box">
        <input
          v-model="keyword"
          type="text"
          placeholder="搜索昵称、弦予号或邮箱"
          @keyup.enter="handleSearch"
        />
        <button class="btn btn-primary" @click="handleSearch">搜索</button>
        <button v-if="keyword" class="btn" @click="clearSearch">清除</button>
      </div>

      <div class="toolbar-actions">
        <button class="btn btn-primary" @click="openAddModal">+ 添加用户</button>
        <div class="batch-slot">
          <Transition name="batch-slide">
            <div v-if="isBatchMode" key="batch" class="batch-bar">
              <button class="batch-act" @click="toggleSelectAll">
                <span class="checkbox-badge" :class="{ checked: isAllSelected }">
                  <svg v-if="isAllSelected" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
                </span>
                {{ isAllSelected ? '取消全选' : '全选' }}
              </button>
              <button class="batch-act" @click="batchToggleSelected(0)" :disabled="selectedCount === 0 || batchLoading">封禁</button>
              <button class="batch-act" @click="batchToggleSelected(1)" :disabled="selectedCount === 0 || batchLoading">启用</button>
              <button class="batch-act" @click="batchBanDevice" :disabled="selectedCount === 0 || batchLoading">封禁ID</button>
              <button class="batch-act" @click="batchResetDuration" :disabled="selectedCount === 0 || batchLoading">重置时长</button>
              <button class="batch-act batch-act--danger" @click="batchDeleteSelected" :disabled="selectedCount === 0 || batchLoading">删除</button>
              <button class="batch-act" @click="deleteEmptyPlaylists" :disabled="batchLoading">清空歌单</button>
              <span class="batch-count">已选 {{ selectedCount }} 项</span>
              <button class="batch-act batch-act--primary" @click="exitBatchMode">完成</button>
            </div>
            <button v-else key="enter" class="btn-batch-enter" @click="enterBatchMode">批量管理</button>
          </Transition>
        </div>
      </div>
    </div>
    </Transition>

    <Transition name="fade-up" appear>
      <div class="stats-row">
        <div class="stat-chip">
          <div class="stat-icon stat-icon-total">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="4" width="18" height="16" rx="2"/><circle cx="9" cy="10" r="2"/><path d="M3 20c1.4-2.6 3.8-4 6-4s4.6 1.4 6 4"/></svg>
          </div>
          <div class="stat-body"><span class="stat-num">{{ stats.total }}</span><span class="stat-label">用户总数</span></div>
        </div>
        <div class="stat-chip">
          <div class="stat-icon stat-icon-active">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
          </div>
          <div class="stat-body"><span class="stat-num">{{ stats.normal }}</span><span class="stat-label">正常</span></div>
        </div>
        <div class="stat-chip">
          <div class="stat-icon stat-icon-banned">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="4.93" y1="4.93" x2="19.07" y2="19.07"/></svg>
          </div>
          <div class="stat-body"><span class="stat-num">{{ stats.banned }}</span><span class="stat-label">被封禁</span></div>
        </div>
      </div>
    </Transition>

    <!-- 用户表格 -->
    <Transition name="fade-up" appear>
    <div class="card">
      <div v-if="loading" class="empty">加载中...</div>
      <div v-else-if="loadError" class="empty">{{ loadError }}</div>
      <div v-else-if="users.length === 0" class="empty">暂无用户数据</div>
      <div v-else class="table-wrapper">
        <table>
          <thead>
            <tr>
              <th v-if="isBatchMode" class="col-check">
                <span class="checkbox-badge" :class="{ checked: isAllSelected }" @click="toggleSelectAll">
                  <svg v-if="isAllSelected" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
                </span>
              </th>
              <th>弦予号</th>
              <th>头像</th>
              <th>昵称</th>
              <th>邮箱</th>
              <th>邮箱验证</th>
              <th>状态</th>
              <th>听歌时长</th>
              <th>注册时间</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(u, idx) in users" :key="u.id" class="table-row" :class="{ 'row-selected': isBatchMode && selectedIds.has(u.id) }" :style="{ animationDelay: `${idx * 40}ms` }">
              <td v-if="isBatchMode" class="col-check">
                <span class="checkbox-badge" :class="{ checked: selectedIds.has(u.id) }" @click="toggleSelect(u.id)">
                  <svg v-if="selectedIds.has(u.id)" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
                </span>
              </td>
              <td>{{ u.ciyuanxi_id || '-' }}</td>
              <td>
                <img
                  v-if="u.avatar_url"
                  :src="u.avatar_url"
                  alt="头像"
                  class="user-avatar"
                  @click="openAvatarView(u)"
                />
                <div v-else class="avatar-placeholder">{{ (u.nickname || u.username || '?').charAt(0) }}</div>
              </td>
              <td>{{ u.nickname || u.username }}</td>
              <td>{{ u.email || '-' }}</td>
              <td>
                <span :class="['badge', u.email_verified == 1 ? 'badge-success' : 'badge-warning']">
                  {{ u.email_verified == 1 ? '已验证' : '未验证' }}
                </span>
              </td>
              <td>
                <span :class="['badge', u.status != 0 ? 'badge-success' : 'badge-error']">
                  {{ u.status != 0 ? '正常' : '禁用' }}
                </span>
                <div v-if="u.status == 0 && u.ban_reason" style="margin-top:4px;font-size:11px;color:#e74c3c;max-width:160px;word-break:break-all;">
                  原因：{{ u.ban_reason }}
                </div>
              </td>
              <td>{{ formatDuration(u.listen_duration) }}</td>
              <td>{{ fmtDateTime(u.created_at) }}</td>
              <td>
                <div class="row-actions">
                  <button class="btn btn-sm btn-primary" @click="openRowMenu(u)">
                    操作
                  </button>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- 分页 -->
      <div v-if="!loading && total > 0" class="pagination">
        <button :disabled="page <= 1" @click="goPage(page - 1)">上一页</button>
        <button
          v-for="p in pageNumbers"
          :key="p"
          :class="{ active: p === page }"
          @click="goPage(p)"
        >{{ p }}</button>
        <button :disabled="page >= totalPages" @click="goPage(page + 1)">下一页</button>
        <span>共 {{ total }} 条</span>
      </div>
    </div>
    </Transition>

    <!-- 添加用户弹窗 -->
    <Transition name="modal">
    <div v-if="showAddModal" class="modal-overlay">
      <div class="modal">
        <h3>添加用户</h3>
        <div class="form-group">
          <label class="required">弦予号</label>
          <input v-model="addForm.username" type="text" placeholder="6-20 位，仅含字母或数字" autocomplete="off" />
          <div class="hint">用于登录，6-20 位，仅含字母或数字（支持大小写字母）</div>
        </div>
        <div class="form-group">
          <label class="required">密码</label>
          <input v-model="addForm.password" type="password" placeholder="至少 6 位" autocomplete="new-password" />
        </div>
        <div class="form-group">
          <label>昵称（选填）</label>
          <input v-model="addForm.nickname" type="text" placeholder='留空默认"弦予+弦予号"' autocomplete="off" />
        </div>
        <div class="form-group">
          <label>邮箱（选填）</label>
          <input v-model="addForm.email" type="email" placeholder="留空则不绑定" autocomplete="off" />
        </div>
        <div class="modal-actions">
          <button class="btn" @click="showAddModal = false">取消</button>
          <button class="btn btn-primary" @click="submitAddUser" :disabled="addLoading">
            {{ addLoading ? '添加中...' : '确定' }}
          </button>
        </div>
      </div>
    </div>
    </Transition>

    <!-- 修改昵称弹窗 -->
    <Transition name="modal">
    <div v-if="showNicknameModal" class="modal-overlay">
      <div class="modal">
        <h3>修改用户昵称</h3>
        <div class="form-group">
          <label>当前昵称</label>
          <input :value="nicknameForm.oldNickname" type="text" disabled />
        </div>
        <div class="form-group">
          <label>弦予号</label>
          <input :value="nicknameForm.ciyuanxiId || '-'" type="text" disabled />
        </div>
        <div class="form-group">
          <label class="required">新昵称</label>
          <input v-model="nicknameForm.newNickname" type="text" placeholder="2-32 位，支持字母、数字、汉字" maxlength="32" />
          <div class="hint">2-32 个字符，支持字母、数字、汉字组合</div>
        </div>
        <div class="form-group">
          <label class="required">修改原因</label>
          <textarea v-model="nicknameForm.reason" rows="3" placeholder="填写修改原因，将同步下发给客户端通知用户" maxlength="255"></textarea>
          <div class="hint">原因将作为回执下发给客户端，用户端会收到昵称修改通知</div>
        </div>
        <div class="modal-actions">
          <button class="btn" @click="showNicknameModal = false">取消</button>
          <button class="btn btn-primary" @click="submitNicknameChange" :disabled="nicknameLoading">
            {{ nicknameLoading ? '提交中...' : '确定修改' }}
          </button>
        </div>
      </div>
    </div>
    </Transition>

    <!-- 修改邮箱弹窗 -->
    <Transition name="modal">
    <div v-if="showEmailModal" class="modal-overlay">
      <div class="modal">
        <h3>修改用户邮箱</h3>
        <div class="form-group">
          <label>昵称</label>
          <input :value="emailForm.username" type="text" disabled />
        </div>
        <div class="form-group">
          <label>当前邮箱</label>
          <input :value="emailForm.currentEmail || '(无)'" type="text" disabled />
        </div>
        <div class="form-group">
          <label>新邮箱</label>
          <input v-model="emailForm.newEmail" type="email" placeholder="留空则清除邮箱" />
          <div class="hint">清除邮箱后该用户将视为普通成员</div>
        </div>
        <div class="modal-actions">
          <button class="btn" @click="showEmailModal = false">取消</button>
          <button class="btn btn-primary" @click="submitEmailChange" :disabled="emailLoading">
            {{ emailLoading ? '提交中...' : '确定' }}
          </button>
        </div>
      </div>
    </div>
    </Transition>

    <!-- 重置听歌时长确认弹窗 -->
    <Transition name="modal">
    <div v-if="showResetModal" class="modal-overlay">
      <div class="modal">
        <h3>重置听歌时长</h3>
        <div class="form-group">
          <label>昵称</label>
          <input :value="resetForm.username" type="text" disabled />
        </div>
        <div class="form-group">
          <label>当前听歌时长</label>
          <input :value="resetForm.duration" type="text" disabled />
        </div>
        <div class="form-group">
          <label>清除原因 <span style="color:#f59e0b">（必填，下发给用户）</span></label>
          <textarea v-model="resetForm.reason" rows="2" placeholder="请输入清除原因，客户端将弹窗通知用户" style="width:100%;box-sizing:border-box;padding:8px 10px;border:1px solid rgba(128,128,128,0.35);border-radius:6px;resize:vertical;" />
        </div>
        <div style="background:rgba(245,158,11,0.14);border:1px solid rgba(245,158,11,0.30);color:#f59e0b;padding:10px 14px;border-radius:6px;font-size:12px;">
          重置后听歌时长与新歌数将清零，此操作不可恢复。
        </div>
        <div class="modal-actions">
          <button class="btn" @click="showResetModal = false">取消</button>
          <button class="btn btn-danger" @click="submitReset" :disabled="resetLoading">
            {{ resetLoading ? '重置中...' : '确定重置' }}
          </button>
        </div>
      </div>
    </div>
    </Transition>

    <!-- 查看插件弹窗 -->
    <Transition name="modal">
    <div v-if="showPluginsModal" class="modal-overlay">
      <div class="modal" style="max-width:900px;">
        <div class="modal-head-bar">
          <h3>用户插件 - {{ pluginsData.nickname || pluginsData.username }}</h3>
          <button class="modal-close-btn" @click="showPluginsModal = false">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
          </button>
        </div>
        <div v-if="pluginsLoading" class="empty">加载中...</div>
        <div v-else>
          <div style="display:flex;gap:16px;margin-bottom:16px;font-size:13px;color:#666;">
            <span>插件数量: {{ pluginsData.plugin_count || 0 }}</span>
            <span v-if="pluginsData.uploaded_at">上传时间: {{ fmtDateTime(pluginsData.uploaded_at) }}</span>
            <span v-if="pluginsData.ciyuanxi_id">弦予号: {{ pluginsData.ciyuanxi_id }}</span>
          </div>
          <div v-if="pluginsData.plugins && pluginsData.plugins.length > 0" class="table-wrapper">
            <table>
              <thead>
                <tr><th>名称</th><th>格式</th><th>版本</th><th>作者</th><th>状态</th><th>大小</th></tr>
              </thead>
              <tbody>
                <tr v-for="(p, i) in pluginsData.plugins" :key="i">
                  <td>
                    {{ p.name }}
                    <div v-if="p.description" style="font-size:11px;color:#999;">{{ p.description }}</div>
                  </td>
                  <td><span class="badge badge-info">{{ p.format }}</span></td>
                  <td>{{ p.version || '-' }}</td>
                  <td>{{ p.author || '-' }}</td>
                  <td>
                    <span :class="['badge', p.enabled ? 'badge-success' : 'badge-error']">
                      {{ p.enabled ? '启用' : '禁用' }}
                    </span>
                  </td>
                  <td>{{ formatScriptSize(p.scriptSize) }}</td>
                </tr>
              </tbody>
            </table>
          </div>
          <div v-else class="empty">该用户暂无插件数据</div>
        </div>
      </div>
    </div>
    </Transition>

    <!-- 头像大图查看弹窗 -->
    <Transition name="modal">
    <div v-if="showAvatarModal" class="modal-overlay">
      <div class="modal" style="width:300px;text-align:center;">
        <h3>{{ avatarViewUser?.nickname || avatarViewUser?.username }} 的头像</h3>
        <img
          v-if="avatarViewUser?.avatar_url"
          :src="avatarViewUser.avatar_url"
          alt="头像"
          style="width:200px;height:200px;border-radius:50%;object-fit:cover;margin:16px auto;"
        />
        <div class="modal-actions">
          <button class="btn" @click="showAvatarModal = false">关闭</button>
          <button class="btn btn-danger" @click="confirmDeleteAvatar" :disabled="avatarDeleting">
            {{ avatarDeleting ? '删除中...' : '删除头像' }}
          </button>
        </div>
      </div>
    </div>
    </Transition>

    <!-- 设备信息弹窗 -->
    <Transition name="modal">
    <div v-if="showDeviceModal" class="modal-overlay">
      <div class="modal" style="max-width:900px;">
        <div class="modal-head-bar">
          <h3>设备信息 - {{ deviceData.nickname || deviceData.username }}</h3>
          <button class="modal-close-btn" @click="showDeviceModal = false">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
          </button>
        </div>
        <div v-if="deviceLoading" class="empty">加载中...</div>
        <div v-else>
          <div style="display:flex;gap:16px;margin-bottom:16px;font-size:13px;color:#666;flex-wrap:wrap;">
            <span>弦予号: {{ deviceData.ciyuanxi_id || '-' }}</span>
          </div>

          <!-- 该账号的全部设备 -->
          <div v-if="userDevices.length > 0" class="user-device-list">
            <div v-for="dv in userDevices" :key="dv.device_id" class="user-device-item" :class="{ 'is-banned': dv.is_banned }">
              <svg v-if="userDeviceIcon(dv) === 'mobile'" class="user-device-icon" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="5" y="2" width="14" height="20" rx="2"/><line x1="12" y1="18" x2="12.01" y2="18"/></svg>
              <svg v-else-if="userDeviceIcon(dv) === 'watch'" class="user-device-icon" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="6"/><polyline points="12 10 12 12 13 13"/><path d="M16.13 7.66l-.81-4.05a2 2 0 0 0-2-1.61h-2.68a2 2 0 0 0-2 1.61l-.78 4.05"/><path d="M7.88 16.36l.8 4a2 2 0 0 0 2 1.61h2.69a2 2 0 0 0 2-1.61l.81-4.05"/></svg>
              <svg v-else class="user-device-icon" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="4" width="20" height="14" rx="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/></svg>
              <div class="user-device-info">
                <div class="user-device-name-row">
                  <span class="user-device-model">{{ dv.device_model || '未知型号' }}</span>
                  <span class="platform-badge" :class="`platform-${userDeviceIcon(dv)}`">{{ userDevicePlatformLabel(dv) }}</span>
                  <span v-if="dv.is_last" class="user-device-current">当前设备</span>
                  <span v-if="dv.is_banned" class="user-device-banned-badge">已封禁</span>
                </div>
                <span class="user-device-id">{{ dv.device_id }}</span>
                <span class="user-device-meta">{{ userDeviceMeta(dv) }}</span>
              </div>
              <div class="user-device-actions">
                <button v-if="!dv.is_banned" class="btn btn-danger btn-sm" @click="banUserDevice(dv.device_id, deviceData.nickname || deviceData.username)">封禁</button>
                <button v-else class="btn btn-success btn-sm" @click="unbanUserDevice(dv.device_id)">解封</button>
              </div>
            </div>
          </div>
          <div v-if="userDevices.length === 0" class="empty">该用户暂无设备记录</div>

          <div v-if="deviceData.login_logs && deviceData.login_logs.length > 0" style="margin-bottom:16px;">
            <h4 style="font-size:13px;margin-bottom:8px;">登录记录</h4>
            <div class="table-wrapper">
              <table>
                <thead><tr><th>设备ID</th><th>IP</th><th>时间</th></tr></thead>
                <tbody>
                  <tr v-for="(log, i) in deviceData.login_logs" :key="'l'+i">
                    <td style="font-size:11px;">{{ log.device_id }}</td>
                    <td>{{ log.ip }}</td>
                    <td>{{ fmtDateTime(log.created_at) }}</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>

          <div v-if="deviceData.open_logs && deviceData.open_logs.length > 0">
            <h4 style="font-size:13px;margin-bottom:8px;">启动记录</h4>
            <div class="table-wrapper">
              <table>
                <thead><tr><th>设备ID</th><th>IP</th><th>版本</th><th>时间</th></tr></thead>
                <tbody>
                  <tr v-for="(log, i) in deviceData.open_logs" :key="'o'+i">
                    <td style="font-size:11px;">{{ log.device_id }}</td>
                    <td>{{ log.ip }}</td>
                    <td>{{ log.app_version }}</td>
                    <td>{{ fmtDateTime(log.created_at) }}</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        </div>
      </div>
    </div>
    </Transition>

    <!-- 设备管理弹窗 -->
    <Transition name="modal">
    <div v-if="showBannedModal" class="modal-overlay">
      <div class="modal" style="max-width:700px;">
        <h3>设备管理</h3>

        <!-- 手动添加封禁 -->
        <div class="ban-form">
          <div class="form-group ban-device-input">
            <input v-model="banDeviceInput" type="text" placeholder="输入设备ID" />
          </div>
          <div class="form-group ban-reason-input">
            <input v-model="banReasonInput" type="text" placeholder="封禁原因（必填）" />
          </div>
          <button class="btn btn-danger ban-submit" @click="manualBanDevice">封禁</button>
        </div>

        <div v-if="bannedLoading" class="empty">加载中...</div>
        <div v-else-if="bannedDevices.length === 0" class="empty">暂无封禁设备</div>
        <div v-else class="table-wrapper">
          <table>
            <thead>
              <tr><th>ID</th><th>设备ID</th><th>原因</th><th>操作人</th><th>封禁时间</th><th>操作</th></tr>
            </thead>
            <tbody>
              <tr v-for="d in bannedDevices" :key="d.id">
                <td>{{ d.id }}</td>
                <td style="font-size:11px;word-break:break-all;">{{ d.device_id }}</td>
                <td>{{ d.reason || '-' }}</td>
                <td>{{ d.banned_by }}</td>
                <td>{{ fmtDateTime(d.created_at) }}</td>
                <td>
                  <button class="btn btn-sm btn-success" @click="unbanDeviceById(d.id, d.device_id)">解封</button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>

        <div class="modal-actions">
          <button class="btn" @click="showBannedModal = false">关闭</button>
        </div>
      </div>
    </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { fmtDateTime } from '@/utils/time'
import { useUserList } from '@/composables/users/useUserList'
import { useAccountModals } from '@/composables/users/useAccountModals'
import { usePluginsModal } from '@/composables/users/usePluginsModal'
import { useUserDevices } from '@/composables/users/useUserDevices'
import { useBannedDevices } from '@/composables/users/useBannedDevices'
import { useAvatarModal } from '@/composables/users/useAvatarModal'
import { useRowActions } from '@/composables/users/useRowActions'
import { useBatchMode } from '@/composables/users/useBatchMode'

// 列表 / 统计 / 分页 / 搜索
const {
  users, loading, loadError, keyword, page, total, totalPages, stats, pageNumbers,
  loadUsers, handleSearch, clearSearch, goPage, formatDuration,
} = useUserList()

// 添加 / 昵称 / 邮箱 / 重置时长 弹窗
const {
  showAddModal, addLoading, addForm, openAddModal, submitAddUser,
  showNicknameModal, nicknameLoading, nicknameForm, openNicknameModal, submitNicknameChange,
  showEmailModal, emailLoading, emailForm, openEmailModal, submitEmailChange,
  showResetModal, resetLoading, resetForm, openResetModal, submitReset,
} = useAccountModals({ loadUsers })

// 插件查看 / 设备信息 / 封禁设备 / 头像查看
const { showPluginsModal, pluginsLoading, pluginsData, viewPlugins, formatScriptSize } = usePluginsModal()
const {
  showDeviceModal, deviceLoading, deviceData, userDevices,
  userDeviceIcon, userDevicePlatformLabel, userDeviceMeta,
  openDeviceModal, banUserDevice, unbanUserDevice,
} = useUserDevices()
const {
  showBannedModal, bannedLoading, bannedDevices, banDeviceInput, banReasonInput,
  manualBanDevice, unbanDeviceById,
} = useBannedDevices()
const { showAvatarModal, avatarViewUser, avatarDeleting, openAvatarView, confirmDeleteAvatar } = useAvatarModal({ loadUsers })

// 行操作菜单（跨子域分发）
const { openRowMenu } = useRowActions({ loadUsers, openNicknameModal, openEmailModal, openResetModal, viewPlugins, openDeviceModal })

// 批量管理模式
const {
  isBatchMode, selectedIds, selectedCount, isAllSelected, batchLoading,
  enterBatchMode, exitBatchMode, toggleSelect, toggleSelectAll,
  batchToggleSelected, batchBanDevice, batchResetDuration, batchDeleteSelected, deleteEmptyPlaylists,
} = useBatchMode({ users, loadUsers })

onMounted(() => {
  loadUsers()
})
</script>

<style scoped>
.users-wrap {
  max-width: 1320px;
  margin: 0 auto;
}

.toolbar-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
  margin-bottom: 16px;
}
.search-box {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 260px;
}
.search-box input {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid var(--border);
  border-radius: 6px;
  font-size: 13px;
  outline: none;
  min-width: 180px;
  background: var(--card-solid);
}
.search-box input:focus { border-color: var(--accent); }

.stats-row {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 12px;
  margin-bottom: 16px;
}
.stat-chip {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 18px;
  background: var(--card-solid);
  border: 1px solid var(--border);
  border-radius: 14px;
  transition: transform 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}
.stat-chip:hover { transform: translateY(-2px); }
.stat-icon {
  width: 38px; height: 38px;
  border-radius: 10px;
  display: flex; align-items: center; justify-content: center;
  flex-shrink: 0;
}
.stat-icon-total { background: #eff6ff; color: #3b82f6; }
.stat-icon-active { background: #f0fdf4; color: #16a34a; }
.stat-icon-banned { background: rgba(236, 65, 65, 0.12); color: #dc2626; }
.stat-body { display: flex; flex-direction: column; }
.stat-num { font-size: 22px; font-weight: 800; line-height: 1.2; }
.stat-label { font-size: 12px; color: var(--text-muted); }
@media (max-width: 768px) {
  .stats-row { grid-template-columns: repeat(3, 1fr); gap: 8px; }
  .stat-chip { padding: 12px; gap: 8px; }
  .stat-icon { width: 32px; height: 32px; }
  .stat-num { font-size: 18px; }
}

.toolbar-actions {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}
.toolbar-actions-row {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}
.batch-slot {
  display: flex;
  align-items: center;
}
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
.batch-bar {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 2px 10px;
  height: 32px;
  border-radius: 9px;
  background: var(--accent-soft);
  border: 1px solid var(--border);
  overflow-x: auto;
  max-width: 100%;
  scrollbar-width: thin;
}
.batch-act {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  padding: 4px 8px;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--card-bg);
  color: var(--text);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  white-space: nowrap;
  flex-shrink: 0;
  transition: all 0.2s;
}
.batch-act:hover {
  border-color: var(--accent-color);
  color: var(--accent-color);
}
.batch-act:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
.batch-act--danger {
  background: #ef4444;
  border-color: #ef4444;
  color: #fff;
}
.batch-act--danger:hover {
  background: #dc2626;
  border-color: #dc2626;
  color: #fff;
}
.batch-act--primary {
  background: var(--accent);
  border-color: var(--accent);
  color: #fff;
}
.batch-act--primary:hover {
  border-color: var(--accent);
  color: #fff;
}
.batch-count {
  font-size: 12px;
  color: var(--accent);
  font-weight: 600;
  margin-left: 2px;
}
.checkbox-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  border: 1.5px solid var(--border);
  border-radius: 5px;
  background: var(--card-solid);
  cursor: pointer;
  transition: all 0.15s;
  vertical-align: -2px;
  margin-right: 6px;
}
.checkbox-badge.checked {
  background: var(--accent);
  border-color: var(--accent);
  color: #fff;
}
.checkbox-badge:hover { border-color: var(--accent); }
.col-check { width: 40px; text-align: center; }
.col-check .checkbox-badge { margin-right: 0; }
.row-selected { background: var(--accent-soft); }
.btn-dark {
  background: var(--accent);
  color: var(--white);
  border-color: var(--accent);
}
.btn-dark:hover { background: #000; border-color: #000; }
.btn-dark:disabled { opacity: 0.6; cursor: not-allowed; }
.btn-warning {
  background: #f39c12;
  color: #fff;
  border-color: #f39c12;
}
.btn-warning:hover { background: #e67e22; border-color: #e67e22; }

.device-id-cell {
  font-size: 11px;
  font-family: monospace;
  cursor: pointer;
  color: var(--primary);
  transition: opacity 0.15s;
}
.device-id-cell:hover { opacity: 0.7; }

.table-wrapper {
  overflow-x: auto;
  -webkit-overflow-scrolling: touch;
}
table {
  width: 100%;
  border-collapse: collapse;
  min-width: 960px;
}
thead th {
  white-space: nowrap;
}

.user-avatar {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  object-fit: cover;
  cursor: pointer;
  transition: opacity 0.15s;
}
.user-avatar:hover { opacity: 0.8; }
.avatar-placeholder {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: #e0e0e0;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #999;
  font-size: 14px;
}

.row-actions {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
}

.ban-form {
  display: flex;
  gap: 10px;
  align-items: flex-start;
  margin-bottom: 16px;
  flex-wrap: wrap;
}
.ban-device-input { flex: 1.2; min-width: 180px; margin-bottom: 0; }
.ban-reason-input { flex: 1; min-width: 160px; margin-bottom: 0; }
.ban-submit {
  flex-shrink: 0;
  height: 40px;
  padding: 0 20px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.pagination {
  display: flex;
  justify-content: center;
  gap: 6px;
  margin-top: 16px;
  align-items: center;
}
.pagination button {
  padding: 6px 12px;
  border: 1px solid var(--border);
  background: var(--card-solid);
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  transition: all 0.15s;
}
.pagination button:hover:not(.active):not(:disabled) {
  border-color: var(--accent);
  color: var(--accent);
}
.pagination button.active {
  background: var(--accent);
  color: var(--white);
  border-color: var(--accent);
}
.pagination button:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
.pagination span {
  font-size: 12px;
  color: var(--text-muted);
  margin-left: 8px;
}

.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.4);
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
}
.modal {
  background: var(--card-solid);
  border-radius: 8px;
  padding: 28px;
  width: 500px;
  max-width: 100%;
  max-height: 85vh;
  overflow-y: auto;
  border: 1px solid var(--border);
}
.modal h3 {
  font-size: 17px;
  margin-bottom: 20px;
  font-weight: 700;
}
.form-group {
  margin-bottom: 16px;
}
.form-group label {
  display: block;
  font-size: 13px;
  color: var(--text-light);
  margin-bottom: 6px;
  font-weight: 500;
}
.form-group input {
  width: 100%;
  padding: 10px 14px;
  border: 1px solid var(--border);
  border-radius: 6px;
  font-size: 14px;
  outline: none;
  transition: border-color 0.15s;
  background: var(--card-solid);
}
.form-group input:focus { border-color: var(--accent); }
.form-group input:disabled { background: #fafafa; color: var(--text-muted); }
.form-group textarea {
  width: 100%;
  padding: 10px 14px;
  border: 1px solid var(--border);
  border-radius: 6px;
  font-size: 14px;
  outline: none;
  transition: border-color 0.15s;
  background: var(--card-solid);
  resize: vertical;
  font-family: inherit;
  line-height: 1.5;
}
.form-group textarea:focus { border-color: var(--accent); }
.form-group .hint {
  font-size: 12px;
  color: var(--text-muted);
  margin-top: 4px;
}
.modal-actions {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
  margin-top: 24px;
}

.modal-head-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 18px;
  padding-bottom: 14px;
  border-bottom: 1px solid var(--border, #eee);
}
.modal-head-bar h3 {
  margin: 0;
  font-size: 17px;
  font-weight: 700;
}
.modal-close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: 8px;
  border: none;
  background: transparent;
  color: var(--text-muted, #999);
  cursor: pointer;
  transition: all 0.2s;
  flex-shrink: 0;
}
.modal-close-btn:hover {
  background: rgba(0, 0, 0, 0.06);
  color: #e74c3c;
}

.modal-enter-active, .modal-leave-active { transition: opacity 0.3s ease; }
.modal-enter-from, .modal-leave-to { opacity: 0; }
.modal-enter-active .modal, .modal-leave-active .modal {
  transition: transform 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}
.modal-enter-from .modal, .modal-leave-to .modal {
  transform: scale(0.92) translateY(20px);
}

/* ===== 页面进入动效 ===== */
.fade-down-enter-active, .fade-down-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-down-enter-from { opacity: 0; transform: translateY(-12px); }

.fade-up-enter-active, .fade-up-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.fade-up-enter-from { opacity: 0; transform: translateY(12px); }

tbody tr.table-row { animation: rowIn 0.4s cubic-bezier(0.16, 1, 0.3, 1) both; }
@keyframes rowIn {
  from { opacity: 0; transform: translateY(8px); }
  to { opacity: 1; transform: translateY(0); }
}

.user-device-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-bottom: 16px;
}
.user-device-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 14px;
  border: 1px solid var(--border, #e5e7eb);
  border-radius: 10px;
  background: var(--card-solid, #fff);
  transition: border-color 0.2s;
}
.user-device-item:hover { border-color: var(--accent, #e74c3c); }
.user-device-item.is-banned { opacity: 0.75; }
.user-device-icon { flex-shrink: 0; color: var(--text-light, #9ca3af); }
.user-device-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 3px;
}
.user-device-name-row { display: flex; align-items: center; flex-wrap: wrap; gap: 6px; }
.user-device-model { font-size: 13px; font-weight: 600; color: var(--text, #1f2937); }
.platform-badge {
  font-size: 10px;
  font-weight: 700;
  padding: 2px 8px;
  border-radius: 999px;
}
.platform-desktop { background: rgba(245, 158, 11, 0.14); color: #d97706; }
.platform-mobile { background: rgba(59, 130, 246, 0.14); color: #2563eb; }
.platform-watch { background: rgba(20, 184, 166, 0.14); color: #14b8a6; }
.user-device-current {
  font-size: 10px;
  font-weight: 700;
  padding: 2px 8px;
  border-radius: 999px;
  background: rgba(231, 76, 60, 0.12);
  color: var(--accent, #e74c3c);
}
.user-device-banned-badge {
  font-size: 10px;
  font-weight: 700;
  padding: 2px 8px;
  border-radius: 999px;
  background: rgba(231, 76, 60, 0.16);
  color: #e74c3c;
}
.user-device-id {
  font-size: 11px;
  color: var(--text-muted, #9ca3af);
  word-break: break-all;
}
.user-device-meta { font-size: 11px; color: var(--text-light, #9ca3af); }
.user-device-actions { flex-shrink: 0; }
</style>
