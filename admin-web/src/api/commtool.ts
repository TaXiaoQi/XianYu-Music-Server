// 通信工具（commtool）+ Webhook 通知域 API
import { adminApi } from './client'
import type { ApiResponse } from './client'

// ===== 服务状态与服务配置 =====

export interface WsClientConfigData {
  url: string
  auto_reconnect: boolean
  reconnect_interval: string
  heartbeat_interval: string
}

export interface CommStatus {
  server_running: boolean
  server_port: number
  server_enabled?: boolean
  server_port_config?: number
  ws_server_count: number
  sse_count?: number
  token_enabled?: boolean
  ws_client: { url: string; connected_at: string } | null
  ws_client_config?: WsClientConfigData
}

export async function getCommStatus(): Promise<ApiResponse<CommStatus>> {
  return adminApi<CommStatus>('comm_get_status')
}

export async function saveCommServiceConfig(params: { enabled: boolean; port: number }): Promise<ApiResponse> {
  return adminApi('comm_service_config', params)
}

// ===== HTTP 客户端 =====

export interface HttpLog {
  time: string
  method: string
  path: string
  query: string
  headers: Record<string, string>
  body: string
}

export interface HttpClientResult {
  status: number
  headers: Record<string, string>
  body: string
  elapsed_ms: number
}

export async function getCommHttpLogs(params: { limit: number }): Promise<ApiResponse<HttpLog[]>> {
  return adminApi<HttpLog[]>('comm_http_logs', params)
}

export async function clearCommHttpLogs(): Promise<ApiResponse> {
  return adminApi('comm_http_clear')
}

export async function commHttpClient(params: { url: string; method: string; headers: string; body: string }): Promise<ApiResponse<HttpClientResult>> {
  return adminApi<HttpClientResult>('comm_http_client', params)
}

// ===== SSE 推送 =====

export async function commSsePush(params: { message: string }): Promise<ApiResponse> {
  return adminApi('comm_sse_push', params)
}

// ===== WS 服务端 =====

export interface WsServerClient {
  id: string
  addr: string
  connected_at: string
  events?: string[]
}

export async function getCommWsServerClients(): Promise<ApiResponse<WsServerClient[]>> {
  return adminApi<WsServerClient[]>('comm_ws_server_list')
}

export async function commWsServerSend(params: { id: string; message: string }): Promise<ApiResponse> {
  return adminApi('comm_ws_server_send', params)
}

export async function commWsServerBroadcast(params: { message: string }): Promise<ApiResponse> {
  return adminApi('comm_ws_server_broadcast', params)
}

// ===== WS 客户端 =====

export interface WsLog {
  time: string
  direction: 'in' | 'out'
  client: string
  type: string
  data: string
}

export async function getCommWsClientConfig(): Promise<ApiResponse<WsClientConfigData>> {
  return adminApi<WsClientConfigData>('comm_ws_client_config')
}

export async function saveCommWsClientConfig(params: WsClientConfigData): Promise<ApiResponse> {
  return adminApi('comm_ws_client_save_config', params)
}

export async function commWsClientConnect(params: { url: string }): Promise<ApiResponse> {
  return adminApi('comm_ws_client_connect', params)
}

export async function commWsClientDisconnect(): Promise<ApiResponse> {
  return adminApi('comm_ws_client_disconnect')
}

export async function commWsClientSend(params: { message: string }): Promise<ApiResponse> {
  return adminApi('comm_ws_client_send', params)
}

export async function getCommWsLogs(params: { limit: number }): Promise<ApiResponse<WsLog[]>> {
  return adminApi<WsLog[]>('comm_ws_client_logs', params)
}

export async function clearCommWsLogs(): Promise<ApiResponse> {
  return adminApi('comm_ws_clear')
}

// ===== 连接鉴权 =====

export interface CommAuthConfig {
  token: string
  token_enabled: boolean
}

export async function getCommAuthConfig(): Promise<ApiResponse<CommAuthConfig>> {
  return adminApi<CommAuthConfig>('comm_auth_config')
}

export async function saveCommAuthConfig(params: { token: string }): Promise<ApiResponse> {
  return adminApi('comm_auth_save_config', params)
}

// ===== 外部客户端管理 =====

export interface CommClient {
  id: number
  name: string
  type: string
  url: string
  events: string
  enabled: boolean
  created_at: string
}

export async function addCommClient(params: { name: string; url: string; type: string; events: string }): Promise<ApiResponse> {
  return adminApi('comm_client_add', params)
}

export async function getCommClients(): Promise<ApiResponse<CommClient[]>> {
  return adminApi<CommClient[]>('comm_client_list')
}

export async function deleteCommClient(params: { id: number }): Promise<ApiResponse> {
  return adminApi('comm_client_delete', params)
}

export async function toggleCommClient(params: { id: number; enabled: boolean }): Promise<ApiResponse> {
  return adminApi('comm_client_toggle', params)
}

// ===== Webhook 通知 =====

export interface WebhookConfig {
  enabled: boolean
  url: string
  method: string
  headers: string
  body_template: string
  modules: Record<string, boolean>
}

export interface WebhookTestResult {
  status: number
  body: string
}

export async function getWebhookConfig(): Promise<ApiResponse<WebhookConfig>> {
  return adminApi<WebhookConfig>('get_webhook_config')
}

export async function saveWebhookConfig(params: {
  enabled: boolean
  url: string
  method: string
  headers: string
  body_template: string
  modules: Record<string, boolean>
}): Promise<ApiResponse> {
  return adminApi('save_webhook_config', params)
}

export async function testWebhook(params: { url: string; method: string; headers: string; body_template: string }): Promise<ApiResponse<WebhookTestResult>> {
  return adminApi<WebhookTestResult>('test_webhook', params)
}
