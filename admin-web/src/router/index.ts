import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'
import { getToken } from '@/api/client'
import { isAdminSessionExpired, logoutByIdleTimeout, markAdminActivity } from '@/utils/adminIdleLogout'
import { isMobileBrowser, toMobilePath } from '@/utils/device'

const routes: RouteRecordRaw[] = [
  {
    path: '/login',
    name: 'Login',
    component: () => import('@/views/Login.vue'),
    meta: { public: true },
  },
  // 邮箱注册登录测试模块（独立于后台管理系统）
  {
    path: '/email',
    redirect: '/email/login',
  },
  {
    path: '/email/login',
    name: 'EmailLogin',
    component: () => import('@/views/email/EmailLogin.vue'),
    meta: { public: true },
  },
  {
    path: '/email/register',
    name: 'EmailRegister',
    component: () => import('@/views/email/EmailRegister.vue'),
    meta: { public: true },
  },
  {
    path: '/email/forgot',
    name: 'EmailForgot',
    component: () => import('@/views/email/EmailForgot.vue'),
    meta: { public: true },
  },
  {
    path: '/email/home',
    name: 'EmailHome',
    component: () => import('@/views/email/EmailHome.vue'),
    meta: { public: true },
  },
  {
    path: '/m',
    component: () => import('@/layouts/MobileLayout.vue'),
    redirect: '/m/dashboard',
    meta: { mobile: true },
    children: [
      {
        path: 'dashboard',
        name: 'MobileDashboard',
        component: () => import('@/views/mobile/MobileDashboard.vue'),
        meta: { title: '移动端首页', mobile: true },
      },
      {
        path: 'users',
        name: 'MobileUsers',
        component: () => import('@/views/mobile/MobileUsers.vue'),
        meta: { title: '用户管理', mobile: true },
      },
      {
        path: 'version',
        name: 'MobileVersion',
        component: () => import('@/views/mobile/MobileVersion.vue'),
        meta: { title: '版本管理', mobile: true },
      },
      {
        path: 'feedback',
        name: 'MobileFeedback',
        component: () => import('@/views/mobile/MobileFeedback.vue'),
        meta: { title: '反馈与建议', mobile: true },
      },
      {
        path: 'more',
        name: 'MobileMore',
        component: () => import('@/views/mobile/MobileMore.vue'),
        meta: { title: '更多功能', mobile: true },
      },
      {
        path: 'announcements',
        name: 'MobileAnnouncements',
        component: () => import('@/views/mobile/MobileAnnouncements.vue'),
        meta: { title: '公告管理', mobile: true },
      },
      {
        path: 'about-config',
        name: 'MobileAboutConfig',
        component: () => import('@/views/mobile/MobileAboutConfig.vue'),
        meta: { title: '关于页配置', mobile: true },
      },
      {
        path: 'wallpapers',
        name: 'MobileWallpapers',
        component: () => import('@/views/mobile/MobileWallpapers.vue'),
        meta: { title: '壁纸管理', mobile: true },
      },
      {
        path: 'avatar-audit',
        name: 'MobileAvatarAudit',
        component: () => import('@/views/mobile/MobileAvatarAudit.vue'),
        meta: { title: '头像/改名审核', mobile: true },
      },
      {
        path: 'email-config',
        name: 'MobileEmailConfig',
        component: () => import('@/views/mobile/MobileEmailConfig.vue'),
        meta: { title: '邮箱机设置', mobile: true },
      },
      {
        path: 'turnstile-config',
        name: 'MobileTurnstileConfig',
        component: () => import('@/views/TurnstileConfig.vue'),
        meta: { title: '人机验证设置', mobile: true },
      },
      {
        path: 'admins',
        name: 'MobileAdmins',
        component: () => import('@/views/mobile/MobileAdmins.vue'),
        meta: { title: '管理员管理', mobile: true },
      },
      {
        path: 'account',
        name: 'MobileAccount',
        component: () => import('@/views/mobile/MobileAccount.vue'),
        meta: { title: '账户管理', mobile: true },
      },
      {
        path: 'password',
        name: 'MobilePassword',
        component: () => import('@/views/mobile/MobilePassword.vue'),
        meta: { title: '修改密码', mobile: true },
      },
      {
        path: 'logs',
        name: 'MobileLogs',
        component: () => import('@/views/mobile/MobileLogs.vue'),
        meta: { title: '后台日志', mobile: true },
      },
      {
        path: 'database',
        name: 'MobileDatabase',
        component: () => import('@/views/mobile/MobileDatabase.vue'),
        meta: { title: '数据库管理', mobile: true },
      },
      {
        path: 'api-test',
        name: 'MobileApiTest',
        component: () => import('@/views/mobile/MobileApiTest.vue'),
        meta: { title: '接口测试', mobile: true },
      },
      {
        path: 'error-log',
        name: 'MobileErrorLog',
        component: () => import('@/views/mobile/MobileErrorLog.vue'),
        meta: { title: '报错日志', mobile: true },
      },
      {
        path: 'app-login-log',
        name: 'MobileAppLoginLog',
        component: () => import('@/views/mobile/MobileAppLoginLog.vue'),
        meta: { title: 'APP登录日志', mobile: true },
      },
      {
        path: ':pathMatch(.*)*',
        name: 'MobileNotFound',
        component: () => import('@/views/mobile/MobileMore.vue'),
        meta: { title: '更多功能', mobile: true },
      },
    ],
  },
  {
    path: '/',
    component: () => import('@/layouts/AdminLayout.vue'),
    redirect: '/dashboard',
    children: [
      {
        path: 'dashboard',
        name: 'Dashboard',
        component: () => import('@/views/Dashboard.vue'),
        meta: { title: '仪表盘' },
      },
      {
        path: 'users',
        name: 'Users',
        component: () => import('@/views/Users.vue'),
        meta: { title: '用户管理' },
      },
      {
        path: 'error-log',
        name: 'ErrorLog',
        component: () => import('@/views/ErrorLog.vue'),
        meta: { title: '报错日志' },
      },
      {
        path: 'app-login-log',
        name: 'AppLoginLog',
        component: () => import('@/views/AppLoginLog.vue'),
        meta: { title: 'APP登录日志' },
      },
      {
        path: 'version',
        name: 'Version',
        component: () => import('@/views/Version.vue'),
        meta: { title: '版本管理' },
      },
      {
        path: 'announcements',
        name: 'Announcements',
        component: () => import('@/views/Announcements.vue'),
        meta: { title: '公告管理' },
      },
      {
        path: 'about-config',
        name: 'AboutConfig',
        component: () => import('@/views/AboutConfig.vue'),
        meta: { title: '关于页配置' },
      },
      {
        path: 'wallpapers',
        name: 'Wallpapers',
        component: () => import('@/views/Wallpapers.vue'),
        meta: { title: '壁纸管理' },
      },
      {
        path: 'avatar-audit',
        name: 'AvatarAudit',
        component: () => import('@/views/AvatarAudit.vue'),
        meta: { title: '头像/改名审核' },
      },
      {
        path: 'email-config',
        name: 'EmailConfig',
        component: () => import('@/views/EmailConfig.vue'),
        meta: { title: '邮箱API配置' },
      },
      {
        path: 'turnstile-config',
        name: 'TurnstileConfig',
        component: () => import('@/views/TurnstileConfig.vue'),
        meta: { title: '人机验证设置' },
      },
      {
        path: 'feedback',
        name: 'Feedback',
        component: () => import('@/views/Feedback.vue'),
        meta: { title: '反馈与建议' },
      },
      {
        path: 'admins',
        name: 'Admins',
        component: () => import('@/views/Admins.vue'),
        meta: { title: '管理员管理' },
      },
      {
        path: 'account',
        name: 'Account',
        component: () => import('@/views/Account.vue'),
        meta: { title: '账户管理' },
      },
      {
        path: 'password',
        name: 'Password',
        component: () => import('@/views/Password.vue'),
        meta: { title: '修改密码' },
      },
      {
        path: 'logs',
        name: 'Logs',
        component: () => import('@/views/Logs.vue'),
        meta: { title: '后台日志' },
      },
      {
        path: 'database',
        name: 'Database',
        component: () => import('@/views/Database.vue'),
        meta: { title: '数据库管理' },
      },
      {
        path: 'api-test',
        name: 'ApiTest',
        component: () => import('@/views/ApiTest.vue'),
        meta: { title: '接口测试' },
      },
      // 兜底：未匹配的子路由
      {
        path: ':pathMatch(.*)*',
        name: 'NotFound',
        component: () => import('@/views/Placeholder.vue'),
        meta: { title: '页面' },
      },
    ],
  },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

// 路由守卫：未登录跳转 login
router.beforeEach((to, _from, next) => {
  const isPublic = to.meta.public === true
  const hasToken = !!getToken()
  const isMobile = isMobileBrowser()

  if (!isPublic && !hasToken) {
    next('/login')
  } else if (!isPublic && hasToken && isAdminSessionExpired()) {
    logoutByIdleTimeout()
    next(false)
  } else if (isPublic && hasToken && to.path === '/login') {
    next(isMobile ? '/m/dashboard' : '/dashboard')
  } else if (!isPublic && hasToken && isMobile && !to.path.startsWith('/m')) {
    next(toMobilePath(to.path))
  } else {
    if (!isPublic && hasToken) {
      markAdminActivity()
    }
    next()
  }
})

export default router
