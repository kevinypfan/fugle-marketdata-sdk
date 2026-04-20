import { useState } from 'react'
import { useAppStore } from '../store/app'
import { api } from '../tauri'
import {
  applyFutoptSession,
  ensureFutoptConnected,
  seedFutoptSymbol,
  seedSymbol,
} from '../hooks/useMarketBridge'
import { WatchlistRow } from './WatchlistRow'

export function Watchlist() {
  const activeMarket = useAppStore((s) => s.activeMarket)
  const setActiveMarket = useAppStore((s) => s.setActiveMarket)
  const stockWatchlist = useAppStore((s) => s.stockWatchlist)
  const futoptWatchlist = useAppStore((s) => s.futoptWatchlist)
  const selected = useAppStore((s) => s.selected)
  const setSelected = useAppStore((s) => s.setSelected)
  const addToWatchlist = useAppStore((s) => s.addToWatchlist)
  const removeFromWatchlist = useAppStore((s) => s.removeFromWatchlist)
  const moveToTop = useAppStore((s) => s.moveToTop)
  const moveToBottom = useAppStore((s) => s.moveToBottom)
  const apiKey = useAppStore((s) => s.apiKey)
  const restBaseUrl = useAppStore((s) => s.restBaseUrl)
  const wsUrl = useAppStore((s) => s.wsUrl)
  const futoptSession = useAppStore((s) => s.futoptSession)

  const [switching, setSwitching] = useState(false)

  const [input, setInput] = useState('')

  const watchlist = activeMarket === 'stock' ? stockWatchlist : futoptWatchlist

  async function handleAdd(e: React.FormEvent) {
    e.preventDefault()
    const symbol = input.trim().toUpperCase()
    if (!symbol) return
    setInput('')
    if (!addToWatchlist(activeMarket, symbol)) return
    if (!apiKey) return
    try {
      if (activeMarket === 'futopt') {
        // Futopt WS is on-demand: user may never touch futopt, in which case
        // we never open that connection. First add triggers the connect.
        // Pass stock wsUrl so dev/prod env stays consistent across markets.
        await ensureFutoptConnected(apiKey, wsUrl)
        const afterHours = futoptSession === 'afterhours'
        // Seed first so REST ticker populates the alias map before WS data
        // arrives. Then subscribe with user-input — server accepts aliases
        // (TXF1! / MXF1!); data events echo canonical which the store
        // redirects via symbolAliases.
        await seedFutoptSymbol(symbol, restBaseUrl, afterHours)
        await api.subscribeFutopt(symbol, afterHours)
      } else {
        await api.subscribe(symbol)
        await seedSymbol(symbol, restBaseUrl)
      }
    } catch (err) {
      console.error('subscribe failed', activeMarket, symbol, err)
    }
  }

  async function handleRemove(symbol: string) {
    removeFromWatchlist(activeMarket, symbol)
    if (!apiKey) return
    try {
      if (activeMarket === 'futopt') {
        await api.unsubscribeFutopt(symbol, futoptSession === 'afterhours')
      } else {
        await api.unsubscribe(symbol)
      }
    } catch (err) {
      console.error('unsubscribe failed', activeMarket, symbol, err)
    }
  }

  async function handleSessionChange(next: 'regular' | 'afterhours') {
    if (next === futoptSession || switching) return
    setSwitching(true)
    try {
      await applyFutoptSession(next, apiKey, restBaseUrl)
    } catch (err) {
      console.error('session switch failed', err)
    } finally {
      setSwitching(false)
    }
  }

  return (
    <div className="flex flex-col h-full bg-bg-panel">
      <header className="flex items-center justify-between px-3 py-2 border-b border-bg-row">
        <span className="text-xs font-medium text-neutral-400">自選</span>
        <div className="flex text-[10px]">
          <button
            type="button"
            onClick={() => setActiveMarket('stock')}
            className={`px-2 py-0.5 border border-bg-row rounded-l ${
              activeMarket === 'stock'
                ? 'bg-bg-row text-neutral-200'
                : 'text-neutral-500 hover:text-neutral-300'
            }`}
          >
            股票
          </button>
          <button
            type="button"
            onClick={() => setActiveMarket('futopt')}
            className={`px-2 py-0.5 border border-l-0 border-bg-row rounded-r ${
              activeMarket === 'futopt'
                ? 'bg-bg-row text-neutral-200'
                : 'text-neutral-500 hover:text-neutral-300'
            }`}
          >
            期貨
          </button>
        </div>
      </header>

      {activeMarket === 'futopt' && (
        <div className="flex items-center justify-between px-3 py-1.5 border-b border-bg-row text-[10px]">
          <span className="text-neutral-500">交易時段</span>
          <div className="flex">
            <button
              type="button"
              disabled={switching}
              onClick={() => handleSessionChange('regular')}
              className={`px-2 py-0.5 border border-bg-row rounded-l ${
                futoptSession === 'regular'
                  ? 'bg-bg-row text-neutral-200'
                  : 'text-neutral-500 hover:text-neutral-300'
              } disabled:opacity-50 disabled:cursor-wait`}
            >
              一般
            </button>
            <button
              type="button"
              disabled={switching}
              onClick={() => handleSessionChange('afterhours')}
              className={`px-2 py-0.5 border border-l-0 border-bg-row rounded-r ${
                futoptSession === 'afterhours'
                  ? 'bg-bg-row text-neutral-200'
                  : 'text-neutral-500 hover:text-neutral-300'
              } disabled:opacity-50 disabled:cursor-wait`}
            >
              盤後
            </button>
          </div>
        </div>
      )}

      <form onSubmit={handleAdd} className="px-3 py-2 border-b border-bg-row">
        <input
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder={activeMarket === 'futopt' ? '合約 e.g. TXFC4' : '代號 e.g. 2330'}
          className="w-full px-2 py-1 text-xs rounded bg-bg-base border border-bg-row focus:outline-none focus:border-neutral-500 font-mono"
        />
      </form>

      <ul className="flex-1 overflow-y-auto">
        {watchlist.map((symbol) => (
          <WatchlistRow
            key={symbol}
            symbol={symbol}
            selected={selected === symbol}
            onSelect={() => setSelected(symbol)}
            onMoveTop={() => moveToTop(activeMarket, symbol)}
            onMoveBottom={() => moveToBottom(activeMarket, symbol)}
            onRemove={() => handleRemove(symbol)}
          />
        ))}
        {watchlist.length === 0 && (
          <li className="px-3 py-6 text-center text-xs text-neutral-500">
            {activeMarket === 'futopt' ? '輸入期貨合約代號加入自選' : '輸入代號加入自選'}
          </li>
        )}
      </ul>
    </div>
  )
}
