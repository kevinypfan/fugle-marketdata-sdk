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
  stockWatchlist: string[]
  futoptWatchlist: string[]
  apiKey: string | null
  restBaseUrl: string | null
  wsUrl: string | null
}> {
  const [w, s] = await Promise.all([getWatchlistStore(), getSecretsStore()])
  // Legacy pre-futopt installs stored stock symbols under `symbols`. Fall back
  // so existing users don't lose their list on upgrade.
  const stockWatchlist =
    (await w.get<string[]>('stockSymbols')) ??
    (await w.get<string[]>('symbols')) ??
    []
  const futoptWatchlist = (await w.get<string[]>('futoptSymbols')) ?? []
  const apiKey = (await s.get<string>('apiKey')) ?? null
  const restBaseUrl = (await s.get<string>('restBaseUrl')) ?? null
  const wsUrl = (await s.get<string>('wsUrl')) ?? null
  return { stockWatchlist, futoptWatchlist, apiKey, restBaseUrl, wsUrl }
}

export async function saveWatchlists(
  stock: string[],
  futopt: string[],
): Promise<void> {
  const w = await getWatchlistStore()
  await w.set('stockSymbols', stock)
  await w.set('futoptSymbols', futopt)
  await w.save()
}

export async function saveApiKey(key: string): Promise<void> {
  const s = await getSecretsStore()
  await s.set('apiKey', key)
  await s.save()
}

export async function saveEndpoints(restBaseUrl: string, wsUrl: string): Promise<void> {
  const s = await getSecretsStore()
  await s.set('restBaseUrl', restBaseUrl)
  await s.set('wsUrl', wsUrl)
  await s.save()
}
