import type { Config } from 'tailwindcss'

const config: Config = {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  darkMode: 'class',
  theme: {
    extend: {
      fontFamily: {
        sans: [
          '"PingFang TC"',
          '"Noto Sans TC"',
          '-apple-system',
          'BlinkMacSystemFont',
          '"Segoe UI"',
          'sans-serif',
        ],
        mono: [
          '"JetBrains Mono"',
          '"SF Mono"',
          'Menlo',
          '"PingFang TC"',
          'monospace',
        ],
      },
      colors: {
        // Taiwan market convention: red = up, green = down
        up: '#ef4444',
        down: '#22c55e',
        flat: '#a3a3a3',
        bg: {
          base: '#0b0e13',
          panel: '#11151c',
          row: '#161b24',
          hover: '#1d2330',
        },
      },
    },
  },
  plugins: [],
}

export default config
