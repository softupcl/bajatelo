# Especificación — Plan de pruebas

> Fase 2 · Fecha: 2026-10-19 · Estado: BORRADOR — pendiente de aprobación
> Regla marco (AGENTS.md §11): tests junto a cada feature; fakes/mocks para unitarias; fixtures autorizados para integración; nada se da por pasado sin ejecutarse. Sin dependencia de plataformas de terceros en la suite por defecto; sin descargas reales de redes sociales en la suite default.

## 1. Niveles y herramientas

| Nivel | Alcance | Herramienta | Ejecución |
|---|---|---|---|
| U1 — Unitarias TS | validación de URL, sanitizador de nombres, parser de progreso, i18n, stores, redacción de URLs, derivación de calidades | Vitest 5 | `npm run test:ts` |
| U2 — Unitarias Rust | máquina de estados, clasificación de errores, naming/conflictos, parseo de JSON de análisis, detectores de dependencias | `cargo test` | `npm run test:rs` (cargo test) |
| I1 — Integración adapters | adapters contra **fakes de proceso** (scripts locales controlados que simulan yt-dlp/ffmpeg: éxito, error, líneas de progreso, cuelgue → timeout, kill) | `cargo test` + scripts fixture | `npm run test:rs` |
| I2 — Integración real (marcada `#[ignore]`/`it:`) | yt-dlp/FFprobe/FFmpeg reales si están instalados; conversión de fixtures de medios propios generados por FFmpeg | cargo test / vitest | `npm run test:integration` (opt-in) |
| E1 — E2E frontend | estados de UI, navegación por teclado, accesibilidad (roles/aria-live), estados vacío/error/carga | Playwright 1.64 contra Vite dev con **backend simulado** (bridge inyectable que devuelve fixtures) | `npm run test:e2e` |
| E2 — E2E app (cuando aplique) | flujos completos con la app Tauri real y fixtures locales | `tauri-driver` + Playwright/WebDriver (AGENTS.md §3: "where suitable") | opt-in; requiere entorno Tauri operativo |

Decisión estructural: el `bridge.ts` (capa de servicios de UI) se inyecta, de modo que E1 funciona sin Rust; I1 verifica que el Rust real se comporta como el fake que E1 usa (contrato compartido).

## 2. Fixtures autorizados

- `tests/fixtures/analysis/*.json`: salidas `yt-dlp -J` sintéticas (YouTube-like, genérico, sin formatos, campos ausentes, títulos con caracteres hostiles, URLs con `list=`).
- `tests/fixtures/progress/*.txt`: secuencias de stdout del motor (progreso multi-archivo, tamaños K/M/G/T, `ERROR:` de cada clase del catálogo, líneas ruidosas).
- `tests/fixtures/media/`: clips pequeños generados por FFmpeg en el entorno de test (sintéticos, sin contenido de terceros) — p. ej. 2 s de color + tono; corrupto truncado; solo-audio.
- Prohibido: descargas de plataformas reales en la suite por defecto (AGENTS.md §11).

## 3. Matriz requisito → pruebas (cobertura mínima obligatoria)

| Requisito | Unitarias | Integración | E2E |
|---|---|---|---|
| RF-01 validación URL | tabla de URLs (válidas/`file:`/`javascript:`/vacío/`list=`) | validación Rust duplicada | pegar URL inválida → mensaje EC-0x |
| RF-02 análisis | parser JSON (fixtures), derivación/emparejamiento/dedup de calidades (REA B2/B3) | adapter `-J` con fake que cuelga → EC-12 | estados "Analizando…", ficha con datos fixture |
| RF-03 formato/calidad/destino | sanitizador (REA C2 + reservas del SO), conflictos | naming en Rust; diálogo nativo no testeado automáticamente (manual checklist) | combos habilitados/deshabilitados por estado |
| RF-04 progreso | parser de líneas (velocidad/ETA/porcentajes, tope 99,9), heurística >10% (REA §4.4) | fake de proceso que emite secuencia → eventos recibidos completos | columna de progreso con fixture stream |
| RF-05 cola/pausa/cancel/reintento | máquina de estados con `pausing/paused` (T1, T6) | pausa/reanudación con fake (continúa `.part`, sin duplicados), kill real de proceso falso (PID vivo/muerto), duplicados prohibidos (T3), limpieza temporales | pausar/reanudar/cancelar desde fila → estados correctos |
| RF-06 conversión | validaciones previas | I2 real con fixtures de medios: éxito, corrupto (EC-30), cancelado (EC-33), salida vacía (EC-34) | estados de la vista Convertir |
| RF-07 historial | redacción de credenciales (CA-07.3), migraciones de esquema | SQLite: transacciones, corrupción → recuperación con backup (EC-41) | historial persiste tras recarga |
| RF-08 diagnóstico | detectores con rutas/PATHe simulados | binarios falsos que fallan/hang → incompatible | banner de degradación (solo "Original") |
| RF-09 preferencias | defaults, guardado, migración de esquema | escritura atómica | persistencia tras reinicio (E1) |
| RF-10 accesibilidad/i18n | ausencia de literales fuera de i18n (test lint) | — | teclado completo, roles ARIA, contraste AA, aria-live |

Casos límite explícitos (de `docs/rea/03-user-flows.md` §"Casos límite"): sin descargas activas (botones off), búsqueda no aplica (fuera de MVP), red caída durante descarga (EC-20), vídeo sin formatos (EC-11), dependencias ausentes (RF-08), cierre con tareas activas (RF-05), URL de playlist (comportamiento definido por EC-10/EC-11 o análisis del vídeo — fijado en RF-01/RF-02), conflicto de nombres (RF-03).

## 4. Criterios de calidad y puertas

- Puerta de cada PR/coherencia de cambio: `npm run test:ts && npm run test:rs && npm run test:e2e` en verde + lint/format.
- Antes de declarar "Definition of Done" de una feature (AGENTS.md §15): resultados reales ejecutados y copiados en `docs/qa/regression-results.md` con comando exacto y fecha.
- Cobertura objetivo: ≥85% en módulos de lógica (parser/estados/naming); sin umbral arbitrario en UI pura, pero estados cubiertos por E1.
- Validación diferencial REA (AGENTS.md §12): para cada feature implementada se contrasta contra el flujo de referencia de `docs/rea/03-user-flows.md` y se registra en `docs/qa/differential-report.md` con estado `passed/failed/blocked/not-tested`.
- Pruebas manuales no automatizables (diálogos nativos, empaquetado por SO) → checklist documentada en `docs/qa/`.

## 5. Riesgos de prueba

- Depender de yt-dlp/FFmpeg reales en CI → mitigado por niveles I1 (fakes) como default y I2 opt-in.
- E2E de Tauri (WebDriver) puede requerir ajustes por SO → E1 desacopla el valor principal; E2 se documenta como mejora continua.
- Procesos zombies en tests de cancelación → asserts de vida por PID y limpieza en `Drop`/afterEach.
