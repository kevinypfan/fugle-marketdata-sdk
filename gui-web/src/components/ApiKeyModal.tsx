interface ApiKeyModalProps {
  open: boolean
  onSubmit: (key: string) => void
}

export function ApiKeyModal({ open, onSubmit }: ApiKeyModalProps) {
  if (!open) return null
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60">
      <div className="w-96 p-5 rounded-lg bg-bg-panel border border-bg-row">
        <h2 className="text-sm font-medium mb-3">設定 Fugle API Key</h2>
        <form
          onSubmit={(e) => {
            e.preventDefault()
            const data = new FormData(e.currentTarget)
            const key = String(data.get('apiKey') ?? '').trim()
            if (key) onSubmit(key)
          }}
        >
          <input
            name="apiKey"
            type="password"
            autoFocus
            className="w-full px-3 py-2 text-sm rounded bg-bg-base border border-bg-row focus:outline-none focus:border-neutral-500 font-mono"
            placeholder="paste your key…"
          />
          <button
            type="submit"
            className="mt-3 w-full py-2 text-sm rounded bg-neutral-700 hover:bg-neutral-600"
          >
            儲存
          </button>
        </form>
      </div>
    </div>
  )
}
