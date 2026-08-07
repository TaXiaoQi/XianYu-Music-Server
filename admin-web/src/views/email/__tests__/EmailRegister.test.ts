/**
 * EmailRegister 组件测试
 */
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import EmailRegister from '../EmailRegister.vue'
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

// RouterLink stub
const RouterLinkStub = {
  template: '<a :href="to"><slot /></a>',
  props: ['to'],
}

function mountComponent() {
  return mount(EmailRegister, {
    global: {
      stubs: { RouterLink: RouterLinkStub },
    },
  })
}

beforeEach(() => {
  vi.clearAllMocks()
  vi.useFakeTimers()
  localStorageMock.clear()
})

afterEach(() => {
  vi.useRealTimers()
})

describe('EmailRegister.vue', () => {
  it('应渲染所有表单字段', () => {
    const wrapper = mountComponent()
    const inputs = wrapper.findAll('input')
    // email, code, password, password2, nickname
    expect(inputs.length).toBe(5)
    expect(wrapper.find('button[type="submit"]').exists()).toBe(true)
    expect(wrapper.find('.code-btn').exists()).toBe(true)
  })

  it('应渲染登录链接指向 /email/login', () => {
    const wrapper = mountComponent()
    const links = wrapper.findAll('a')
    const loginLink = links.find(l => l.text().includes('直接登录'))
    expect(loginLink?.attributes('href')).toBe('/email/login')
  })

  it('空邮箱点击获取验证码不应调用 API', async () => {
    const sendSpy = vi.spyOn(emailApi, 'sendCode')
    vi.spyOn(emailApi, 'getEmailToken').mockReturnValue(null)

    const wrapper = mountComponent()
    await wrapper.find('.code-btn').trigger('click')
    await flushPromises()

    expect(sendSpy).not.toHaveBeenCalled()
  })

  it('非法邮箱点击获取验证码不应调用 API', async () => {
    const sendSpy = vi.spyOn(emailApi, 'sendCode')
    vi.spyOn(emailApi, 'getEmailToken').mockReturnValue(null)

    const wrapper = mountComponent()
    await wrapper.find('input[type="email"]').setValue('notanemail')
    await wrapper.find('.code-btn').trigger('click')
    await flushPromises()

    expect(sendSpy).not.toHaveBeenCalled()
  })

  it('合法邮箱获取验证码成功后应启动倒计时', async () => {
    vi.spyOn(emailApi, 'sendCode').mockResolvedValueOnce({
      code: 200,
      msg: '验证码已发送',
      data: null,
    })
    vi.spyOn(emailApi, 'getEmailToken').mockReturnValue(null)

    const wrapper = mountComponent()
    await wrapper.find('input[type="email"]').setValue('user@test.com')
    await wrapper.find('.code-btn').trigger('click')
    await flushPromises()

    // 倒计时按钮文本应变化
    const btn = wrapper.find('.code-btn')
    expect(btn.text()).toContain('60s')
    expect(btn.attributes('disabled')).toBeDefined()

    // 推进 30 秒
    vi.advanceTimersByTime(30000)
    await flushPromises()
    expect(wrapper.find('.code-btn').text()).toContain('30s')
  })

  it('验证码发送失败不应启动倒计时', async () => {
    vi.spyOn(emailApi, 'sendCode').mockResolvedValueOnce({
      code: 400,
      msg: '发送过于频繁',
      data: null,
    })
    vi.spyOn(emailApi, 'getEmailToken').mockReturnValue(null)

    const wrapper = mountComponent()
    await wrapper.find('input[type="email"]').setValue('user@test.com')
    await wrapper.find('.code-btn').trigger('click')
    await flushPromises()

    const btn = wrapper.find('.code-btn')
    expect(btn.text()).not.toContain('s')
    expect(btn.text()).toContain('获取验证码')
  })

  it('注册成功应跳转到登录页', async () => {
    vi.spyOn(emailApi, 'emailRegister').mockResolvedValueOnce({
      code: 200,
      msg: '注册成功',
      data: null,
    })
    vi.spyOn(emailApi, 'getEmailToken').mockReturnValue(null)

    const wrapper = mountComponent()

    await wrapper.find('input[type="email"]').setValue('new@test.com')
    const inputs = wrapper.findAll('input')
    await inputs[1].setValue('123456') // code
    await inputs[2].setValue('pass123') // password
    await inputs[3].setValue('pass123') // password2

    await wrapper.find('form').trigger('submit.prevent')
    await flushPromises()

    // 组件中注册成功后 setTimeout 1200ms 跳转
    vi.advanceTimersByTime(1200)
    await flushPromises()

    expect(mockPush).toHaveBeenCalledWith('/email/login')
  })

  it('密码不一致应阻止提交', async () => {
    const regSpy = vi.spyOn(emailApi, 'emailRegister')
    vi.spyOn(emailApi, 'getEmailToken').mockReturnValue(null)

    const wrapper = mountComponent()

    await wrapper.find('input[type="email"]').setValue('new@test.com')
    const inputs = wrapper.findAll('input')
    await inputs[1].setValue('123456')
    await inputs[2].setValue('pass123')
    await inputs[3].setValue('different')

    await wrapper.find('form').trigger('submit.prevent')
    await flushPromises()

    expect(regSpy).not.toHaveBeenCalled()
    expect(mockPush).not.toHaveBeenCalled()
  })

  it('注册失败不应跳转', async () => {
    vi.spyOn(emailApi, 'emailRegister').mockResolvedValueOnce({
      code: 400,
      msg: '该邮箱已注册',
      data: null,
    })
    vi.spyOn(emailApi, 'getEmailToken').mockReturnValue(null)

    const wrapper = mountComponent()

    await wrapper.find('input[type="email"]').setValue('new@test.com')
    const inputs = wrapper.findAll('input')
    await inputs[1].setValue('123456')
    await inputs[2].setValue('pass123')
    await inputs[3].setValue('pass123')

    await wrapper.find('form').trigger('submit.prevent')
    await flushPromises()
    vi.advanceTimersByTime(2000)
    await flushPromises()

    expect(mockPush).not.toHaveBeenCalled()
  })

  it('已登录用户访问应自动跳转主页', async () => {
    vi.spyOn(emailApi, 'getEmailToken').mockReturnValue('existing-token')

    mountComponent()
    await flushPromises()

    expect(mockReplace).toHaveBeenCalledWith('/email/home')
  })
})
