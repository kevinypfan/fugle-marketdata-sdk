import { useState } from 'react'
import {
  DEFAULT_REST_URL,
  DEFAULT_WS_URL,
  REST_PRESETS,
  WS_PRESETS,
} from '../types/endpoints'

export interface ApiKeySubmit {
  /** Empty string means "keep existing" (only valid in edit mode). */
  apiKey: string
  restBaseUrl: string
  wsUrl: string
}

interface ApiKeyModalProps {
  open: boolean
  /** If true, `onClose` is rendered as a cancel button and empty apiKey is allowed. */
  editMode?: boolean
  initialRestBaseUrl?: string | null
  initialWsUrl?: string | null
  onClose?: () => void
  onSubmit: (payload: ApiKeySubmit) => void
}

export function ApiKeyModal({
  open,
  editMode = false,
  initialRestBaseUrl,
  initialWsUrl,
  onClose,
  onSubmit,
}: ApiKeyModalProps) {
  const [restBaseUrl, setRestBaseUrl] = useState(initialRestBaseUrl ?? DEFAULT_REST_URL)
  const [wsUrl, setWsUrl] = useState(initialWsUrl ?? DEFAULT_WS_URL)

  if (!open) return null
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60">
      <div className="w-96 p-5 rounded-lg bg-bg-panel border border-bg-row">
        <div className="flex items-center justify-between mb-3">
          <h2 className="text-sm font-medium">
            {editMode ? '設定' : '設定 Fugle API Key'}
          </h2>
          {editMode && onClose && (
            <button
              type="button"
              onClick={onClose}
              className="text-neutral-500 hover:text-neutral-200 text-lg leading-none"
              aria-label="close"
            >
              ×
            </button>
          )}
        </div>
        <form
          onSubmit={(e) => {
            e.preventDefault()
            const data = new FormData(e.currentTarget)
            const key = String(data.get('apiKey') ?? '').trim()
            if (!editMode && !key) return
            onSubmit({ apiKey: key, restBaseUrl, wsUrl })
          }}
        >
          <input
            name="apiKey"
            type="password"
            autoFocus
            className="w-full px-3 py-2 text-sm rounded bg-bg-base border border-bg-row focus:outline-none focus:border-neutral-500 font-mono"
            placeholder={editMode ? '留空則沿用現有 key' : 'paste your key…'}
          />

          <details className="mt-3 text-xs" open={editMode}>
            <summary className="cursor-pointer select-none text-neutral-400 hover:text-neutral-200">
              進階設定
            </summary>
            <div className="mt-2 space-y-2">
              <label className="block">
                <span className="text-neutral-400">REST 端點</span>
                <select
                  value={restBaseUrl}
                  onChange={(e) => setRestBaseUrl(e.target.value)}
                  className="mt-1 w-full px-2 py-1.5 rounded bg-bg-base border border-bg-row text-neutral-200 focus:outline-none focus:border-neutral-500"
                >
                  {REST_PRESETS.map((p) => (
                    <option key={p.url} value={p.url}>
                      {p.label}
                    </option>
                  ))}
                </select>
              </label>
              <label className="block">
                <span className="text-neutral-400">WebSocket 端點</span>
                <select
                  value={wsUrl}
                  onChange={(e) => setWsUrl(e.target.value)}
                  className="mt-1 w-full px-2 py-1.5 rounded bg-bg-base border border-bg-row text-neutral-200 focus:outline-none focus:border-neutral-500"
                >
                  {WS_PRESETS.map((p) => (
                    <option key={p.url} value={p.url}>
                      {p.label}
                    </option>
                  ))}
                </select>
              </label>
            </div>
          </details>

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
