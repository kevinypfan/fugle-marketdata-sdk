import { memo, useMemo, useState } from 'react'
import { useAppStore } from '../store/app'
import { Sparkline } from './Sparkline'
import { TONE_CLASS, toneFromDiff } from '../lib/tone'

interface WatchlistRowProps {
  symbol: string
  selected?: boolean
  onSelect?: () => void
  onMoveTop?: () => void
  onMoveBottom?: () => void
  onRemove?: () => void
}

function WatchlistRowImpl({
  symbol,
  selected,
  onSelect,
  onMoveTop,
  onMoveBottom,
  onRemove,
}: WatchlistRowProps) {
  const [menuOpen, setMenuOpen] = useState(false)

  const ticker = useAppStore((s) => s.symbols[symbol]?.ticker)
  const lastPrice = useAppStore(
    (s) => s.symbols[symbol]?.agg?.lastPrice ?? s.symbols[symbol]?.tape?.[0]?.price,
  )
  const prevClose = useAppStore(
    (s) =>
      s.symbols[symbol]?.ticker?.previousClose ?? s.symbols[symbol]?.agg?.previousClose,
  )
  const candles = useAppStore((s) => s.symbols[symbol]?.candles)

  const change =
    lastPrice !== undefined && prevClose !== undefined ? lastPrice - prevClose : undefined
  const changePct =
    change !== undefined && prevClose && prevClose !== 0 ? (change / prevClose) * 100 : undefined
  const tone = toneFromDiff(change)

  // Stable ref: closes only re-derives when candles ref changes (on CandleTick,
  // ~1/min), not on Aggregate ticks (~10Hz). Keeps Sparkline memo hitting.
  const closes = useMemo(() => candles?.map((c) => c.close) ?? [], [candles])

  const name = ticker?.name
  const marketLabel =
    ticker?.market === 'TSE' ? '市' : ticker?.market === 'OTC' ? '櫃' : ''

  return (
    <li
      onClick={onSelect}
      className={`group relative flex px-3 py-2 border-b border-bg-row/30 cursor-pointer hover:bg-bg-hover ${
        selected ? 'bg-bg-hover' : ''
      }`}
    >
      <div className="flex-1 min-w-0">
        <div className="flex items-center justify-between gap-2">
          <span className="text-sm truncate">{name ?? '—'}</span>
          <div className="flex items-center gap-2 shrink-0">
            <span className={`font-mono text-sm ${TONE_CLASS[tone]}`}>
              {lastPrice !== undefined ? lastPrice.toFixed(2) : '—'}
            </span>
            <Sparkline data={closes} tone={tone} />
          </div>
        </div>

        <div className="flex items-center justify-between gap-2 text-[11px] font-mono">
          <span className="text-neutral-500 truncate">
            {symbol}{' '}
            {marketLabel && <span className="text-neutral-400">{marketLabel}</span>}
          </span>
          <div className={`flex gap-2 ${TONE_CLASS[tone]}`}>
            <span>{signed(change)}</span>
            <span>
              {signed(changePct)}
              {changePct !== undefined ? '%' : ''}
            </span>
          </div>
        </div>

        {(ticker?.isAttention || ticker?.isDisposition) && (
          <div className="mt-1 flex gap-1">
            {ticker?.isAttention && <Badge>注</Badge>}
            {ticker?.isDisposition && <Badge>處</Badge>}
          </div>
        )}
      </div>

      <button
        type="button"
        onClick={(e) => {
          e.stopPropagation()
          setMenuOpen((o) => !o)
        }}
        className="absolute right-1 top-1 px-1 text-xs text-neutral-500 opacity-0 group-hover:opacity-100 hover:text-neutral-200"
        aria-label="more"
      >
        ⋯
      </button>

      {menuOpen && (
        <div
          className="absolute right-2 top-8 z-10 w-32 rounded border border-bg-row bg-bg-panel text-xs shadow"
          onClick={(e) => e.stopPropagation()}
        >
          <button
            className="block w-full px-3 py-1.5 text-left hover:bg-bg-hover"
            onClick={() => {
              onMoveTop?.()
              setMenuOpen(false)
            }}
          >
            置頂
          </button>
          <button
            className="block w-full px-3 py-1.5 text-left hover:bg-bg-hover"
            onClick={() => {
              onMoveBottom?.()
              setMenuOpen(false)
            }}
          >
            置底
          </button>
          <button
            className="block w-full px-3 py-1.5 text-left text-red-400 hover:bg-bg-hover"
            onClick={() => {
              onRemove?.()
              setMenuOpen(false)
            }}
          >
            移除
          </button>
        </div>
      )}
    </li>
  )
}

export const WatchlistRow = memo(WatchlistRowImpl)

function signed(n: number | undefined): string {
  if (n === undefined || Number.isNaN(n)) return '—'
  const s = n.toFixed(2)
  return n > 0 ? `+${s}` : s
}

function Badge({ children }: { children: React.ReactNode }) {
  return (
    <span className="px-1 py-0.5 text-[10px] bg-neutral-500/20 text-neutral-300 rounded">
      {children}
    </span>
  )
}
