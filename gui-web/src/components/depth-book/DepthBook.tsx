import { memo } from 'react'
import { useAppStore } from '../../store/app'
import { DepthRatioBar } from './DepthRatioBar'
import { DepthRow } from './DepthRow'

const LEVELS = 5
const UP_HEX = '#ef4444'
const DOWN_HEX = '#22c55e'

function DepthBookImpl() {
  const selected = useAppStore((s) => s.selected)

  // Six primitive selectors so children only re-render when their slice changes
  // (Immer preserves inner refs via structural sharing — see plan).
  const referencePrice = useAppStore((s) => {
    if (!selected) return undefined
    const sym = s.symbols[selected]
    return sym?.agg?.previousClose ?? sym?.ticker?.previousClose
  })
  const highPrice = useAppStore((s) =>
    selected ? s.symbols[selected]?.agg?.highPrice : undefined,
  )
  const lowPrice = useAppStore((s) =>
    selected ? s.symbols[selected]?.agg?.lowPrice : undefined,
  )
  const total = useAppStore((s) =>
    selected ? s.symbols[selected]?.agg?.total : undefined,
  )
  const bids = useAppStore((s) => {
    if (!selected) return undefined
    const sym = s.symbols[selected]
    return sym?.book?.bids ?? sym?.agg?.bids
  })
  const asks = useAppStore((s) => {
    if (!selected) return undefined
    const sym = s.symbols[selected]
    return sym?.book?.asks ?? sym?.agg?.asks
  })

  if (!selected) {
    return (
      <div className="flex flex-col bg-bg-panel">
        <div className="p-3 text-neutral-500 text-xs">尚未選擇標的</div>
      </div>
    )
  }

  const atBid = total?.tradeVolumeAtBid ?? 0
  const atAsk = total?.tradeVolumeAtAsk ?? 0
  const innerSum = atBid + atAsk
  const innerPct = innerSum === 0 ? 0 : (atBid / innerSum) * 100

  const totalBid = (bids ?? []).reduce((a, l) => a + l.size, 0)
  const totalAsk = (asks ?? []).reduce((a, l) => a + l.size, 0)
  const qtySum = totalBid + totalAsk
  const bidPct = qtySum === 0 ? 0 : (totalBid / qtySum) * 100

  const paddedBids = pad(bids ?? [], LEVELS)
  const paddedAsks = pad(asks ?? [], LEVELS)

  return (
    <div className="flex flex-col bg-bg-panel">
      <DepthRatioBar
        left={
          <>
            <span className="text-neutral-500">內盤</span>
            <span className="text-down">
              {innerSum > 0 ? `${innerPct.toFixed(0)}%` : '-%'}
            </span>
          </>
        }
        center="內外盤比"
        right={
          <>
            <span className="text-up">
              {innerSum > 0 ? `${(100 - innerPct).toFixed(0)}%` : '-%'}
            </span>
            <span className="text-neutral-500">外盤</span>
          </>
        }
        leftPct={innerPct}
        leftColor={DOWN_HEX}
        rightColor={UP_HEX}
        hasData={innerSum > 0}
      />

      <div className="grid grid-cols-2 gap-x-6 px-3 py-2 text-xs text-neutral-500 border-b border-bg-row">
        <div className="flex items-center justify-between">
          <span>委買量</span>
          <span>買價</span>
        </div>
        <div className="flex items-center justify-between">
          <span>賣價</span>
          <span>委賣量</span>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-x-6">
        <div className="flex flex-col">
          {paddedBids.map((l, i) => (
            <DepthRow
              key={`b${i}`}
              level={l}
              side="bid"
              referencePrice={referencePrice}
              highPrice={highPrice}
              lowPrice={lowPrice}
            />
          ))}
        </div>
        <div className="flex flex-col">
          {paddedAsks.map((l, i) => (
            <DepthRow
              key={`a${i}`}
              level={l}
              side="ask"
              referencePrice={referencePrice}
              highPrice={highPrice}
              lowPrice={lowPrice}
            />
          ))}
        </div>
      </div>

      <DepthRatioBar
        left={<span className="text-neutral-200">{formatTotal(totalBid)}</span>}
        right={<span className="text-neutral-200">{formatTotal(totalAsk)}</span>}
        leftPct={bidPct}
        leftColor={UP_HEX}
        rightColor={DOWN_HEX}
        hasData={qtySum > 0}
      />
    </div>
  )
}

function pad<T>(arr: T[], n: number): (T | null)[] {
  const out: (T | null)[] = arr.slice(0, n)
  while (out.length < n) out.push(null)
  return out
}

function formatTotal(n: number): string {
  return n.toLocaleString('en-US')
}

export const DepthBook = memo(DepthBookImpl)
