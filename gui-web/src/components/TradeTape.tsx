import { memo } from 'react'
import { useAppStore } from '../store/app'

const LARGE_ORDER_THRESHOLD = 100

function fmtTime(t: number): string {
  const d = new Date(t)
  return `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}:${String(d.getSeconds()).padStart(2, '0')}`
}

function TradeTapeImpl() {
  const selected = useAppStore((s) => s.selected)
  // NOTE: never allocate a fresh array inside the selector — it makes
  // useSyncExternalStore think state changed every tick and infinite-loops.
  const tape = useAppStore((s) => (selected ? s.symbols[selected]?.tape : undefined)) ?? []

  return (
    <div className="flex flex-col h-full bg-bg-panel">
      <header className="px-3 py-2 text-xs font-medium text-neutral-400 border-b border-bg-row">
        成交明細
      </header>
      <ul className="flex-1 overflow-y-auto text-xs font-mono">
        {tape.map((t, i) => {
          const tone =
            t.direction === 1 ? 'bg-up/15 text-up'
            : t.direction === -1 ? 'bg-down/15 text-down'
            : 'text-neutral-300'
          const big = t.size >= LARGE_ORDER_THRESHOLD ? 'font-bold' : ''
          return (
            <li
              key={`${t.time}-${i}`}
              className={`flex items-center justify-between px-3 py-1 border-b border-bg-row/50 ${tone}`}
            >
              <span className="text-neutral-500">{fmtTime(t.time)}</span>
              <span className={big}>{t.price.toFixed(2)}</span>
              <span className={`${big} w-10 text-right`}>{t.size}</span>
            </li>
          )
        })}
        {tape.length === 0 && (
          <li className="px-3 py-6 text-center text-neutral-500">尚無成交</li>
        )}
      </ul>
    </div>
  )
}

export const TradeTape = memo(TradeTapeImpl)
