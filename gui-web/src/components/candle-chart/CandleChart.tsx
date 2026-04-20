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
const STOCK_TIMEFRAMES: Timeframe[] = ['1', 'D', 'W', 'M']
/** FutOpt's historical daily/weekly/monthly endpoint isn't supported in this
 *  SDK bridge — only intraday 1-min. Restricting the picker avoids empty
 *  charts and confusing error logs on clicks. */
const FUTOPT_TIMEFRAMES: Timeframe[] = ['1']

function CandleChartImpl() {
  const containerRef = useRef<HTMLDivElement>(null)
  const chartRef = useRef<Chart | null>(null)
  // Monotonic token discards stale fetches when user rapidly swaps symbol/timeframe.
  const fetchGenRef = useRef(0)
  const [timeframe, setTimeframe] = useState<Timeframe>('1')

  const selected = useAppStore((s) => s.selected)
  const restBaseUrl = useAppStore((s) => s.restBaseUrl)
  const candles = useAppStore((s) =>
    selected ? s.symbols[selected]?.candles : undefined,
  )
  const selectedMarket = useAppStore((s) =>
    selected ? s.symbols[selected]?.market : undefined,
  )
  const availableTimeframes =
    selectedMarket === 'futopt' ? FUTOPT_TIMEFRAMES : STOCK_TIMEFRAMES

  // If user was on D/W/M for a stock and switches to a futopt symbol, snap
  // back to '1' — the alternative is an empty chart with a silent fetch error.
  useEffect(() => {
    if (!availableTimeframes.includes(timeframe)) {
      setTimeframe('1')
    }
  }, [availableTimeframes, timeframe])

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

  const futoptSession = useAppStore((s) => s.futoptSession)

  // (2) Fetch when symbol, timeframe, or session changes; result lands in
  // the store. Generation token prevents a stale response from overwriting
  // fresh data. For futopt symbols, the session flag gates which disjoint
  // stream (日盤 vs 夜盤) REST returns — forgetting it silently overwrites
  // the chart with the other session's candles.
  useEffect(() => {
    if (!selected || selected.startsWith(INDICES_PREFIX)) return
    const gen = ++fetchGenRef.current
    void (async () => {
      try {
        const market = useAppStore.getState().symbols[selected]?.market ?? 'stock'
        const data =
          market === 'futopt'
            ? await api.fetchFutoptCandles(
                selected,
                timeframe,
                restBaseUrl,
                futoptSession === 'afterhours',
              )
            : await api.fetchCandles(selected, timeframe, restBaseUrl)
        if (gen !== fetchGenRef.current) return
        useAppStore.getState().setCandles(selected, data, timeframe)
      } catch (e) {
        console.error('fetchCandles failed', selected, timeframe, e)
      }
    })()
  }, [selected, timeframe, restBaseUrl, futoptSession])

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
        {availableTimeframes.map((tf) => (
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
