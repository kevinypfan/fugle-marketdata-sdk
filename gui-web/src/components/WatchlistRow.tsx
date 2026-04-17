import { memo, useState } from 'react'

interface WatchlistRowProps {
  symbol: string
  name?: string
  price?: number
  changePercent?: number
  selected?: boolean
  onSelect?: () => void
  onMoveTop?: () => void
  onMoveBottom?: () => void
  onRemove?: () => void
}

function WatchlistRowImpl({
  symbol,
  name,
  price,
  changePercent,
  selected,
  onSelect,
  onMoveTop,
  onMoveBottom,
  onRemove,
}: WatchlistRowProps) {
  const [menuOpen, setMenuOpen] = useState(false)
  const cls =
    changePercent === undefined
      ? 'text-flat'
      : changePercent > 0
        ? 'text-up'
        : changePercent < 0
          ? 'text-down'
          : 'text-flat'

  return (
    <li
      className={`relative flex items-center justify-between px-3 py-2 cursor-pointer hover:bg-bg-hover ${selected ? 'bg-bg-hover' : ''}`}
      onClick={onSelect}
    >
      <div className="flex flex-col min-w-0">
        <span className="font-mono text-sm">{symbol}</span>
        <span className="text-[10px] text-neutral-500 truncate">{name ?? '—'}</span>
      </div>
      <div className="flex flex-col items-end">
        <span className={`font-mono text-sm ${cls}`}>{price?.toFixed(2) ?? '—'}</span>
        <span className={`text-[10px] ${cls}`}>
          {changePercent !== undefined ? `${changePercent.toFixed(2)}%` : '—'}
        </span>
      </div>

      <button
        type="button"
        className="ml-2 px-1 text-neutral-500 hover:text-neutral-200"
        onClick={(e) => {
          e.stopPropagation()
          setMenuOpen((o) => !o)
        }}
        aria-label="more"
      >
        ⋯
      </button>

      {menuOpen && (
        <div
          className="absolute right-2 top-10 z-10 w-32 rounded border border-bg-row bg-bg-panel text-xs shadow"
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
