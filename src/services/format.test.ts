import { describe, expect, it } from 'vitest'
import { es } from '../i18n/es'
import { sections } from '../i18n'
import { formatBytes, formatDuration } from '../services/format'

/** RF-10: todas las secciones de i18n deben existir y no estar vacías. */
describe('i18n es', () => {
  it('expone todas las secciones requeridas', () => {
    for (const required of [
      'app',
      'nav',
      'common',
      'downloads',
      'conversion',
      'history',
      'settings',
      'about',
    ] as const) {
      expect(sections).toContain(required)
    }
  })

  it('la navegación tiene las cinco pantallas del shell', () => {
    expect(Object.keys(es.nav).sort()).toEqual(
      ['about', 'conversion', 'downloads', 'history', 'settings'].sort(),
    )
  })

  it('ninguna cadena visible es vacía', () => {
    const check = (obj: unknown): void => {
      for (const value of Object.values(obj as Record<string, unknown>)) {
        if (typeof value === 'string') expect(value.trim().length).toBeGreaterThan(0)
        else if (typeof value === 'object') check(value)
      }
    }
    check(es)
  })
})

/** RF-04 / convención de unidades (REA: B/KiB/MiB/GiB/TiB con espacio fino). */
describe('formatBytes', () => {
  it('formatea unidades binarias con un decimal', () => {
    expect(formatBytes(0)).toBe('0\u2009B')
    expect(formatBytes(512)).toBe('512\u2009B')
    expect(formatBytes(1024)).toBe('1\u2009KiB')
    expect(formatBytes(1536)).toBe('1.5\u2009KiB')
    expect(formatBytes(1048576)).toBe('1\u2009MiB')
  })

  it('devuelve un guion para valores indeterminados (nunca fabrica datos)', () => {
    expect(formatBytes(null)).toBe('—')
    expect(formatBytes(undefined)).toBe('—')
    expect(formatBytes(-5)).toBe('—')
  })
})

describe('formatDuration', () => {
  it('formatea mm:ss y h:mm:ss', () => {
    expect(formatDuration(65)).toBe('01:05')
    expect(formatDuration(3671)).toBe('1:01:11')
  })

  it('devuelve cadena vacía sin dato (indeterminado)', () => {
    expect(formatDuration(null)).toBe('')
    expect(formatDuration(0)).toBe('')
  })
})
