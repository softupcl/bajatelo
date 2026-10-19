import { describe, expect, it } from 'vitest'
import { errorText, toTaskError } from './errors'
import { ERROR_CODES } from '../types/contracts'

describe('catálogo de errores', () => {
  it('todos los códigos del contrato tienen texto en español', () => {
    for (const code of ERROR_CODES) {
      const text = errorText(code)
      expect(text.length).toBeGreaterThan(5)
      expect(text).not.toBe('Error desconocido.')
    }
  })

  it('códigos fuera del catálogo no generan texto fabricado', () => {
    expect(errorText('NO-EXISTE' as never)).toBe('Error desconocido.')
  })
})

describe('toTaskError', () => {
  it('reconoce un TaskError serializado de Rust', () => {
    const out = toTaskError({ code: 'EC-10', detail: 'algo' })
    expect(out?.code).toBe('EC-10')
    expect(out?.detail).toBe('algo')
  })

  it('rechaza objetos que no son TaskError', () => {
    expect(toTaskError('boom')).toBeNull()
    expect(toTaskError({ code: 'NOPE' })).toBeNull()
    expect(toTaskError(42)).toBeNull()
  })
})
