import { memo } from 'react'
import { useAppStore } from '../store/app'
import type { PriceLevel } from '../types/market'

const LEVELS = 5

function maxSize(levels: PriceLevel[]): number {
  let max = 0
  for (const l of levels) if (l.size > max) max = l.size
  return max || 1
}

function pad<T>(arr: T[], n: number): (T | null)[] {
  const out: (T | null)[] = arr.slice(0, n)
  while (out.length < n) out.push(null)
  return out
}

function DepthBookImpl() {
  const selected = useAppStore((s) => s.selected)
  const book = useAppStore((s) => (selected ? s.symbols[selected]?.book : undefined))
  const agg = useAppStore((s) => (selected ? s.symbols[selected]?.agg : undefined))

  // BookSnap channel takes priority; Aggregate carries depth on every tick.
  const bidsSrc = book?.bids ?? agg?.bids ?? []
  const asksSrc = book?.asks ?? agg?.asks ?? []

  const bids = pad(bidsSrc, LEVELS)
  const asks = pad(asksSrc, LEVELS)
  const max = Math.max(maxSize(bidsSrc), maxSize(asksSrc))

  return (
    <div className="flex flex-col h-full bg-bg-panel">
      <header className="px-3 py-2 text-xs font-medium text-neutral-400 border-b border-bg-row">
        五檔
      </header>
      <div className="flex-1 grid grid-cols-2">
        {/* bids — left side, larger size = wider bar from right */}
        <div className="flex flex-col">
          {bids.map((l, i) => (
            <DepthCell key={`b${i}`} level={l} max={max} side="bid" />
          ))}
        </div>
        {/* asks — right side, larger size = wider bar from left */}
        <div className="flex flex-col">
          {asks.map((l, i) => (
            <DepthCell key={`a${i}`} level={l} max={max} side="ask" />
          ))}
        </div>
      </div>
    </div>
  )
}

interface DepthCellProps {
  level: PriceLevel | null
  max: number
  side: 'bid' | 'ask'
}

function DepthCell({ level, max, side }: DepthCellProps) {
  const pct = level ? Math.min(100, Math.max(2, (level.size / max) * 100)) : 0
  const barColor = side === 'bid' ? 'bg-up/20' : 'bg-down/20'
  const align = side === 'bid' ? 'right-0 left-auto' : 'left-0 right-auto'
  const textColor = side === 'bid' ? 'text-up' : 'text-down'
  const layout = side === 'bid' ? 'flex-row-reverse' : 'flex-row'

  return (
    <div className="relative flex-1 border-b border-bg-row last:border-0 overflow-hidden">
      {level && (
        <div
          className={`absolute top-0 bottom-0 ${align} ${barColor}`}
          style={{ width: `${pct}%` }}
        />
      )}
      <div className={`relative flex ${layout} items-center justify-between h-full px-3 text-xs font-mono`}>
        <span className={textColor}>{level?.price.toFixed(2) ?? '—'}</span>
        <span className="text-neutral-400">{level?.size ?? '—'}</span>
      </div>
    </div>
  )
}

export const DepthBook = memo(DepthBookImpl)
