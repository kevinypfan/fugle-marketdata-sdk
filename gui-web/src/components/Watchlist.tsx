import { useState } from 'react'
import { useAppStore } from '../store/app'
import { api } from '../tauri'
import { seedSymbol } from '../hooks/useMarketBridge'
import { WatchlistRow } from './WatchlistRow'

export function Watchlist() {
  const watchlist = useAppStore((s) => s.watchlist)
  const selected = useAppStore((s) => s.selected)
  const setSelected = useAppStore((s) => s.setSelected)
  const addToWatchlist = useAppStore((s) => s.addToWatchlist)
  const removeFromWatchlist = useAppStore((s) => s.removeFromWatchlist)
  const moveToTop = useAppStore((s) => s.moveToTop)
  const moveToBottom = useAppStore((s) => s.moveToBottom)
  const apiKey = useAppStore((s) => s.apiKey)
  const restBaseUrl = useAppStore((s) => s.restBaseUrl)

  const [input, setInput] = useState('')

  async function handleAdd(e: React.FormEvent) {
    e.preventDefault()
    const symbol = input.trim().toUpperCase()
    if (!symbol) return
    setInput('')
    if (!addToWatchlist(symbol)) return
    if (apiKey) {
      try {
        await api.subscribe(symbol)
        await seedSymbol(symbol, restBaseUrl)
      } catch (err) {
        console.error('subscribe failed', symbol, err)
      }
    }
  }

  async function handleRemove(symbol: string) {
    removeFromWatchlist(symbol)
    if (apiKey) {
      try {
        await api.unsubscribe(symbol)
      } catch (err) {
        console.error('unsubscribe failed', symbol, err)
      }
    }
  }

  return (
    <div className="flex flex-col h-full bg-bg-panel">
      <header className="px-3 py-2 text-xs font-medium text-neutral-400 border-b border-bg-row">
        自選
      </header>

      <form onSubmit={handleAdd} className="px-3 py-2 border-b border-bg-row">
        <input
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder="代號 e.g. 2330"
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
            onMoveTop={() => moveToTop(symbol)}
            onMoveBottom={() => moveToBottom(symbol)}
            onRemove={() => handleRemove(symbol)}
          />
        ))}
        {watchlist.length === 0 && (
          <li className="px-3 py-6 text-center text-xs text-neutral-500">
            輸入代號加入自選
          </li>
        )}
      </ul>
    </div>
  )
}
