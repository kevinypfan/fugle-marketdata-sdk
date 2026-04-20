import { memo } from 'react'
import { useAppStore } from '../store/app'
import { INDEX_LABEL, INDEX_SYMBOLS } from '../types/events'
import { TONE_CLASS, toneFromDiff, type Tone } from '../lib/tone'
import { Sparkline } from './Sparkline'

function arrow(tone: Tone): string {
  return tone === 'up' ? '▲' : tone === 'down' ? '▼' : ''
}

function IndicesCellImpl({ symbol }: { symbol: string }) {
  const slice = useAppStore((s) => s.indices[symbol])
  const value = slice?.value
  // Fall back to the oldest in-session value when previousClose isn't seeded —
  // Fugle REST doesn't serve indices history to this SDK, so we approximate
  // "change since open" with "change since app started" during trading hours.
  const prev = slice?.previousClose ?? slice?.history?.[0]
  const history = slice?.history ?? []

  const change = value !== undefined && prev !== undefined ? value - prev : undefined
  const changePct =
    change !== undefined && prev && prev !== 0 ? (change / prev) * 100 : undefined
  const tone = toneFromDiff(change)
  const toneClass = TONE_CLASS[tone]

  const name = INDEX_LABEL[symbol] ?? symbol
  const hasChange = change !== undefined

  const changeClass = hasChange ? toneClass : 'text-neutral-500'

  // 3-column layout mirroring fugle-web: name/value stack on the left,
  // change/percent stacked compact in the middle, sparkline spanning the
  // full cell height on the right. Avoids the prior 2-row layout where
  // the percent got pushed to its own row below the chart.
  return (
    <div className="flex-1 min-w-0 px-3 py-1.5 flex items-stretch gap-3">
      <div className="flex-1 min-w-0 flex flex-col justify-between">
        <span className="text-xs text-neutral-300">{name}</span>
        <span className={`font-mono text-base font-semibold ${toneClass}`}>
          {value !== undefined ? value.toFixed(2) : '—'}
        </span>
      </div>
      <div className="shrink-0 flex flex-col justify-between items-end font-mono text-xs">
        <span className={changeClass}>
          {hasChange ? `${arrow(tone)}${Math.abs(change).toFixed(2)}` : '—'}
        </span>
        <span className={changeClass}>
          {changePct !== undefined
            ? `${arrow(tone)}${Math.abs(changePct).toFixed(2)}%`
            : '—'}
        </span>
      </div>
      <div className="shrink-0 flex items-center">
        <Sparkline data={history} tone={tone} width={160} height={40} />
      </div>
    </div>
  )
}

const IndicesCell = memo(IndicesCellImpl)

function IndicesBarImpl() {
  return (
    <div className="flex items-stretch border-b border-bg-row bg-bg-panel">
      {INDEX_SYMBOLS.map((symbol, i) => (
        <div
          key={symbol}
          className={`flex-1 min-w-0 ${i > 0 ? 'border-l border-bg-row' : ''}`}
        >
          <IndicesCell symbol={symbol} />
        </div>
      ))}
    </div>
  )
}

export const IndicesBar = memo(IndicesBarImpl)
