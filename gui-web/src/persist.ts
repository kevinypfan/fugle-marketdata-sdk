import { Store, load } from '@tauri-apps/plugin-store'

let watchlistStore: Store | null = null
let secretsStore: Store | null = null

async function getWatchlistStore(): Promise<Store> {
  if (!watchlistStore) {
    watchlistStore = await load('watchlist.json')
  }
  return watchlistStore
}

async function getSecretsStore(): Promise<Store> {
  if (!secretsStore) {
    secretsStore = await load('secrets.json')
  }
  return secretsStore
}

export async function loadPersisted(): Promise<{
  watchlist: string[]
  apiKey: string | null
}> {
  const [w, s] = await Promise.all([getWatchlistStore(), getSecretsStore()])
  const watchlist = (await w.get<string[]>('symbols')) ?? []
  const apiKey = (await s.get<string>('apiKey')) ?? null
  return { watchlist, apiKey }
}

export async function saveWatchlist(symbols: string[]): Promise<void> {
  const w = await getWatchlistStore()
  await w.set('symbols', symbols)
  await w.save()
}

export async function saveApiKey(key: string): Promise<void> {
  const s = await getSecretsStore()
  await s.set('apiKey', key)
  await s.save()
}
