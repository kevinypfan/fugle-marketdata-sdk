import { memo, useEffect, useRef, useState } from 'react'
import {
  dispose,
  init,
  type Chart,
  type DeepPartial,
  type KLineData,
  type Styles,
} from 'klinecharts'
import { api } from '../../tauri'
import { useAppStore } from '../../store/app'
import type { CandleDto } from '../../types/market'
import type { Timeframe } from '../../types/timeframe'
import { INDICES_PREFIX } from '../../types/events'

// klinecharts runs on Canvas so it can't read Tailwind classes; mirror the
// theme tokens as hex (up=#ef4444, down=#22c55e, bg-row=#161b24, flat=#a3a3a3)
// so chart colors stay in sync with the rest of the UI by inspection.
const UP = '#ef4444'
const DOWN = '#22c55e'
const GRID = '#161b24'
const TEXT = '#a3a3a3'

const chartStyles: DeepPartial<Styles> = {
  grid: {
    horizontal: { color: GRID },
    vertical: { color: GRID },
  },
  candle: {
    bar: {
      upColor: UP,
      downColor: DOWN,
      upBorderColor: UP,
      downBorderColor: DOWN,
      upWickColor: UP,
      downWickColor: DOWN,
    },
  },
  xAxis: { axisLine: { color: GRID }, tickText: { color: TEXT } },
  yAxis: { axisLine: { color: GRID }, tickText: { color: TEXT } },
  separator: { color: GRID },
  crosshair: {
    horizontal: { line: { color: TEXT } },
    vertical: { line: { color: TEXT } },
  },
}

const TIMEFRAME_LABEL: Record<Timeframe, string> = {
  '1': '1分',
  D: '日',
  W: '週',
  M: '月',
}
const TIMEFRAMES: Timeframe[] = ['1', 'D', 'W', 'M']

function CandleChartImpl() {
  const containerRef = useRef<HTMLDivElement>(null)
  const chartRef = useRef<Chart | null>(null)
  // Monotonic token discards stale fetches when user rapidly swaps symbol/timeframe.
  const fetchGenRef = useRef(0)
  const [timeframe, setTimeframe] = useState<Timeframe>('1')

  const selected = useAppStore((s) => s.selected)
  const candles = useAppStore((s) =>
    selected ? s.symbols[selected]?.candles : undefined,
  )

  // (1) Mount once: init + VOL pane + styles. Container is always rendered
  // (overlay pattern below) so the ref is valid on first commit.
  useEffect(() => {
    if (!containerRef.current) return
    const chart = init(containerRef.current)
    if (!chart) return
    chart.setStyles(chartStyles)
    chart.createIndicator('VOL', false, { id: 'pane_vol' })
    chartRef.current = chart
    return () => {
      const c = chartRef.current
      if (c) dispose(c)
      chartRef.current = null
    }
  }, [])

  // (2) Fetch when symbol or timeframe changes; result lands in the store.
  // Generation token prevents a stale response from overwriting fresh data.
  useEffect(() => {
    if (!selected || selected.startsWith(INDICES_PREFIX)) return
    const gen = ++fetchGenRef.current
    void (async () => {
      try {
        const data = await api.fetchCandles(selected, timeframe)
        if (gen !== fetchGenRef.current) return
        useAppStore.getState().setCandles(selected, data, timeframe)
      } catch (e) {
        console.error('fetchCandles failed', selected, timeframe, e)
      }
    })()
  }, [selected, timeframe])

  // (3) Sync store candles → chart. Streaming CandleTick mutates the same
  // array, so this effect also handles the live update path.
  useEffect(() => {
    const chart = chartRef.current
    if (!chart) return
    chart.applyNewData((candles ?? []).map(toKLineData))
  }, [candles])

  const placeholder = !selected
    ? '尚未選擇標的'
    : selected.startsWith(INDICES_PREFIX)
      ? '此標的無 K 線'
      : null

  return (
    <div className="flex flex-col h-full bg-bg-base">
      <div className="flex items-center gap-1 px-3 h-8 border-b border-bg-row text-xs">
        {TIMEFRAMES.map((tf) => (
          <button
            key={tf}
            onClick={() => setTimeframe(tf)}
            className={`px-2 py-0.5 rounded ${
              tf === timeframe
                ? 'bg-bg-hover text-neutral-200'
                : 'text-neutral-500 hover:text-neutral-300'
            }`}
          >
            {TIMEFRAME_LABEL[tf]}
          </button>
        ))}
      </div>
      <div className="relative flex-1 min-h-0">
        <div ref={containerRef} className="absolute inset-0" />
        {placeholder && (
          <div className="absolute inset-0 flex items-center justify-center bg-bg-base text-neutral-600 text-sm">
            {placeholder}
          </div>
        )}
      </div>
    </div>
  )
}

function toKLineData(c: CandleDto): KLineData {
  return {
    timestamp: new Date(c.date).getTime(),
    open: c.open,
    high: c.high,
    low: c.low,
    close: c.close,
    volume: c.volume,
  }
}

export const CandleChart = memo(CandleChartImpl)
