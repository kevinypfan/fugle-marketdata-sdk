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

  return (
    <div className="flex-1 min-w-0 px-3 py-1.5">
      <div className="flex items-center justify-between gap-2">
        <span className="text-xs text-neutral-300">{name}</span>
        <div className="flex items-center gap-2 shrink-0">
          <span className={`font-mono text-xs ${hasChange ? toneClass : 'text-neutral-500'}`}>
            {hasChange
              ? `${arrow(tone)}${Math.abs(change).toFixed(2)}`
              : '—'}
          </span>
          <Sparkline data={history} tone={tone} width={160} height={22} />
        </div>
      </div>
      <div className="flex items-baseline justify-between gap-2">
        <span className={`font-mono text-base font-semibold ${toneClass}`}>
          {value !== undefined ? value.toFixed(2) : '—'}
        </span>
        <span className={`font-mono text-xs ${hasChange ? toneClass : 'text-neutral-500'}`}>
          {changePct !== undefined
            ? `${arrow(tone)}${Math.abs(changePct).toFixed(2)}%`
            : '—'}
        </span>
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
