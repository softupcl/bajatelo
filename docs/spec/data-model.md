# Especificación — Modelo de datos

> Fase 2 · Fecha: 2026-10-19 · Estado: BORRADOR — pendiente de aprobación
> Persistencia: **SQLite embebido** vía `rusqlite` (feature `bundled`; sin dependencia del sistema operativo) en el directorio de datos de la app (**D-07 aprobada**). Journal mode WAL, escritura transaccional y recuperación tolerante a corrupción.
> Los tipos existen en paralelo: Rust (`models/`) y TypeScript (`types/`) — definición única conceptual, duplicada con tests de serialización cruzada.

## 1. Máquina de estados de tarea (DownloadTask)

Estados (AGENTS.md §8 + pausa de REA C7, **D-05 aprobada**): `queued · inspecting · ready · downloading · pausing · paused · converting · completed · failed · cancelled`.

Transiciones legales:

```
queued      → inspecting | failed | cancelled
inspecting  → ready | failed | cancelled
ready       → downloading | cancelled
downloading → converting | completed | failed | cancelled | pausing
pausing     → paused | cancelled
paused      → downloading (reanudar) | cancelled
converting  → completed | failed | cancelled
completed   → (∅ terminal)
failed      → downloading   (reintento: vuelve a lanzar con la misma configuración)
cancelled   → downloading   (reintento)
```

Reglas verificables:
- T1. Toda transición ilegal se rechaza en el `task_manager` (los estados son un enum con método `can_transition_to`).
- T2. `completed` exige verificación previa (exit 0 + archivo existe + tamaño > 0). Prohibido `completed` por simple arranque o salida parcial (AGENTS.md §8).
- T3. Reintento (`failed|cancelled → downloading`) solo si no hay proceso vivo asociado a la tarea (registro de PID/vida).
- T4. Cancelación en cualquier estado no-terminal → kill del proceso → limpieza de temporales y listeners → `cancelled`.
- T5. Cada transición emite evento a la UI y actualiza `updatedAt`.
- T6. Pausa: `terminate()` del proceso (patrón REA §4.4); el estado `pausing` solo pasa a `paused` cuando se confirma la muerte del proceso; reanudar relanza la descarga con la misma configuración y plantilla de salida, continuando los archivos `.part` — nunca con proceso duplicado (complementa T3).

## 2. Entidades

### 2.1 DownloadTask (memoria, no persistida íntegra)
| Campo | Tipo | Notas |
|---|---|---|
| id | string (UUID v4) | estable durante reintentos |
| sourceUrl | string | redactada en logs/historial (sin `user:pass@`) |
| portal | string | del extractor (p. ej. "youtube"), REA A2 |
| title | string \| null | del análisis |
| durationSec | int \| null | del análisis |
| thumbnailUrl | string \| null | opcional; no se persiste |
| qualities | QualityOption[] | solo las reportadas (RF-02) |
| selectedQualityId | string \| null | |
| selectedFormat | OutputFormat | `mp4 · mp3 · original` |
| destinationPath | string | validado |
| state | TaskState | §1 |
| progress | ProgressInfo \| null | §2.2 |
| error | TaskError \| null | código del catálogo + detalle técnico opcional |
| createdAt/updatedAt/startedAt/finishedAt | datetime | |
| processIds | list<int> | para vida/cancelación (T3) |
| retryCount | int | |

### 2.2 ProgressInfo
`percent (0–99.9 | null)`, `bytesDownloaded`, `bytesTotal (null si indeterminado)`, `speedBps (null)`, `etaSeconds (null)`, `phase: downloading|converting`. Nulos = indeterminado visible (RF-04); prohibido fabricarlos.

### 2.3 ConversionTask
Como DownloadTask sin `portal/qualities`: `inputPath` (validado), `inputInfo` (FFprobe: contenedor, códecs, duración), `selectedFormat`, `destinationPath`, estado/error/timestamps.

### 2.4 HistoryEntry (persistida)
| Campo | Tipo | Notas |
|---|---|---|
| id | string | UUID |
| kind | `download · convert` | |
| title | string | |
| portal | string \| null | — en conversiones |
| sourceUrl | string \| null | **redactada** (RF-07) |
| format | string · quality | string \| null |
| destinationPath | string | |
| finalSizeBytes | int \| null | solo si verificada |
| status | `completed · failed · cancelled` | |
| errorCode | string \| null | catálogo §3 |
| startedAt / finishedAt | datetime | |

### 2.5 Preferences (persistida)
| Clave | Tipo | Por defecto | Base REA |
|---|---|---|---|
| defaultDestination | string | SO (Descargas) | F8.2 |
| alwaysUseLastPath | bool | true | F8.2 |
| neverAskFilename | bool | false | C3 |
| conflictPolicy | `ask · number · cancel` | `ask` | RF-03 / D-04 |
| rememberQuality · rememberFormat | bool | true · true | B4 |
| removeFinishedFromQueue | bool | false | F8.2 |
| locale | string | `"es"` | AGENTS.md §10 |
| schemaVersion | int | actual | migraciones |

### 2.6 DiagnosticsReport (memoria)
Por herramienta (`ytdlp · ffmpeg · ffprobe`): `status: ok · missing · incompatible`, `version`, `path`, `rawVersionOutput` (opcional, sin secretos).

### 2.7 MediaInfo / QualityOption (del análisis)
`MediaInfo`: título, portal, duración, urlNormalizada. `QualityOption`: `id` (format_id del motor), `label` (`1080p (HD)` — REA B2), `height`, `fps`, `container`, `videoFormatId/audioFormatId` (emparejados, REA B3), `estimatedBytes` (si el motor lo da), `audioOnly: bool`.

## 3. Almacenamiento

```
app_data_dir/
  rea.db            # SQLite (WAL) — en uso se generan rea.db-wal / rea.db-shm
```

Esquema v1 (migraciones numeradas en `persistence/migrations.rs`):

```sql
CREATE TABLE history (
  id TEXT PRIMARY KEY,
  kind TEXT NOT NULL,              -- 'download' | 'convert'
  title TEXT NOT NULL,
  portal TEXT,
  source_url TEXT,                 -- redactada (CA-07.3)
  format TEXT, quality TEXT,
  destination_path TEXT NOT NULL,
  final_size_bytes INTEGER,
  status TEXT NOT NULL,            -- 'completed' | 'failed' | 'cancelled'
  error_code TEXT,                 -- catálogo error-catalog.md
  started_at TEXT NOT NULL,
  finished_at TEXT NOT NULL
);

CREATE TABLE preferences (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE TABLE tasks (              -- cola persistida para recuperación tras cierre
  id TEXT PRIMARY KEY,
  payload_json TEXT NOT NULL,
  state TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
```

- `rusqlite` con feature `bundled` (SQLite se compila con la app; **no requiere instalación del SO**) y journal mode WAL.
- Recuperación: al abrir, `PRAGMA integrity_check`; si falla → restaurar `rea.db.bak` (copia de seguridad al cerrar limpiamente) o recrear la base con aviso visible (EC-41); nunca crash.
- Redacción: helper único `redact_url()` usado por historial y logs (CA-07.3).
- Sin secretos en la base de datos (RF-09/CA-09.3).

## 4. Contratos UI ↔ Rust (comandos y eventos)

Comandos: `analyze_url(url) → MediaInfo+qualities · start_download(taskId, qualityId, formatId, dest) · cancel_task(taskId) · retry_task(taskId) · convert_file(input, format, dest) · probe_file(input) → MediaInfo · get_diagnostics() → DiagnosticsReport[] · get/set preferences · history: list/remove/clear`.
Eventos: `task://state {taskId, state, error?}`, `task://progress {taskId, ProgressInfo}`.

Todos los errores hacia la UI usan el catálogo (`error-catalog.md`) — nunca texto técnico crudo como mensaje principal (AGENTS.md §10).
