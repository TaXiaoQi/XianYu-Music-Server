import { ref } from 'vue'
import { getUserPlaylists, deleteUserSyncPlaylist } from '@/api/users'
import type { User } from '@/api/users'
import { showToast } from '@/api/client'
import { webConfirm } from '@/utils/webDialog'

// 歌单类型展示
export function formatPlaylistType(t: string | undefined): string {
  switch (t) {
    case 'local': return '本地'
    case 'online': return '在线'
    case 'mixed': return '混合'
    default: return '-'
  }
}

// 查看用户歌单弹窗
export function usePlaylistsModal() {
  const showPlaylistsModal = ref(false)
  const playlistsLoading = ref(false)
  const playlistsData = ref<any>({})
  const playlistsUser = ref<User | null>(null)
  const playlistDeleting = ref(-1)

  async function viewPlaylists(u: User) {
    playlistsUser.value = u
    showPlaylistsModal.value = true
    playlistsLoading.value = true
    playlistsData.value = {}
    const res = await getUserPlaylists({ user_id: u.id })
    playlistsLoading.value = false
    if (res.code === 200 && res.data) {
      playlistsData.value = res.data
    } else {
      showToast(res.msg || '加载失败')
    }
  }

  // 删除快照中的单个歌单
  async function removePlaylist(p: any, index: number) {
    const u = playlistsUser.value
    if (!u || playlistDeleting.value >= 0) return
    const ok = await webConfirm(
      `确定删除歌单「${p.name || '(未命名)'}」？将从服务器的同步快照中移除该歌单（含 ${p.songCount || 0} 首歌曲），该用户客户端下次全量同步后可能恢复。`,
      { title: '删除歌单', confirmText: '确认删除' }
    )
    if (!ok) return
    playlistDeleting.value = index
    const res = await deleteUserSyncPlaylist({ user_id: u.id, index, name: p.name || '' })
    playlistDeleting.value = -1
    if (res.code === 200) {
      showToast('删除成功', 'success')
      await viewPlaylists(u)
    } else {
      showToast(res.msg || '删除失败')
    }
  }

  return { showPlaylistsModal, playlistsLoading, playlistsData, viewPlaylists, removePlaylist, playlistDeleting, formatPlaylistType }
}
