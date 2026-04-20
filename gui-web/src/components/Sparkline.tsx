import { memo } from 'react'
import type { Tone } from '../lib/tone'

interface Props {
  data: readonly number[]
  tone: Tone
  width?: number
  height?: number
}

function SparklineImpl({ data, tone, width = 44, height = 22 }: Props) {
  if (data.length < 2) return <svg width={width} height={height} />
  let min = data[0]
  let max = data[0]
  for (let i = 1; i < data.length; i++) {
    const v = data[i]
    if (v < min) min = v
    if (v > max) max = v
  }
  const range = max - min || 1
  const stroke = tone === 'up' ? '#ef4444' : tone === 'down' ? '#22c55e' : '#6b7280'
  const step = width / (data.length - 1)
  let points = ''
  for (let i = 0; i < data.length; i++) {
    const x = (i * step).toFixed(1)
    const y = (height - ((data[i] - min) / range) * height).toFixed(1)
    points += (i ? ' ' : '') + x + ',' + y
  }
  return (
    <svg width={width} height={height} className="shrink-0">
      <polyline points={points} fill="none" stroke={stroke} strokeWidth={1} />
    </svg>
  )
}

export const Sparkline = memo(SparklineImpl)
