export type Tone = 'up' | 'down' | 'flat'

export const TONE_CLASS: Record<Tone, string> = {
  up: 'text-up',
  down: 'text-down',
  flat: 'text-flat',
}

export function toneFromDiff(diff: number | undefined): Tone {
  if (diff === undefined || Number.isNaN(diff)) return 'flat'
  if (diff > 0) return 'up'
  if (diff < 0) return 'down'
  return 'flat'
}
