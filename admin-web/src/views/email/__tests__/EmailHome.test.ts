/**
 * EmailHome 组件测试
 */
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import EmailHome from '../EmailHome.vue'
import * as emailApi from '@/api/email'

// mock vue-router
const mockPush = vi.fn()
const mockReplace = vi.fn()
vi.mock('vue-router', () => ({
  useRouter: () => ({ push: mockPush, replace: mockReplace }),
}))

// mock localStorage
const localStorageMock = (() => {
  let store: Record<string, string> = {}
  return {
    getItem: (key: string) => store[key] ?? null,
    setItem: (key: string, val: string) => { store[key] = val },
    removeItem: (key: string) => { delete store[key] },
    clear: () => { store = {} },
  }
})()
vi.stubGlobal('localStorage', localStorageMock)

function mountComponent() {
  return mount(EmailHome)
}

const mockProfile = {
  id: 1,
  email: 'test@xianyu.com',
  nickname: 'TestUser',
  status: 1,
  created_at: '2026-01-15 10:30:00',
  last_login: '2026-08-07 14:20:00',
  logs: [
    { action: 'login', detail: '127.0.0.1', created_at: '2026-08-07 14:20:00' },
    { action: 'register', detail: '127.0.0.1', created_at: '2026-01-15 10:30:00' },
  ],
}

beforeEach(() => {
  vi.clearAllMocks()
  vi.useFakeTimers()
  localStorageMock.clear()
})

afterEach(() => {
  vi.useRealTimers()
})

describe('EmailHome.vue', () => {
  it('无 token 应跳转到登录页', async () => {
    vi.spyOn(emailApi, 'getEmailToken').mockReturnValue(null)

    mountComponent()
    await flushPromises()

    expect(mockReplace).toHaveBeenCalledWith('/email/login')
  })

  it('有 token 应加载用户信息', async () => {
    vi.spyOn(emailApi, 'getEmailToken').mockReturnValue('valid-token')
    const profileSpy = vi.spyOn(emailApi, 'emailGetProfile').mockResolvedValueOnce({
      code: 200,
      msg: '',
      data: mockProfile,
    })

    const wrapper = mountComponent()
    await flushPromises()

    expect(profileSpy).toHaveBeenCalled()
    expect(wrapper.find('h2').text()).toBe('TestUser')
    expect(wrapper.find('.email-text').text()).toBe('test@xianyu.com')
  })

  it('应显示加载状态', async () => {
    vi.spyOn(emailApi, 'getEmailToken').mockReturnValue('valid-token')
    vi.spyOn(emailApi, 'emailGetProfile').mockReturnValue(new Promise(() => {}))

    const wrapper = mountComponent()
    await flushPromises()

    expect(wrapper.find('.loading-state').exists()).toBe(true)
    expect(wrapper.find('.loading-spinner').exists()).toBe(true)
    expect(wrapper.text()).toContain('加载中')
  })

  it('应正确渲染用户资料卡片', async () => {
    vi.spyOn(emailApi, 'getEmailToken').mockReturnValue('valid-token')
    vi.spyOn(emailApi, 'emailGetProfile').mockResolvedValueOnce({
      code: 200,
      msg: '',
      data: mockProfile,
    })

    const wrapper = mountComponent()
    await flushPromises()

    // 昵称和邮箱
    expect(wrapper.find('h2').text()).toBe('TestUser')
    expect(wrapper.find('.email-text').text()).toBe('test@xianyu.com')

    // 状态徽章
    const badge = wrapper.find('.status-badge')
    expect(badge.exists()).toBe(true)
    expect(badge.classes()).toContain('active')
    expect(badge.text()).toBe('正常')

    // 头像首字母
    expect(wrapper.find('.avatar').text()).toBe('T')
  })

  it('应显示注册时间和最后登录时间', async () => {
    vi.spyOn(emailApi, 'getEmailToken').mockReturnValue('valid-token')
    vi.spyOn(emailApi, 'emailGetProfile').mockResolvedValueOnce({
      code: 200,
      msg: '',
      data: mockProfile,
    })

    const wrapper = mountComponent()
    await flushPromises()

    const metaValues = wrapper.findAll('.meta-value')
    expect(metaValues.length).toBeGreaterThanOrEqual(2)
    expect(metaValues[0].text()).toContain('2026-01-15')
    expect(metaValues[1].text()).toContain('2026-08-07')
  })

  it('应渲染活动日志列表', async () => {
    vi.spyOn(emailApi, 'getEmailToken').mockReturnValue('valid-token')
    vi.spyOn(emailApi, 'emailGetProfile').mockResolvedValueOnce({
      code: 200,
      msg: '',
      data: mockProfile,
    })

    const wrapper = mountComponent()
    await flushPromises()

    const logItems = wrapper.findAll('.log-item')
    expect(logItems.length).toBe(2)

    // 第一条日志应为登录
    const firstAction = logItems[0].find('.log-action')
    expect(firstAction.text()).toBe('登录')

    // 第二条日志应为注册
    const secondAction = logItems[1].find('.log-action')
    expect(secondAction.text()).toBe('注册')
  })

  it('无活动日志应显示空状态', async () => {
    vi.spyOn(emailApi, 'getEmailToken').mockReturnValue('valid-token')
    vi.spyOn(emailApi, 'emailGetProfile').mockResolvedValueOnce({
      code: 200,
      msg: '',
      data: { ...mockProfile, logs: [] },
    })

    const wrapper = mountComponent()
    await flushPromises()

    expect(wrapper.find('.empty-state').exists()).toBe(true)
    expect(wrapper.find('.empty-state').text()).toContain('暂无活动记录')
  })

  it('token 过期应清除 token 并跳转登录', async () => {
    vi.spyOn(emailApi, 'getEmailToken').mockReturnValue('expired-token')
    vi.spyOn(emailApi, 'emailGetProfile').mockResolvedValueOnce({
      code: 401,
      msg: '未登录或登录已过期',
      data: null,
    })
    const clearSpy = vi.spyOn(emailApi, 'clearEmailToken')

    mountComponent()
    await flushPromises()

    expect(clearSpy).toHaveBeenCalled()

    // 跳转在 setTimeout 1000ms 中
    vi.advanceTimersByTime(1000)
    await flushPromises()

    expect(mockReplace).toHaveBeenCalledWith('/email/login')
  })

  it('退出登录应清除 token 并跳转', async () => {
    vi.spyOn(emailApi, 'getEmailToken').mockReturnValue('valid-token')
    vi.spyOn(emailApi, 'emailGetProfile').mockResolvedValueOnce({
      code: 200,
      msg: '',
      data: mockProfile,
    })
    const clearSpy = vi.spyOn(emailApi, 'clearEmailToken')

    const wrapper = mountComponent()
    await flushPromises()

    await wrapper.find('.logout-btn').trigger('click')

    expect(clearSpy).toHaveBeenCalled()
    expect(mockReplace).toHaveBeenCalledWith('/email/login')
  })

  it('禁用状态用户不应显示 active 徽章', async () => {
    vi.spyOn(emailApi, 'getEmailToken').mockReturnValue('valid-token')
    vi.spyOn(emailApi, 'emailGetProfile').mockResolvedValueOnce({
      code: 200,
      msg: '',
      data: { ...mockProfile, status: 0 },
    })

    const wrapper = mountComponent()
    await flushPromises()

    const badge = wrapper.find('.status-badge')
    expect(badge.classes()).not.toContain('active')
    expect(badge.text()).toBe('禁用')
  })

  it('重置密码操作应显示橙色标签', async () => {
    vi.spyOn(emailApi, 'getEmailToken').mockReturnValue('valid-token')
    vi.spyOn(emailApi, 'emailGetProfile').mockResolvedValueOnce({
      code: 200,
      msg: '',
      data: {
        ...mockProfile,
        logs: [
          { action: 'reset_password', detail: '127.0.0.1', created_at: '2026-08-07 15:00:00' },
        ],
      },
    })

    const wrapper = mountComponent()
    await flushPromises()

    const actionTag = wrapper.find('.log-action')
    expect(actionTag.text()).toBe('重置密码')
    expect(actionTag.classes()).toContain('tag-orange')
  })
})
