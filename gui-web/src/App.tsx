import { useState } from 'react'
import { useAppStore } from './store/app'
import { useMarketBridge, connectAndResubscribe } from './hooks/useMarketBridge'
import { saveApiKey, saveEndpoints } from './persist'
import { api } from './tauri'
import { StatusBar } from './components/StatusBar'
import { Watchlist } from './components/Watchlist'
import { DepthBook } from './components/depth-book/DepthBook'
import { TradeTape } from './components/TradeTape'
import { CandleChart } from './components/candle-chart/CandleChart'
import { IndicesBar } from './components/IndicesBar'
import { ApiKeyModal } from './components/ApiKeyModal'

function App() {
  useMarketBridge()
  const apiKey = useAppStore((s) => s.apiKey)
  const restBaseUrl = useAppStore((s) => s.restBaseUrl)
  const wsUrl = useAppStore((s) => s.wsUrl)
  const [settingsOpen, setSettingsOpen] = useState(false)

  return (
    <div className="grid h-full grid-rows-[1fr_24px] grid-cols-[260px_1fr_360px] bg-bg-base text-neutral-200">
      <aside className="row-span-1 col-start-1 col-end-2 border-r border-bg-row overflow-hidden">
        <Watchlist />
      </aside>

      <main className="row-span-1 col-start-2 col-end-3 flex flex-col min-w-0 overflow-hidden">
        <IndicesBar />
        <div className="flex-1 min-h-0 overflow-hidden">
          <CandleChart />
        </div>
      </main>

      <aside className="row-span-1 col-start-3 col-end-4 flex flex-col border-l border-bg-row overflow-hidden">
        <div className="border-b border-bg-row">
          <DepthBook />
        </div>
        <div className="flex-1 min-h-0 overflow-hidden">
          <TradeTape />
        </div>
      </aside>

      <footer className="col-span-3 border-t border-bg-row">
        <StatusBar onOpenSettings={() => setSettingsOpen(true)} />
      </footer>

      <ApiKeyModal
        open={!apiKey || settingsOpen}
        editMode={Boolean(apiKey)}
        initialRestBaseUrl={restBaseUrl}
        initialWsUrl={wsUrl}
        onClose={() => setSettingsOpen(false)}
        onSubmit={async ({ apiKey: typedKey, restBaseUrl: rest, wsUrl: ws }) => {
          const store = useAppStore.getState()
          const key = typedKey || store.apiKey || ''
          if (!key) return

          // Persist + update store first so any reconnect below reads fresh values.
          const tasks: Promise<unknown>[] = [saveEndpoints(rest, ws)]
          if (typedKey) tasks.push(saveApiKey(typedKey))
          await Promise.all(tasks)
          if (typedKey) store.setApiKey(typedKey)
          store.setEndpoints(rest, ws)
          setSettingsOpen(false)

          // Bridge short-circuits connect() while a client is live, so any
          // endpoint/key change needs a disconnect → reconnect round-trip.
          if (store.conn && store.conn.state !== 'disconnected') {
            try {
              await api.disconnect()
            } catch (e) {
              console.warn('disconnect failed before reconnect', e)
            }
          }
          await connectAndResubscribe(key, rest, ws, store.watchlist)
        }}
      />
    </div>
  )
}

export default App
