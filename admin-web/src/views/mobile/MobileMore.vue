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
      { to: '/m/wallpapers', label: '壁纸管理', desc: '审核和管理壁纸资源' },
      { to: '/m/avatar-audit', label: '头像/改名审核', desc: '处理用户资料审核' },
    ],
  },
  {
    title: '系统管理',
    items: [
      { to: '/m/admins', label: '管理员管理', desc: '管理后台管理员账号' },
      { to: '/m/account', label: '账户管理', desc: '查看当前账户资料' },
      { to: '/m/password', label: '修改密码', desc: '更新后台登录密码' },
      { to: '/m/email-config', label: '邮箱机设置', desc: '配置邮箱 API 或 SMTP' },
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
