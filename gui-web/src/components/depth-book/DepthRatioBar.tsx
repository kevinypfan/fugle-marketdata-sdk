import { memo, type ReactNode } from 'react'

const FLAT_HEX = '#a3a3a3'

interface Props {
  /** Left-side content (label + value). */
  left: ReactNode
  /** Optional center text (e.g. "內外盤比"). */
  center?: ReactNode
  /** Right-side content. */
  right: ReactNode
  /** 0-100; the proportion the left color occupies. */
  leftPct: number
  leftColor: string
  rightColor: string
  /** When false, render a flat grey bar regardless of leftPct. */
  hasData: boolean
}

/**
 * Header / Summary 共用 ratio bar — 一行文字 + 下方 2px gradient bar。
 * Gradient 用同色雙 stop 製造 sharp 邊界 (masterlink progress-bar 的 trick)。
 */
function DepthRatioBarImpl({
  left,
  center,
  right,
  leftPct,
  leftColor,
  rightColor,
  hasData,
}: Props) {
  const stops = hasData
    ? [
        { color: leftColor, pct: leftPct },
        { color: rightColor, pct: 100 - leftPct },
      ]
    : [{ color: FLAT_HEX, pct: 100 }]

  return (
    <div>
      <div className="flex items-center justify-between px-3 h-8 text-xs">
        <span className="flex items-center gap-1">{left}</span>
        {center && <span className="text-neutral-500">{center}</span>}
        <span className="flex items-center gap-1">{right}</span>
      </div>
      <div className="h-[2px]" style={{ background: gradientStops(stops) }} />
    </div>
  )
}

function gradientStops(stops: { color: string; pct: number }[]): string {
  let acc = 0
  const parts: string[] = []
  for (const s of stops) {
    parts.push(`${s.color} ${acc}%`)
    acc += s.pct
    parts.push(`${s.color} ${acc}%`)
  }
  return `linear-gradient(to right, ${parts.join(',')})`
}

export const DepthRatioBar = memo(DepthRatioBarImpl)
