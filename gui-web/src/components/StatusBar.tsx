import { useAppStore } from '../store/app'

function connLabel(conn: ReturnType<typeof useAppStore.getState>['conn']): {
  text: string
  color: string
} {
  if (!conn) return { text: '未連線', color: 'text-neutral-500' }
  switch (conn.state) {
    case 'connecting':
      return { text: '連線中…', color: 'text-yellow-400' }
    case 'connected':
      return { text: '已連線', color: 'text-emerald-400' }
    case 'reconnecting':
      return { text: `重連 #${conn.attempt}`, color: 'text-yellow-400' }
    case 'disconnected':
      return { text: `斷線：${conn.reason}`, color: 'text-neutral-400' }
    case 'failed':
      return { text: `失敗：${conn.message}`, color: 'text-red-400' }
  }
}

export function StatusBar() {
  const conn = useAppStore((s) => s.conn)
  const taiex = useAppStore((s) => s.taiex)
  const { text, color } = connLabel(conn)

  return (
    <div className="flex items-center justify-between h-full px-3 text-xs bg-bg-panel">
      <span className={color}>{text}</span>
      <span className="font-mono text-neutral-300">
        TAIEX {taiex ? taiex.value.toFixed(2) : '—'}
      </span>
    </div>
  )
}
