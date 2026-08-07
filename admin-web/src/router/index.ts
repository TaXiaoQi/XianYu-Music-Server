import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'
import { getToken } from '@/api/client'

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
        path: 'wallpapers',
        name: 'Wallpapers',
        component: () => import('@/views/Wallpapers.vue'),
        meta: { title: '壁纸管理' },
      },
      {
        path: 'avatar-audit',
        name: 'AvatarAudit',
        component: () => import('@/views/AvatarAudit.vue'),
        meta: { title: '头像审核' },
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

  if (!isPublic && !hasToken) {
    next('/login')
  } else if (isPublic && hasToken && to.path === '/login') {
    next('/dashboard')
  } else {
    next()
  }
})

export default router
