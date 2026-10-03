import type { Ref } from 'vue'
import { showToast, getAdminUser } from '@/api/client'
import { webConfirm, webInfo } from '@/utils/webDialog'
import {
  claimFeedback as claimFeedbackApi,
  addCollaborator,
  respondCollabRequest as respondCollabRequestApi,
  pollCollabRequests,
  pollAdminNotifications,
  markNotificationsRead,
  abandonFeedback as abandonFeedbackApi,
} from '@/api/feedback'
import type { Feedback } from '@/api/feedback'
import { collaboratorsOf, completedOf } from './shared'

// 认领 / 协同 / 放弃 / 协同请求与通知轮询
export function useFeedbackCollab(options: { feedbackList: Ref<Feedback[]>; loadList: () => Promise<void> }) {
  const { feedbackList, loadList } = options

  const currentAdminName = getAdminUser()?.username || ''
  function isMineFeedback(item: Feedback): boolean {
    return !!item.assignee && item.assignee === currentAdminName
  }
  function isCollaborator(item: Feedback): boolean {
    return collaboratorsOf(item).includes(currentAdminName)
  }
  function isParticipant(item: Feedback): boolean {
    return isMineFeedback(item) || isCollaborator(item)
  }
  function getCollabCount(item: Feedback): number {
    return collaboratorsOf(item).length
  }
  function getCompletedDisplay(item: Feedback): string {
    const done = completedOf(item).length
    const total = 1 + collaboratorsOf(item).length
    return `完成 ${done}/${total}`
  }

  async function claimFeedback(id: number) {
    const item = feedbackList.value.find(f => f.id === id)
    const isTransfer = item?.status === 'processing'
    const ok = await webConfirm(isTransfer ? '确认将该反馈转认领到自己名下？认领后问题将转移到您的名下。' : '确认认领该反馈？认领后将自动划入您的名下并移入处理中。', {
      title: '认领反馈',
      confirmText: '认领',
    })
    if (!ok) return
    const res = await claimFeedbackApi(id)
    if (res.code === 200) {
      showToast(isTransfer ? '已转认领到自己名下' : '认领成功，已置为处理中', 'success')
      await loadList()
    } else {
      showToast(res.msg || '认领失败')
    }
  }

  async function requestCollaborate(item: Feedback) {
    const ok = await webConfirm(`确认申请协同处理该反馈？需认领人 ${item.assignee} 弹窗同意后方可加入。`, {
      title: '申请协同',
      confirmText: '申请',
    })
    if (!ok) return
    const res = await addCollaborator(item.id)
    if (res.code === 200) {
      showToast('协同请求已发送，等待认领人确认', 'success')
    } else {
      showToast(res.msg || '操作失败')
    }
  }

  async function respondCollabRequest(request: any, approve: boolean) {
    const res = await respondCollabRequestApi(request.id, approve)
    if (res.code === 200) {
      showToast(approve ? `已同意 ${request.requester} 协同处理` : `已拒绝 ${request.requester} 的协同请求`, 'success')
      await loadList()
    } else {
      showToast(res.msg || '操作失败')
    }
  }

  // 轮询协同请求与通知（转认告知 / 协同结果），8s 一次由视图 onMounted 驱动
  let alertProcessing = false
  async function pollFeedbackAlerts() {
    if (alertProcessing) return
    alertProcessing = true
    try {
      const reqRes = await pollCollabRequests()
      if (reqRes.code === 200 && reqRes.data?.list?.length) {
        for (const req of reqRes.data.list) {
          const approve = await webConfirm(`${req.requester} 请求协同处理反馈「${req.feedback_title || '无标题'}」，是否同意？`, {
            title: '协同请求',
            confirmText: '同意',
            cancelText: '拒绝',
          })
          await respondCollabRequest(req, approve)
        }
      }
      const notifRes = await pollAdminNotifications()
      if (notifRes.code === 200 && notifRes.data?.list?.length) {
        const list = notifRes.data.list
        for (const n of list) {
          await webInfo(n.content, { title: '反馈通知' })
        }
        await markNotificationsRead(list.map((n: any) => n.id))
        await loadList()
      }
    } catch {
    } finally {
      alertProcessing = false
    }
  }

  async function abandonFeedback(id: number) {
    const ok = await webConfirm('确认放弃认领该反馈？仅放弃自己的账号，其他参与人不受影响。', {
      title: '放弃认领',
      confirmText: '放弃',
      danger: true,
    })
    if (!ok) return
    const res = await abandonFeedbackApi(id)
    if (res.code === 200) {
      showToast(res.msg || '已放弃', 'success')
      await loadList()
    } else {
      showToast(res.msg || '操作失败')
    }
  }

  return {
    isCollaborator,
    isParticipant,
    getCollabCount,
    getCompletedDisplay,
    claimFeedback,
    requestCollaborate,
    respondCollabRequest,
    pollFeedbackAlerts,
    abandonFeedback,
  }
}
