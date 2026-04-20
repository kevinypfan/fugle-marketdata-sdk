import { useEffect, useState } from 'react'
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

/** Returns `(Ns 前)` style staleness label + color coded threshold.
 *  Green <5s, amber 5-30s, red >30s. Undefined if never received / not
 *  connected. */
function stalenessLabel(
  lastAt: number | null,
  now: number,
  connected: boolean,
): { text: string; color: string } | null {
  if (!connected) return null
  if (lastAt == null) {
    return { text: '無資料', color: 'text-neutral-600' }
  }
  const ageSec = Math.max(0, Math.floor((now - lastAt) / 1000))
  const text = ageSec === 0 ? '< 1s 前' : `${ageSec}s 前`
  const color =
    ageSec < 5 ? 'text-neutral-500' : ageSec < 30 ? 'text-yellow-500' : 'text-red-400'
  return { text, color }
}

interface StatusBarProps {
  onOpenSettings: () => void
}

export function StatusBar({ onOpenSettings }: StatusBarProps) {
  const stockConn = useAppStore((s) => s.conn.stock)
  const futoptConn = useAppStore((s) => s.conn.futopt)
  const stockLast = useAppStore((s) => s.dataStaleness.stock)
  const futoptLast = useAppStore((s) => s.dataStaleness.futopt)

  // Tick once per second so the "Ns 前" label ages visibly even when no
  // data arrives — the whole point of this indicator is to make silence
  // observable. Cost is ~1 rerender/sec of a footer.
  const [now, setNow] = useState(() => Date.now())
  useEffect(() => {
    const id = setInterval(() => setNow(Date.now()), 1000)
    return () => clearInterval(id)
  }, [])

  return (
    <div className="flex items-center justify-between h-full px-3 text-xs bg-bg-panel">
      <div className="flex items-center gap-3">
        <MarketStatus label="股票" conn={stockConn} lastAt={stockLast} now={now} />
        <span className="text-neutral-700">|</span>
        <MarketStatus label="期貨" conn={futoptConn} lastAt={futoptLast} now={now} />
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

function MarketStatus({
  label,
  conn,
  lastAt,
  now,
}: {
  label: string
  conn: ConnectionState | null
  lastAt: number | null
  now: number
}) {
  const { text, color } = connLabel(conn)
  const staleness = stalenessLabel(lastAt, now, conn?.state === 'connected')
  return (
    <span className="flex items-center gap-1.5">
      <span className="text-neutral-500">{label}</span>
      <span className={color}>{text}</span>
      {staleness && <span className={staleness.color}>({staleness.text})</span>}
    </span>
  )
}
