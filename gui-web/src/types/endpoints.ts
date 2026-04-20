export interface EndpointPreset {
  label: string
  url: string
}

export const REST_PRESETS: EndpointPreset[] = [
  { label: '正式 (Prod)', url: 'https://api.fugle.tw/marketdata/v1.0' },
  { label: '測試 (Dev)', url: 'https://api-dev.fugle.tw/marketdata/v1.0' },
]

export const WS_PRESETS: EndpointPreset[] = [
  { label: '正式 API', url: 'wss://api.fugle.tw/marketdata/v1.0/stock/streaming' },
  { label: '測試 API', url: 'wss://api-dev.fugle.tw/marketdata/v1.0/stock/streaming' },
  { label: '正式 Express', url: 'wss://express.fugle.tw/marketdata/v1.0/stock/streaming' },
  { label: '測試 Express', url: 'wss://express-dev.fugle.tw/marketdata/v1.0/stock/streaming' },
]

export const DEFAULT_REST_URL = REST_PRESETS[0].url
export const DEFAULT_WS_URL = WS_PRESETS[0].url
