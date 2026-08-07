/**
 * 邮箱模块 API 客户端测试
 */
import { describe, it, expect, beforeEach, vi } from 'vitest'
import {
  sendCode,
  emailRegister,
  emailLogin,
  emailResetPassword,
  emailGetProfile,
  getEmailToken,
  setEmailToken,
  clearEmailToken,
  getEmailUser,
  setEmailUser,
} from '../email'

// mock fetch
const mockFetch = vi.fn()
vi.stubGlobal('fetch', mockFetch)

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

beforeEach(() => {
  mockFetch.mockReset()
  localStorageMock.clear()
})

describe('邮箱 API 客户端', () => {
  describe('token 管理', () => {
    it('setEmailToken / getEmailToken 应正确存取', () => {
      expect(getEmailToken()).toBeNull()
      setEmailToken('my-token-123')
      expect(getEmailToken()).toBe('my-token-123')
    })

    it('clearEmailToken 应清除 token 和 user', () => {
      setEmailToken('tok')
      setEmailUser({ id: 1, email: 'a@b.com', nickname: 'A' })
      clearEmailToken()
      expect(getEmailToken()).toBeNull()
      expect(getEmailUser()).toBeNull()
    })

    it('setEmailUser / getEmailUser 应正确存取', () => {
      const user = { id: 42, email: 'test@x.com', nickname: 'Test' }
      setEmailUser(user)
      expect(getEmailUser()).toEqual(user)
    })

    it('getEmailUser 非法 JSON 应返回 null', () => {
      localStorageMock.setItem('email_user', 'not-json')
      expect(getEmailUser()).toBeNull()
    })
  })

  describe('sendCode', () => {
    it('应向 /api?action=email_send_code 发送 POST 请求', async () => {
      mockFetch.mockResolvedValueOnce({
        json: async () => ({ code: 200, msg: '验证码已发送', data: null }),
      })

      const res = await sendCode('user@test.com')

      expect(mockFetch).toHaveBeenCalledWith(
        '/api?action=email_send_code',
        expect.objectContaining({
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ email: 'user@test.com' }),
        }),
      )
      expect(res.code).toBe(200)
      expect(res.msg).toBe('验证码已发送')
    })

    it('网络错误应返回 code=500', async () => {
      mockFetch.mockRejectedValueOnce(new Error('network'))

      const res = await sendCode('user@test.com')

      expect(res.code).toBe(500)
      expect(res.msg).toContain('网络错误')
    })
  })

  describe('emailRegister', () => {
    it('应发送正确的注册参数', async () => {
      mockFetch.mockResolvedValueOnce({
        json: async () => ({ code: 200, msg: '注册成功', data: null }),
      })

      const res = await emailRegister('a@b.com', '123456', 'pass123', 'pass123', 'Nick')

      expect(JSON.parse(mockFetch.mock.calls[0][1].body)).toEqual({
        email: 'a@b.com',
        code: '123456',
        password: 'pass123',
        password2: 'pass123',
        nickname: 'Nick',
      })
      expect(res.code).toBe(200)
    })

    it('注册失败应透传错误码和消息', async () => {
      mockFetch.mockResolvedValueOnce({
        json: async () => ({ code: 400, msg: '该邮箱已注册', data: null }),
      })

      const res = await emailRegister('a@b.com', '123456', 'pass123', 'pass123', '')

      expect(res.code).toBe(400)
      expect(res.msg).toBe('该邮箱已注册')
    })
  })

  describe('emailLogin', () => {
    it('登录成功应返回 token 和 user', async () => {
      const mockData = {
        code: 200,
        msg: '登录成功',
        data: {
          token: 'jwt-token-abc',
          user: { id: 1, email: 'user@test.com', nickname: 'User' },
        },
      }
      mockFetch.mockResolvedValueOnce({ json: async () => mockData })

      const res = await emailLogin('user@test.com', 'password')

      expect(res.code).toBe(200)
      expect(res.data?.token).toBe('jwt-token-abc')
      expect(res.data?.user.id).toBe(1)
    })

    it('密码错误应返回 400', async () => {
      mockFetch.mockResolvedValueOnce({
        json: async () => ({ code: 400, msg: '邮箱或密码不正确', data: null }),
      })

      const res = await emailLogin('user@test.com', 'wrongpass')

      expect(res.code).toBe(400)
    })
  })

  describe('emailResetPassword', () => {
    it('应发送重置密码参数', async () => {
      mockFetch.mockResolvedValueOnce({
        json: async () => ({ code: 200, msg: '密码已重置成功', data: null }),
      })

      const res = await emailResetPassword('a@b.com', '123456', 'newpass', 'newpass')

      const body = JSON.parse(mockFetch.mock.calls[0][1].body)
      expect(body.email).toBe('a@b.com')
      expect(body.code).toBe('123456')
      expect(body.password).toBe('newpass')
      expect(body.password2).toBe('newpass')
      expect(res.code).toBe(200)
    })
  })

  describe('emailGetProfile', () => {
    it('应携带 token 请求用户信息', async () => {
      setEmailToken('profile-token')
      mockFetch.mockResolvedValueOnce({
        json: async () => ({
          code: 200,
          msg: '',
          data: {
            id: 1,
            email: 'user@test.com',
            nickname: 'User',
            status: 1,
            created_at: '2026-01-01 00:00:00',
            last_login: '2026-08-07 12:00:00',
            logs: [],
          },
        }),
      })

      const res = await emailGetProfile()

      const body = JSON.parse(mockFetch.mock.calls[0][1].body)
      expect(body.token).toBe('profile-token')
      expect(res.code).toBe(200)
      expect(res.data?.email).toBe('user@test.com')
    })

    it('token 过期应返回 401', async () => {
      setEmailToken('expired-token')
      mockFetch.mockResolvedValueOnce({
        json: async () => ({ code: 401, msg: '未登录或登录已过期', data: null }),
      })

      const res = await emailGetProfile()

      expect(res.code).toBe(401)
    })
  })
})
