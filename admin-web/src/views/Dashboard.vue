<template>
  <div class="dashboard-wrap">
    <!-- 标题区 -->
    <div class="dsh-head">
      <div class="dsh-head-l">
        <div class="dsh-head-label">Dashboard Overview</div>
        <div class="dsh-head-title">数据<em>概览</em></div>
        <div class="dsh-head-date">{{ today }} · 弦予音乐后台</div>
      </div>
      <div class="dsh-head-r">
        <span class="dsh-live-dot"></span>
        <span>实时同步</span>
      </div>
    </div>

    <!-- 统计卡片 -->
    <div class="stats-grid" v-if="!loading && !loadError">
      <div class="stat-card">
        <div class="label">总用户数</div>
        <div class="value">{{ stats.total_users ?? 0 }}</div>
        <div class="sub">今日新增 {{ stats.today_users ?? 0 }} · 昨日 {{ stats.yesterday_users ?? 0 }}</div>
      </div>
      <div class="stat-card">
        <div class="label">今日音源调用</div>
        <div class="value">{{ stats.today_source_calls ?? 0 }}</div>
        <div class="sub">总计 {{ stats.total_source_calls ?? 0 }} 次</div>
      </div>
      <div class="stat-card">
        <div class="label">今日登录</div>
        <div class="value">{{ stats.today_logins ?? 0 }}</div>
        <div class="sub">总计 {{ stats.total_logins ?? 0 }} 次</div>
      </div>
      <div class="stat-card">
        <div class="label">今日报错</div>
        <div class="value">{{ stats.today_errors ?? 0 }}</div>
        <div class="sub">总计 {{ stats.total_errors ?? 0 }} 条</div>
      </div>
      <div class="stat-card">
        <div class="label">今日分享</div>
        <div class="value">{{ stats.today_shares ?? 0 }}</div>
        <div class="sub">总计 {{ stats.total_shares ?? 0 }} 次</div>
      </div>
      <div class="stat-card">
        <div class="label">管理员数</div>
        <div class="value">{{ stats.total_admins ?? 0 }}</div>
        <div class="sub">系统管理员</div>
      </div>
    </div>

    <!-- 加载中 -->
    <div v-if="loading" class="empty">加载中...</div>
    <div v-if="!loading && loadError" class="empty">{{ loadError }}</div>

    <!-- 快捷操作 -->
    <div class="card" style="margin-top: 20px;">
      <h3>快捷操作</h3>
      <div style="display: flex; gap: 12px; flex-wrap: wrap;">
        <router-link to="/users" class="btn btn-primary">用户管理</router-link>
        <router-link to="/database" class="btn">数据库管理</router-link>
        <router-link to="/version" class="btn">版本管理</router-link>
        <router-link to="/api-test" class="btn">接口测试</router-link>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { adminApi } from '@/api/client'

interface DashboardStats {
  total_users?: number
  today_users?: number
  yesterday_users?: number
  total_admins?: number
  total_source_calls?: number
  today_source_calls?: number
  yesterday_source_calls?: number
  today_source_success?: number
  total_source_success?: number
  total_errors?: number
  today_errors?: number
  yesterday_errors?: number
  total_shares?: number
  today_shares?: number
  yesterday_shares?: number
  total_logins?: number
  today_logins?: number
}

const stats = ref<DashboardStats>({})
const loading = ref(true)
const loadError = ref('')

const today = new Date().toLocaleDateString('zh-CN', { year: 'numeric', month: 'long', day: 'numeric' })

onMounted(async () => {
  const res = await adminApi<DashboardStats>('dashboard_stats')
  if (res.code === 200 && res.data) {
    stats.value = res.data
  } else {
    loadError.value = res.msg || '数据加载失败（数据库可能未连接）'
  }
  loading.value = false
})
</script>

<style scoped>
.dashboard-wrap {
  max-width: 1320px;
  margin: 0 auto;
}

/* 标题区 */
.dsh-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  margin-bottom: 28px;
  padding-bottom: 20px;
  border-bottom: 1px solid var(--border);
}
.dsh-head-label {
  font-size: 11px;
  letter-spacing: 0.25em;
  text-transform: uppercase;
  color: var(--text-muted);
  font-weight: 500;
  margin-bottom: 4px;
}
.dsh-head-title {
  font-size: 32px;
  font-weight: 700;
  letter-spacing: -0.02em;
  line-height: 1;
  color: var(--accent);
}
.dsh-head-title em {
  font-style: italic;
  font-weight: 300;
  color: #c77d5d;
}
.dsh-head-date {
  font-size: 12px;
  color: var(--text-soft);
  margin-top: 8px;
}
.dsh-head-r {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: var(--text-light);
}
.dsh-live-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: #4a6b5c;
  position: relative;
}
.dsh-live-dot::after {
  content: '';
  position: absolute;
  inset: -4px;
  border-radius: 50%;
  background: #4a6b5c;
  opacity: 0.3;
  animation: dshPulse 2s ease-out infinite;
}
@keyframes dshPulse {
  0% { transform: scale(1); opacity: 0.4; }
  100% { transform: scale(2.5); opacity: 0; }
}
</style>
