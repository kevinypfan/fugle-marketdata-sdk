import { memo, useEffect, useState } from 'react'
import { useAppStore } from '../store/app'
import { api } from '../tauri'

const LARGE_ORDER_THRESHOLD = 100
const PAGE_SIZE = 100

function fmtTime(t: number): string {
  const d = new Date(t)
  return `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}:${String(d.getSeconds()).padStart(2, '0')}`
}

function TradeTapeImpl() {
  const selected = useAppStore((s) => s.selected)
  const restBaseUrl = useAppStore((s) => s.restBaseUrl)
  // NOTE: never allocate a fresh array inside the selector — it makes
  // useSyncExternalStore think state changed every tick and infinite-loops.
  const liveTape = useAppStore((s) =>
    selected ? s.symbols[selected]?.tape : undefined,
  )
  const trialTape = useAppStore((s) =>
    selected ? s.symbols[selected]?.trialTape : undefined,
  )
  const tapeSeedCount = useAppStore((s) =>
    selected ? s.symbols[selected]?.tapeSeedCount : undefined,
  )
  const tapeExtraCount = useAppStore((s) =>
    selected ? s.symbols[selected]?.tapeExtraCount : undefined,
  )
  const hasMoreTape = useAppStore((s) =>
    selected ? s.symbols[selected]?.hasMoreTape : undefined,
  )
  const trialSeedCount = useAppStore((s) =>
    selected ? s.symbols[selected]?.trialSeedCount : undefined,
  )
  const trialExtraCount = useAppStore((s) =>
    selected ? s.symbols[selected]?.trialExtraCount : undefined,
  )
  const hasMoreTrialTape = useAppStore((s) =>
    selected ? s.symbols[selected]?.hasMoreTrialTape : undefined,
  )

  const [trialMode, setTrialMode] = useState(false)
  const [loadingMore, setLoadingMore] = useState(false)
  const [loadingTrial, setLoadingTrial] = useState(false)

  // Fetch trial snapshot on toggle ON / symbol change while in trial mode.
  // Static seed (no live stream): WS has no per-trade `isTrial` flag. The user
  // can paginate deeper via load-more the same way they do for regular trades.
  useEffect(() => {
    if (!trialMode || !selected) return
    let cancelled = false
    setLoadingTrial(true)
    api
      .fetchTrades(selected, restBaseUrl, { isTrial: true, limit: PAGE_SIZE })
      .then((trades) => {
        if (cancelled) return
        useAppStore.getState().setTrialTape(selected, trades, PAGE_SIZE)
      })
      .catch((e) => console.warn('trial trades fetch failed', selected, e))
      .finally(() => {
        if (!cancelled) setLoadingTrial(false)
      })
    return () => {
      cancelled = true
    }
  }, [trialMode, selected, restBaseUrl])

  async function loadMore() {
    if (!selected || loadingMore) return
    setLoadingMore(true)
    try {
      if (trialMode) {
        const offset = (trialSeedCount ?? 0) + (trialExtraCount ?? 0)
        const older = await api.fetchTrades(selected, restBaseUrl, {
          isTrial: true,
          offset,
          limit: PAGE_SIZE,
        })
        useAppStore.getState().appendOlderTrialTrades(selected, older, PAGE_SIZE)
      } else {
        const offset = (tapeSeedCount ?? 0) + (tapeExtraCount ?? 0)
        const older = await api.fetchTrades(selected, restBaseUrl, {
          offset,
          limit: PAGE_SIZE,
        })
        useAppStore.getState().appendOlderTrades(selected, older, PAGE_SIZE)
      }
    } catch (e) {
      console.warn('load more trades failed', selected, e)
    } finally {
      setLoadingMore(false)
    }
  }

  const tape = trialMode ? (trialTape ?? []) : (liveTape ?? [])
  const showLoadMore = trialMode
    ? (hasMoreTrialTape ?? false)
    : (hasMoreTape ?? false)

  return (
    <div className="flex flex-col h-full bg-bg-panel">
      <header className="border-b border-bg-row">
        <div className="flex items-center justify-between px-3 py-2">
          <span className="text-xs font-medium text-neutral-400">成交明細</span>
          <div className="flex text-[10px]">
            <button
              type="button"
              onClick={() => setTrialMode(false)}
              className={`px-2 py-0.5 border border-bg-row rounded-l ${!trialMode ? 'bg-bg-row text-neutral-200' : 'text-neutral-500 hover:text-neutral-300'}`}
            >
              一般
            </button>
            <button
              type="button"
              onClick={() => setTrialMode(true)}
              className={`px-2 py-0.5 border border-l-0 border-bg-row rounded-r ${trialMode ? 'bg-bg-row text-neutral-200' : 'text-neutral-500 hover:text-neutral-300'}`}
            >
              試撮
            </button>
          </div>
        </div>
        <div className="grid grid-cols-[56px_1fr_1fr_1fr_40px] gap-2 px-3 pb-1 text-[10px] text-neutral-500 font-mono">
          <span>時間</span>
          <span className="text-right">買</span>
          <span className="text-right">賣</span>
          <span className="text-right">成交</span>
          <span className="text-right">量</span>
        </div>
      </header>
      <ul className="flex-1 overflow-y-auto text-xs font-mono">
        {tape.map((t) => {
          const tone =
            t.direction === 1 ? 'bg-up/15 text-up'
            : t.direction === -1 ? 'bg-down/15 text-down'
            : 'text-neutral-300'
          const big = t.size >= LARGE_ORDER_THRESHOLD ? 'font-bold' : ''
          return (
            <li
              key={t.serial}
              className={`grid grid-cols-[56px_1fr_1fr_1fr_40px] items-center gap-2 px-3 py-1 border-b border-bg-row/50 ${tone}`}
            >
              <span className="text-neutral-500">{fmtTime(t.time)}</span>
              <span className="text-right text-neutral-500">
                {t.bid != null ? t.bid.toFixed(2) : '—'}
              </span>
              <span className="text-right text-neutral-500">
                {t.ask != null ? t.ask.toFixed(2) : '—'}
              </span>
              <span className={`${big} text-right`}>{t.price.toFixed(2)}</span>
              <span className={`${big} text-right`}>{t.size}</span>
            </li>
          )
        })}
        {tape.length === 0 && (
          <li className="px-3 py-6 text-center text-neutral-500">
            {trialMode
              ? loadingTrial
                ? '載入試撮資料…'
                : '此時段無試撮資料'
              : '尚無成交'}
          </li>
        )}
        {showLoadMore && (
          <li className="px-3 py-2 border-b border-bg-row/50">
            <button
              type="button"
              onClick={loadMore}
              disabled={loadingMore}
              className="w-full py-1.5 text-xs text-neutral-300 bg-bg-row/60 rounded hover:bg-bg-row disabled:opacity-50 disabled:cursor-wait"
            >
              {loadingMore ? '載入中…' : '載入更多'}
            </button>
          </li>
        )}
      </ul>
    </div>
  )
}

export const TradeTape = memo(TradeTapeImpl)
