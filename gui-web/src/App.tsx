import { useAppStore } from './store/app'
import { useMarketBridge, connectAndResubscribe } from './hooks/useMarketBridge'
import { saveApiKey } from './persist'
import { StatusBar } from './components/StatusBar'
import { Watchlist } from './components/Watchlist'
import { DepthBook } from './components/depth-book/DepthBook'
import { TradeTape } from './components/TradeTape'
import { CandleChart } from './components/candle-chart/CandleChart'
import { ApiKeyModal } from './components/ApiKeyModal'

function App() {
  useMarketBridge()
  const apiKey = useAppStore((s) => s.apiKey)

  return (
    <div className="grid h-full grid-rows-[1fr_24px] grid-cols-[260px_1fr_360px] bg-bg-base text-neutral-200">
      <aside className="row-span-1 col-start-1 col-end-2 border-r border-bg-row overflow-hidden">
        <Watchlist />
      </aside>

      <main className="row-span-1 col-start-2 col-end-3 min-w-0 overflow-hidden">
        <CandleChart />
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
        <StatusBar />
      </footer>

      <ApiKeyModal
        open={!apiKey}
        onSubmit={async (key) => {
          await saveApiKey(key)
          useAppStore.getState().setApiKey(key)
          await connectAndResubscribe(key, useAppStore.getState().watchlist)
        }}
      />
    </div>
  )
}

export default App
