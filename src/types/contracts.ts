/**
 * Contratos compartidos entre el frontend (Vue/TS) y el backend (Rust).
 * Fuente de verdad: docs/spec/data-model.md
 * Los modelos Rust en src-tauri/src/models/ deben mantenerse sincronizados
 * (hay tests de serialización que verifican la correspondencia).
 */

/** Estados de tarea — docs/spec/data-model.md §1 (AGENTS.md §8 + pausa D-05). */
export type TaskState =
  | 'queued'
  | 'inspecting'
  | 'ready'
  | 'downloading'
  | 'pausing'
  | 'paused'
  | 'converting'
  | 'completed'
  | 'failed'
  | 'cancelled'

export type OutputFormat = 'mp4' | 'mp3' | 'original'

/** Opción de calidad derivada exclusivamente de lo que reporta el motor (RF-02). */
export interface QualityOption {
  /** Identificador de formato del motor (p. ej. format_id de yt-dlp). */
  id: string
  /** Etiqueta visible, p. ej. "1080p (HD)" (patrón REA B2). */
  label: string
  height: number | null
  fps: number | null
  container: string
  audioOnly: boolean
  /** Tamaño estimado en bytes si el motor lo reporta; null si no. */
  estimatedBytes: number | null
}

/** Metadatos del análisis (RF-02). */
export interface MediaInfo {
  title: string | null
  portal: string | null
  durationSec: number | null
  qualities: QualityOption[]
}

/** Progreso (RF-04). Valores null = indeterminado visible; prohibido fabricarlos. */
export interface ProgressInfo {
  percent: number | null
  bytesDownloaded: number | null
  bytesTotal: number | null
  speedBps: number | null
  etaSeconds: number | null
  phase: 'downloading' | 'converting'
}

/** Códigos del catálogo de errores — docs/spec/error-catalog.md. */
export type ErrorCode =
  | 'EC-01'
  | 'EC-02'
  | 'EC-03'
  | 'EC-10'
  | 'EC-11'
  | 'EC-12'
  | 'EC-13'
  | 'EC-20'
  | 'EC-21'
  | 'EC-22'
  | 'EC-23'
  | 'EC-24'
  | 'EC-25'
  | 'EC-26'
  | 'EC-30'
  | 'EC-31'
  | 'EC-32'
  | 'EC-33'
  | 'EC-34'
  | 'EC-35'
  | 'EC-40'
  | 'EC-41'
  | 'EC-42'
  | 'EC-43'

/** Error de tarea: siempre con código del catálogo; detalle técnico opcional (CAT-2). */
export interface TaskError {
  code: ErrorCode
  /** Detalle técnico truncado y sin secretos, para soporte/logs (CAT-3). */
  detail?: string
}

/* Diagnóstico de dependencias (RF-08). */

export type ToolKind = 'ytdlp' | 'ffmpeg' | 'ffprobe'
export type ToolStatus = 'ok' | 'missing' | 'incompatible'

export interface ToolDiagnostics {
  tool: ToolKind
  status: ToolStatus
  /** Versión detectada, p. ej. "2026.08.19"; null si no hay. */
  version: string | null
  /** Ruta del ejecutable localizado; null si no hay. */
  path: string | null
}

/* Historial (RF-07) — persistido en SQLite (D-07). */

export type HistoryKind = 'download' | 'convert'
export type HistoryStatus = 'completed' | 'failed' | 'cancelled'

export interface HistoryEntry {
  id: string
  kind: HistoryKind
  title: string
  portal: string | null
  /** URL redactada (sin credenciales) o null en conversiones. */
  sourceUrl: string | null
  format: string | null
  quality: string | null
  destinationPath: string
  finalSizeBytes: number | null
  status: HistoryStatus
  errorCode: string | null
  startedAt: string
  finishedAt: string
}

/* Preferencias (RF-09). */

export type ConflictPolicy = 'ask' | 'number' | 'cancel'

export interface Preferences {
  defaultDestination: string
  alwaysUseLastPath: boolean
  neverAskFilename: boolean
  conflictPolicy: ConflictPolicy
  rememberQuality: boolean
  rememberFormat: boolean
  removeFinishedFromQueue: boolean
  locale: string
}
