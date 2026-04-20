import { useAppStore } from '../store/app'
import type { ConnectionState } from '../types/market'

function connLabel(conn: ConnectionState | null): { text: string; color: string } {
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

interface StatusBarProps {
  onOpenSettings: () => void
}

export function StatusBar({ onOpenSettings }: StatusBarProps) {
  const stockConn = useAppStore((s) => s.conn.stock)
  const futoptConn = useAppStore((s) => s.conn.futopt)

  return (
    <div className="flex items-center justify-between h-full px-3 text-xs bg-bg-panel">
      <div className="flex items-center gap-3">
        <MarketStatus label="股票" conn={stockConn} />
        <span className="text-neutral-700">|</span>
        <MarketStatus label="期貨" conn={futoptConn} />
      </div>
      <button
        type="button"
        onClick={onOpenSettings}
        className="text-neutral-400 hover:text-neutral-200"
        title="設定"
      >
        設定
      </button>
    </div>
  )
}

function MarketStatus({ label, conn }: { label: string; conn: ConnectionState | null }) {
  const { text, color } = connLabel(conn)
  return (
    <span className="flex items-center gap-1.5">
      <span className="text-neutral-500">{label}</span>
      <span className={color}>{text}</span>
    </span>
  )
}
