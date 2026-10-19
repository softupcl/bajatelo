import { invoke } from '@tauri-apps/api/core'
import { getVersion } from '@tauri-apps/api/app'
import { open as openDialog, save as saveDialog } from '@tauri-apps/plugin-dialog'
import type {
  HistoryEntry,
  LocalMediaInfo,
  MediaInfo,
  Preferences,
  ToolDiagnostics,
} from '../types/contracts'

/**
 * Puente UI ↔ Rust (docs/spec/architecture.md §4).
 * Interfaz inyectable: los tests E2E usan un doble; las vistas llaman a getBridge().
 */
export interface Bridge {
  /** Diagnóstico real de dependencias (RF-08) — comando Rust `get_diagnostics`. */
  getDiagnostics(): Promise<ToolDiagnostics[]>
  /** Análisis real de una URL (RF-01/RF-02) — comando Rust `analyze_url`. */
  analyzeUrl(url: string): Promise<MediaInfo>
  /** Conversión local (RF-06/C-20): selección nativa + FFprobe + FFmpeg. */
  pickMediaFile(): Promise<string | null>
  /** Diálogo nativo de guardado (RF-06): destino de la conversión. */
  pickSaveDestination(defaultName: string): Promise<string | null>
  probeLocalFile(path: string): Promise<LocalMediaInfo>
  startConversion(
    input: string,
    format: 'mp4' | 'mp3',
    destination: string,
    policy: ConflictPolicyAlias,
  ): Promise<string>
  cancelTask(taskId: string): Promise<void>
  /** Preferencias (RF-09, persistencia SQLite D-07). */
  getPreferences(): Promise<Preferences>
  savePreferences(prefs: Preferences): Promise<Preferences>
  /** Historial (RF-07). */
  historyList(): Promise<HistoryEntry[]>
  historyRemove(id: string): Promise<boolean>
  historyClear(): Promise<number>
  /** Diálogos nativos (RF-03): carpeta destino. */
  pickDirectory(): Promise<string | null>
  /** Versión de la app desde el empaquetado Tauri. */
  getAppVersion(): Promise<string>
}

/** Alias local para no repetir la dependencia del contrato del clon. */
type ConflictPolicyAlias = 'ask' | 'number' | 'cancel'

class TauriBridge implements Bridge {
  async getDiagnostics(): Promise<ToolDiagnostics[]> {
    return invoke<ToolDiagnostics[]>('get_diagnostics')
  }
  async analyzeUrl(url: string): Promise<MediaInfo> {
    return invoke<MediaInfo>('analyze_url', { url })
  }
  async getPreferences(): Promise<Preferences> {
    return invoke<Preferences>('get_preferences')
  }
  async savePreferences(prefs: Preferences): Promise<Preferences> {
    return invoke<Preferences>('save_preferences', { prefs })
  }
  async historyList(): Promise<HistoryEntry[]> {
    return invoke<HistoryEntry[]>('history_list')
  }
  async historyRemove(id: string): Promise<boolean> {
    return invoke<boolean>('history_remove', { id })
  }
  async historyClear(): Promise<number> {
    return invoke<number>('history_clear')
  }
  async pickDirectory(): Promise<string | null> {
    const result = await openDialog({
      directory: true,
      multiple: false,
      title: 'Carpeta de destino',
    })
    return typeof result === 'string' ? result : null
  }
  async pickMediaFile(): Promise<string | null> {
    const result = await openDialog({
      multiple: false,
      title: 'Archivo a convertir',
      filters: [
        { name: 'Vídeo', extensions: ['mp4', 'm4v', 'webm', 'mkv', 'avi', 'mov'] },
        { name: 'Audio', extensions: ['mp3', 'm4a', 'ogg', 'opus', 'wav', 'flac'] },
        { name: 'Todos', extensions: ['*'] },
      ],
    })
    return typeof result === 'string' ? result : null
  }
  async pickSaveDestination(defaultName: string): Promise<string | null> {
    const result = await saveDialog({
      title: 'Guardar conversión como…',
      defaultPath: defaultName,
    })
    return typeof result === 'string' ? result : null
  }
  async probeLocalFile(path: string): Promise<LocalMediaInfo> {
    return invoke<LocalMediaInfo>('probe_local_file', { path })
  }
  async startConversion(
    input: string,
    format: 'mp4' | 'mp3',
    destination: string,
    policy: ConflictPolicyAlias,
  ): Promise<string> {
    return invoke<string>('start_conversion', { input, format, destination, policy })
  }
  /** Cancela una tarea en curso (T4) — mismo comando que para descargas. */
  async cancelTask(taskId: string): Promise<void> {
    await invoke<void>('cancel_task', { taskId })
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
