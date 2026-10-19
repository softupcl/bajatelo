import { invoke } from '@tauri-apps/api/core'
import { getVersion } from '@tauri-apps/api/app'
import type { ToolDiagnostics } from '../types/contracts'

/**
 * Puente UI ↔ Rust (docs/spec/architecture.md §4).
 * Interfaz inyectable: los tests E2E usan un doble; las vistas llaman a getBridge().
 */
export interface Bridge {
  /** Diagnóstico real de dependencias (RF-08) — comando Rust `get_diagnostics`. */
  getDiagnostics(): Promise<ToolDiagnostics[]>
  /** Versión de la app desde el empaquetado Tauri. */
  getAppVersion(): Promise<string>
}

class TauriBridge implements Bridge {
  async getDiagnostics(): Promise<ToolDiagnostics[]> {
    return invoke<ToolDiagnostics[]>('get_diagnostics')
  }
  async getAppVersion(): Promise<string> {
    return getVersion()
  }
}

let active: Bridge = new TauriBridge()

/** Sustituir el puente (solo en pruebas). */
export function setBridge(bridge: Bridge): void {
  active = bridge
}

export function getBridge(): Bridge {
  return active
}
