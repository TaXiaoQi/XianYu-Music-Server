import { adminApi, adminApiUpload } from './client'
import type { ApiResponse, UploadHandle } from './client'

// 版本管理（在线更新配置）相关接口

export interface DesktopVersionSavePayload {
  platform: string
  system: string
  arch: string
  pkg: string
  channel: 'stable' | 'beta'
  version: string
  original_version?: string
  download_url: string
  update_content: string
  enabled: number
  store_url: string
  file_name: string
  file_data: string
}

export interface DesktopVersionDeletePayload {
  platform: string
  system: string
  arch: string
  pkg: string
  version: string
}

export interface DesktopVersionListData {
  list: any[]
}

export interface BetaTesterListData {
  list: any[]
}

export interface BetaTesterDetail {
  tester?: { device_id: string; note?: string; created_at?: string }
  device?: {
    brand?: string
    model?: string
    os_version?: string
    app_version?: string
    architecture?: string
    machine_name?: string
  }
  accounts?: Array<{ nickname?: string; ciyuanxi_id: string; source?: string }>
}

export async function fetchDesktopVersions(): Promise<ApiResponse<DesktopVersionListData>> {
  return adminApi<DesktopVersionListData>('get_desktop_version')
}

export async function saveDesktopVersion(payload: DesktopVersionSavePayload): Promise<ApiResponse> {
  return adminApi('save_desktop_version', payload)
}

// 安装包独立上传：渠道弹窗确认后即调用，返回下发链接与进度句柄
export function uploadPackage(
  payload: { file_name: string; file_data: string },
  onProgress?: (percent: number) => void,
): UploadHandle {
  return adminApiUpload('upload_package', payload, onProgress)
}

export async function deleteDesktopVersion(payload: DesktopVersionDeletePayload): Promise<ApiResponse> {
  return adminApi('delete_desktop_version', payload)
}

export async function listBetaTesters(): Promise<ApiResponse<BetaTesterListData>> {
  return adminApi<BetaTesterListData>('list_beta_testers')
}

export async function createBetaTester(payload: { device_id: string; note: string }): Promise<ApiResponse> {
  return adminApi('add_beta_tester', payload)
}

export async function deleteBetaTester(payload: { device_id: string }): Promise<ApiResponse> {
  return adminApi('delete_beta_tester', payload)
}

export async function getBetaTesterDetail(payload: { device_id: string }): Promise<ApiResponse<BetaTesterDetail>> {
  return adminApi<BetaTesterDetail>('get_beta_tester_detail', payload)
}

export async function updateBetaTesterNote(payload: { device_id: string; note: string }): Promise<ApiResponse> {
  return adminApi('update_beta_tester_note', payload)
}
