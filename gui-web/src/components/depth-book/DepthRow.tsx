import { memo } from 'react'
import type { PriceLevel } from '../../types/market'

const TONE_CLASS = {
  up: 'text-up',
  down: 'text-down',
  flat: 'text-flat',
} as const

interface Props {
  level: PriceLevel | null
  side: 'bid' | 'ask'
  referencePrice?: number
  highPrice?: number
  lowPrice?: number
}

function DepthRowImpl({ level, side, referencePrice, highPrice, lowPrice }: Props) {
  const tone = level ? priceTone(level.price, referencePrice) : 'flat'
  const hl = level ? hlLabel(level.price, highPrice, lowPrice) : null

  const hlCell = (
    <span className={`text-center ${hl?.cls ?? ''}`}>{hl?.ch ?? ''}</span>
  )

  // bid: [HL] [SIZE 靠左] [PRICE 靠右]   ask: [PRICE 靠左] [SIZE 靠右] [HL]
  if (side === 'bid') {
    return (
      <div className="grid grid-cols-[16px_1fr_auto] gap-3 items-center px-3 py-2 border-b border-bg-row last:border-0 text-sm font-mono">
        {hlCell}
        <span className="text-neutral-400 text-left">
          {level ? formatSize(level.size) : '—'}
        </span>
        <span className={`${TONE_CLASS[tone]} text-right tabular-nums`}>
          {level ? formatPrice(level.price) : '—'}
        </span>
      </div>
    )
  }
  return (
    <div className="grid grid-cols-[auto_1fr_16px] gap-3 items-center px-3 py-2 border-b border-bg-row last:border-0 text-sm font-mono">
      <span className={`${TONE_CLASS[tone]} text-left tabular-nums`}>
        {level ? formatPrice(level.price) : '—'}
      </span>
      <span className="text-neutral-400 text-right">
        {level ? formatSize(level.size) : '—'}
      </span>
      {hlCell}
    </div>
  )
}

function priceTone(price: number, ref?: number): keyof typeof TONE_CLASS {
  if (ref === undefined) return 'flat'
  if (price > ref) return 'up'
  if (price < ref) return 'down'
  return 'flat'
}

function hlLabel(
  price: number,
  high?: number,
  low?: number,
): { ch: string; cls: string } | null {
  if (high !== undefined && price === high) return { ch: 'H', cls: 'text-up' }
  if (low !== undefined && price === low) return { ch: 'L', cls: 'text-down' }
  return null
}

function formatSize(n: number): string {
  return n.toLocaleString('en-US')
}

function formatPrice(p: number): string {
  return p.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })
}

export const DepthRow = memo(DepthRowImpl)
