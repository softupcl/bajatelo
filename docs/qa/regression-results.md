# QA — Resultados de regresión (R-31)

> Última ejecución: 2026-10-19 · Todas las cifras de este documento provienen de ejecuciones reales capturadas en la sesión (AGENTS.md §2, §11).
> Entorno verificado: macOS (Apple Silicon), Node v24.21.0, Rust estable **1.99.1** (2026-10-15), Tauri **2.12.4**, Vue **3.5.44**, Vitest **5.0.5**, Playwright **1.64.1** (instalado como devDep), yt-dlp **2026.8.19** (Homebrew), FFmpeg/FFprobe **9.0.2_1** (Homebrew), SQLite embebido vía `rusqlite 0.40` (`bundled`).

## Ejecución completa de la suite

| # | Comando exacto | Resultado | Fecha |
|---|---|---|---|
| 1 | `cargo test --manifest-path src-tauri/Cargo.toml` | ✅ `test result: ok. 60 passed; 0 failed; 1 ignored` (3 suites) | 2026-10-19 |
| 2 | `cargo test --manifest-path src-tauri/Cargo.toml -- --ignored` (I2 real FFmpeg/FFprobe) | ✅ `test result: ok. 1 passed; 0 failed` en 0,15 s | 2026-10-19 |
| 3 | `npm test` (Vitest) | ✅ `Test Files 2 passed (2)` · `Tests 11 passed (11)` | 2026-10-19 |
| 4 | `npm run typecheck` (vue-tsc --noEmit) | ✅ sin salida de error | 2026-10-19 |
| 5 | `npm run lint` (ESLint + Prettier check) | ✅ `All matched files use Prettier code style!` · 0 errores | 2026-10-19 |
| 6 | `npm run build` (vue-tsc + vite build) | ✅ `✓ built in 84ms` | 2026-10-19 |
| 7 | `npm run tauri dev` (smoke arranque app) | ✅ `Running target/debug/rea` — PIDs verificados: 17954, 19248, 21025, 21872 (todas las sesiones detenidas limpiamente) | 2026-10-19 |
| 8 | Flujo nativo FFmpeg manual (fixture `testsrc2+sine` 1 s) | ✅ ffprobe `format_name=mov,mp4,m4a,3gp,3g2,mj2` + `duration=1.000000`; conversiones MP4 (51 KB) y MP3 (34 KB) no vacías | 2026-10-19 |
| 9 | Verificación de la base de datos del sistema en frío | ✅ `~/Library/Application Support/com.softnext.rea/rea.db` con WAL; `PRAGMA user_version = 1`; tablas `history`, `preferences`, `tasks` | 2026-10-19 |

## Cobertura por área (AGENTS.md §11)

| Área mínima obligatoria | Estado |
|---|---|
| URL validation y protocolos no soportados | ✅ 8 tests (EC-01/02/03 + normalización YouTube REA A3) |
| Metadata parsing y campos ausentes | ✅ 8 tests (EC-10/EC-11, fixtures con campos vacíos/hostiles) |
| Comportamiento de la cola y transiciones legales de estado | ✅ 3 tests de máquina de estados (legales completas + ilegales muestreadas + serialización) |
| Gestión de eventos de progreso | ✅ 7 tests del parser (Destination/Merged/Already/percent/ETA/ERROR) |
| Cancelación, reintentos y limpieza de procesos | ✅ backend: único lanzador (T3), señales T4/T6, cleanup bounded por prefijo de tarea |
| Herramientas de medios ausentes y diagnóstico de versiones | ✅ `tool_locator` 5 tests + degradación RF-08 |
| Éxito y fallo de conversión | ✅ unit (compatibilidad códecs) + **I2 real** (MP4/MP3 con FFmpeg) |
| Conflictos de salida, paths inválidos, filenames saneados | ✅ 8 tests de naming (incl. traversal imposible por construcción) |
| Persistencia del historial y preferencias | ✅ 6 tests del store (round-trip, backup+recuperación, recreate) |
| Estados de UI vacíos/cargando/error/completado | 🔶 passed en construcción (EmptyState/estados); e2e automatizado pendiente (R-30) |

## Registros ejecutados y corregidos durante esta fase (transparencia)

- `gen` palabra reservada de Rust (edition 2024) → variable renombrada a `generated` en el test I2.
- `#[tokio::test]` añádido al test async I2.
- `SaveDialogOptions` no acepta `multiple` → opción eliminada del puente.
- `eta_to_seconds`: `?` dentro del closure de `.map()` → reescrito con bucle de parseo seguro.
- `parse_motor_line`: prefijo `[download]` del `already been downloaded` y tokens `~` sueltos de la estimación de tamaño → corregidos con tests.
- Sanado (sanitize_filename): `is_control` → espacio (no eliminación) y `..+` interior → `.`; traversal imposible por construcción (tests actualizados a la semántica exacta).
- Un cierre `}` singular en `run_analysis` causando desbalance de braces en el fichero central → corregido en el proceso.

## Nota honesta sobre la suite

- Las verificaciones 1–6 son repetibles localmente sin red ni plataformas de terceros (AGENTS.md §11).
- La única prueba con binarios reales de medios (I2) es **opt-in** (`--ignored`) y usa un fixture sintético generado por el propio test; no descarga contenido de terceros.
- La descarga real contra YouTube (flujo F4/F5 con red) NO ha sido automatizada y sí debe verificarse manualmente antes de release (R-32 checklist).
