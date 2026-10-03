import { adminApi } from './client'
import type { ApiResponse } from './client'

export interface Feedback {
  id: number
  ciyuanxi_id: string
  nickname: string
  title: string
  content: string
  status: string
  admin_reply: string | null
  error_logs?: string | null
  all_logs?: string | null
  log_meta?: string | null
  error_logs_chars?: number
  all_logs_chars?: number
  has_error_logs?: number | string | boolean
  has_all_logs?: number | string | boolean
  replied_at: string | null
  replied_by: string
  assignee: string
  resolve_note: string | null
  ip: string
  created_at: string
  updated_at: string
  [key: string]: any
}

export interface FbStats {
  total: number
  pending: number
  processing: number
  resolved: number
  rejected: number
}

export interface FeedbackLimit {
  feedback_daily_limit: number
}

export interface FeedbackAdminStatsRow {
  admin_name: string
  total: number
  processing: number
  resolved: number
  rejected: number
  pending: number
}

export interface FeedbackListData {
  list: Feedback[]
  stats: FbStats
}

export interface FeedbackResolveData {
  resolved?: boolean
  completed?: number
  total?: number
}

export interface CreateFeedbackPayload {
  feedback_type: string
  platform: string
  title: string
  content: string
  images: string[]
  notify_external: number
}

export async function listFeedback(statusFilter: string, sort: string): Promise<ApiResponse<FeedbackListData>> {
  return adminApi<FeedbackListData>('list_feedback', {
    status_filter: statusFilter,
    sort,
  })
}

export async function getFeedbackDetail(id: number): Promise<ApiResponse<Feedback>> {
  return adminApi<Feedback>('get_feedback_detail', { id })
}

export async function updateFeedbackStatus(id: number, status: string, reason?: string): Promise<ApiResponse> {
  const data: Record<string, any> = { id, status }
  if (reason !== undefined) data.reason = reason
  return adminApi('update_feedback_status', data)
}

export async function createFeedback(payload: CreateFeedbackPayload): Promise<ApiResponse> {
  return adminApi('create_feedback', payload)
}

export async function claimFeedback(id: number): Promise<ApiResponse> {
  return adminApi('claim_feedback', { id })
}

export async function addCollaborator(id: number): Promise<ApiResponse> {
  return adminApi('add_collaborator', { id })
}

export async function respondCollabRequest(requestId: number, approve: boolean): Promise<ApiResponse> {
  return adminApi('respond_collab_request', { request_id: requestId, approve: approve ? 1 : 0 })
}

export async function pollCollabRequests(): Promise<ApiResponse<any>> {
  return adminApi<any>('poll_collab_requests')
}

export async function pollAdminNotifications(): Promise<ApiResponse<any>> {
  return adminApi<any>('poll_admin_notifications')
}

export async function markNotificationsRead(ids: number[]): Promise<ApiResponse> {
  return adminApi('mark_notifications_read', { ids })
}

export async function abandonFeedback(id: number): Promise<ApiResponse> {
  return adminApi('abandon_feedback', { id })
}

export async function resolveFeedback(id: number, note: string, images: string[]): Promise<ApiResponse<FeedbackResolveData>> {
  return adminApi('resolve_feedback', { id, note, images })
}

export async function collaboratorComplete(id: number, note: string, images: string[]): Promise<ApiResponse<FeedbackResolveData>> {
  return adminApi('collaborator_complete', { id, note, images })
}

export async function resolveBetaApplication(id: number, note: string, deviceNote: string): Promise<ApiResponse<{ device_added?: boolean }>> {
  return adminApi('resolve_beta_application', { id, note, device_note: deviceNote })
}

export async function getFeedbackAdminStats(): Promise<ApiResponse<{ list: FeedbackAdminStatsRow[]; grand_total: number }>> {
  return adminApi<{ list: FeedbackAdminStatsRow[]; grand_total: number }>('feedback_admin_stats')
}

export async function getFeedbackLimit(): Promise<ApiResponse<FeedbackLimit>> {
  return adminApi<FeedbackLimit>('get_feedback_limit')
}

export async function updateFeedbackLimit(limit: number): Promise<ApiResponse<FeedbackLimit>> {
  return adminApi<FeedbackLimit>('update_feedback_limit', { feedback_daily_limit: limit })
}

export async function batchDeleteFeedback(ids: number[]): Promise<ApiResponse<{ deleted?: number }>> {
  return adminApi<{ deleted?: number }>('batch_delete_feedback', { ids })
}

export async function listRecycleBin(): Promise<ApiResponse<{ list: any[] }>> {
  return adminApi<{ list: any[] }>('list_recycle_bin')
}

export async function restoreFeedback(id: number): Promise<ApiResponse> {
  return adminApi('restore_feedback', { id })
}
