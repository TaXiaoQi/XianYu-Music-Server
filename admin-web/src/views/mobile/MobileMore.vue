<template>
  <div class="mobile-more">
    <section v-for="group in groups" :key="group.title" class="more-group">
      <h2>{{ group.title }}</h2>
      <router-link v-for="item in group.items" :key="item.to" :to="item.to" class="more-item">
        <span>{{ item.label }}</span>
        <small>{{ item.desc }}</small>
      </router-link>
    </section>

    <router-link to="/dashboard" class="desktop-link">切换到电脑端后台</router-link>
  </div>
</template>

<script setup lang="ts">
const groups = [
  {
    title: '内容管理',
    items: [
      { to: '/m/announcements', label: '公告管理', desc: '发布和维护 APP 公告' },
      { to: '/m/about-config', label: '关于页配置', desc: '维护关于页展示内容' },
      { to: '/m/site-config', label: 'Logo 配置', desc: '上传站点 Logo 即时生效' },
      { to: '/m/wallpapers', label: '壁纸管理', desc: '审核和管理壁纸资源' },
      { to: '/m/avatar-audit', label: '头像/改名审核', desc: '处理用户资料审核' },
      { to: '/m/user-agreement', label: '用户协议', desc: '维护注册和登录页协议内容' },
    ],
  },
  {
    title: '系统管理',
    items: [
      { to: '/m/admin-account', label: '后台账号管理', desc: '管理管理员账号与登录' },
      { to: '/m/external-notification', label: '外部通知', desc: '管理后台状态通知邮箱' },
      { to: '/m/email-config', label: '邮箱机设置', desc: '配置邮箱 API 或 SMTP' },
      { to: '/m/turnstile-config', label: '人机验证设置', desc: '配置 Turnstile / hCaptcha' },
      { to: '/m/config-file', label: '配置文件管理', desc: '维护服务端 config.json' },
    ],
  },
  {
    title: '日志与工具',
    items: [
      { to: '/m/error-log', label: '报错日志', desc: '查看 APP 崩溃和错误' },
      { to: '/m/app-login-log', label: 'APP 登录日志', desc: '查看客户端登录记录' },
      { to: '/m/logs', label: '后台日志', desc: '查看后台操作记录' },
      { to: '/m/api-test', label: '接口测试', desc: '调试服务端接口' },
    ],
  },
]
</script>

<style scoped>
.mobile-more {
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.mobile-more > * {
  animation: moreIn 0.42s cubic-bezier(0.16, 1, 0.3, 1) both;
}
.mobile-more > *:nth-child(1) { animation-delay: 0.02s; }
.mobile-more > *:nth-child(2) { animation-delay: 0.10s; }
.mobile-more > *:nth-child(3) { animation-delay: 0.18s; }
.mobile-more > *:nth-child(4) { animation-delay: 0.26s; }
@keyframes moreIn {
  from { opacity: 0; transform: translateY(12px); }
  to { opacity: 1; transform: translateY(0); }
}
.more-group {
  padding: 16px;
  border: 1px solid var(--border);
  border-radius: 22px;
  background: var(--card);
  box-shadow: var(--shadow-soft);
}
.more-group h2 {
  margin: 0 0 12px;
  font-size: 15px;
}
.more-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 13px 2px;
  color: var(--text);
  text-decoration: none;
  border-top: 1px solid var(--border);
}
.more-item:first-of-type {
  border-top: 0;
}
.more-item span {
  font-size: 14px;
  font-weight: 800;
}
.more-item small {
  color: var(--text-muted);
  font-size: 12px;
}
.desktop-link {
  display: block;
  padding: 14px;
  border-radius: 18px;
  background: rgba(236, 65, 65, 0.09);
  color: #EC4141;
  text-align: center;
  text-decoration: none;
  font-size: 13px;
  font-weight: 800;
}
</style>
