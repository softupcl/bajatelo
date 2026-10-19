/**
 * Formateo de tamaños/duración para la UI.
 * Convención de la referencia (REA 01: humanizeBytes) adaptada al clon: unidades
 * binarias B/KiB/MiB/GiB/TiB con un decimal y espacio fino U+2009.
 */
const UNITS = ['B', 'KiB', 'MiB', 'GiB', 'TiB'] as const
const THIN_SPACE = '\u2009'

export function formatBytes(bytes: number | null | undefined): string {
  if (bytes == null || !Number.isFinite(bytes) || bytes < 0) return '—'
  let value = bytes
  let unitIndex = 0
  while (value >= 1024 && unitIndex < UNITS.length - 1) {
    value /= 1024
    unitIndex++
  }
  const rounded = unitIndex === 0 ? Math.round(value) : Math.round(value * 10) / 10
  return `${rounded}${THIN_SPACE}${UNITS[unitIndex]}`
}

/** Duración en segundos → "mm:ss" u "h:mm:ss" ("" si no hay dato). */
export function formatDuration(seconds: number | null | undefined): string {
  if (seconds == null || !Number.isFinite(seconds) || seconds <= 0) return ''
  const total = Math.round(seconds)
  const h = Math.floor(total / 3600)
  const m = Math.floor((total % 3600) / 60)
  const s = total % 60
  const pad = (n: number) => String(n).padStart(2, '0')
  return h > 0 ? `${h}:${pad(m)}:${pad(s)}` : `${pad(m)}:${pad(s)}`
}
