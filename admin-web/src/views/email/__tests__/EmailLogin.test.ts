/**
 * EmailLogin 组件测试
 */
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import EmailLogin from '../EmailLogin.vue'
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

// RouterLink stub - renders as <a> tag
const RouterLinkStub = {
  template: '<a :href="to"><slot /></a>',
  props: ['to'],
}

function mountComponent() {
  return mount(EmailLogin, {
    global: {
      stubs: { RouterLink: RouterLinkStub },
    },
  })
}

beforeEach(() => {
  vi.clearAllMocks()
  localStorageMock.clear()
})

describe('EmailLogin.vue', () => {
  it('应渲染品牌标题和表单', () => {
    const wrapper = mountComponent()
    expect(wrapper.find('h1').text()).toBe('弦予邮箱')
    expect(wrapper.find('input[type="email"]').exists()).toBe(true)
    expect(wrapper.find('input[type="password"]').exists()).toBe(true)
    expect(wrapper.find('button[type="submit"]').exists()).toBe(true)
  })

  it('应渲染注册和找回密码链接', () => {
    const wrapper = mountComponent()
    const links = wrapper.findAll('a')
    const linkTexts = links.map(l => l.text())
    expect(linkTexts).toContain('注册账号')
    expect(linkTexts).toContain('忘记密码')
  })

  it('注册链接应指向 /email/register', () => {
    const wrapper = mountComponent()
    const links = wrapper.findAll('a')
    const registerLink = links.find(l => l.text() === '注册账号')
    expect(registerLink?.attributes('href')).toBe('/email/register')
  })

  it('忘记密码链接应指向 /email/forgot', () => {
    const wrapper = mountComponent()
    const links = wrapper.findAll('a')
    const forgotLink = links.find(l => l.text() === '忘记密码')
    expect(forgotLink?.attributes('href')).toBe('/email/forgot')
  })

  it('登录成功应存储 token 并跳转 /email/home', async () => {
    const loginSpy = vi.spyOn(emailApi, 'emailLogin').mockResolvedValueOnce({
      code: 200,
      msg: '登录成功',
      data: {
        token: 'test-jwt-token',
        user: { id: 1, email: 'test@x.com', nickname: 'Test' },
      },
    })
    vi.spyOn(emailApi, 'getEmailToken').mockReturnValue(null)

    const wrapper = mountComponent()

    await wrapper.find('input[type="email"]').setValue('test@x.com')
    await wrapper.find('input[type="password"]').setValue('password123')
    await wrapper.find('form').trigger('submit.prevent')
    await flushPromises()

    expect(loginSpy).toHaveBeenCalledWith('test@x.com', 'password123')
    expect(localStorageMock.getItem('email_token')).toBe('test-jwt-token')
    expect(mockPush).toHaveBeenCalledWith('/email/home')
  })

  it('登录失败应显示错误提示', async () => {
    vi.spyOn(emailApi, 'emailLogin').mockResolvedValueOnce({
      code: 400,
      msg: '邮箱或密码不正确',
      data: null,
    })
    vi.spyOn(emailApi, 'getEmailToken').mockReturnValue(null)

    const wrapper = mountComponent()

    await wrapper.find('input[type="email"]').setValue('test@x.com')
    await wrapper.find('input[type="password"]').setValue('wrongpass')
    await wrapper.find('form').trigger('submit.prevent')
    await flushPromises()

    // 不应跳转
    expect(mockPush).not.toHaveBeenCalled()
    // 不应存储 token
    expect(localStorageMock.getItem('email_token')).toBeNull()
  })

  it('loading 状态应禁用按钮', async () => {
    vi.spyOn(emailApi, 'emailLogin').mockReturnValue(new Promise(() => {})) // 永不 resolve
    vi.spyOn(emailApi, 'getEmailToken').mockReturnValue(null)

    const wrapper = mountComponent()

    await wrapper.find('input[type="email"]').setValue('test@x.com')
    await wrapper.find('input[type="password"]').setValue('password123')
    await wrapper.find('form').trigger('submit.prevent')
    await flushPromises()

    const btn = wrapper.find('button[type="submit"]')
    expect(btn.attributes('disabled')).toBeDefined()
    expect(btn.text()).toContain('登录中')
  })

  it('已登录用户访问应自动跳转主页', async () => {
    vi.spyOn(emailApi, 'getEmailToken').mockReturnValue('existing-token')

    mountComponent()
    await flushPromises()

    expect(mockReplace).toHaveBeenCalledWith('/email/home')
  })
})
