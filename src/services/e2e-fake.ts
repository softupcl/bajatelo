import type { Bridge } from './bridge'
import type { HistoryEntry, LocalMediaInfo, MediaInfo, Preferences, ToolDiagnostics } from '../types/contracts'

/**
 * Doble del puente SOLO para pruebas E2E (Playwright sobre Vite, sin backend Tauri).
 * Se activa con la clave 'rea-e2e=1' en localStorage desde main.ts.
 * Los fixtures siguen los contratos de src/types/contracts.ts — nada falso se
 * presenta como funcionalidad real fuera de este modo de pruebas.
 */
const FIXTURE_INFO: MediaInfo = {
  title: 'Vídeo demo del_analysis',
  portal: 'youtube',
  durationSec: 61.5,
  qualities: [
    { id: '137', label: '1080p (HD)', height: 1080, fps: 30, container: 'mp4', audioOnly: false, audioFormatId: '140', estimatedBytes: 5_800 },
    { id: '136', label: '720p (HD)', height: 720, fps: 30, container: 'mp4', audioOnly: false, audioFormatId: '140', estimatedBytes: 3_800 },
    { id: '18', label: '360p', height: 360, fps: 30, container: 'mp4', audioOnly: false, audioFormatId: null, estimatedBytes: 1_000 },
  ],
}

class E2eFake implements Bridge {
  async getDiagnostics(): Promise<ToolDiagnostics[]> {
    return [
      { tool: 'ytdlp', status: 'ok', version: '2026.e2e', path: '/e2e/fake/yt-dlp' },
      { tool: 'ffmpeg', status: 'ok', version: 'e2e.9.0', path: '/e2e/fake/ffmpeg' },
      { tool: 'ffprobe', status: 'ok', version: 'e2e.9.0', path: '/e2e/fake/ffprobe' },
    ]
  }
  async analyzeUrl(_url: string): Promise<MediaInfo> {
    return FIXTURE_INFO
  }
  async pickMediaFile(): Promise<string | null> {
    return '/e2e/fake/entrada.mp4'
  }
  async pickSaveDestination(defaultName: string): Promise<string | null> {
    return `/e2e/fake/${defaultName}`
  }
  async probeLocalFile(_path: string): Promise<LocalMediaInfo> {
    return {
      container: 'mov,mp4,m4a,3gp,3g2,mj2',
      videoCodec: 'avc1',
      audioCodec: 'aac',
      durationSec: 1.0,
      sizeBytes: 47_143,
    }
  }
  async startConversion(): Promise<string> {
    // En modo E2E no se automatiza la conversión (requiere runtime Tauri).
    return Promise.reject(new Error('E2E: conversión no automatizable'))
  }
  async cancelTask(): Promise<void> {
    /* no-op en E2E */
  }
  async getPreferences(): Promise<Preferences> {
    return {
      defaultDestination: '',
      alwaysUseLastPath: true,
      neverAskFilename: false,
      conflictPolicy: 'ask',
      rememberQuality: true,
      rememberFormat: true,
      removeFinishedFromQueue: false,
      locale: 'es',
    }
  }
  async savePreferences(prefs: Preferences): Promise<Preferences> {
    return prefs
  }
  async historyList(): Promise<HistoryEntry[]> {
    return []
  }
  async historyRemove(): Promise<boolean> {
    return false
  }
  async historyClear(): Promise<number> {
    return 0
  }
  async pickDirectory(): Promise<string | null> {
    return '/e2e/fake/destino'
  }
  async getAppVersion(): Promise<string> {
    return '0.1.0-e2e'
  }
}

/** Instalar el doble para Playwright (bajo su competencia). */
export function installE2eBridge(): void {
  if (typeof localStorage !== 'undefined' && localStorage.getItem('rea-e2e') === '1') {
    import('./bridge').then(({ setBridge }) => setBridge(new E2eFake()))
  }
}
