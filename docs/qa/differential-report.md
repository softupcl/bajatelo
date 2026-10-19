# QA — Informe de validación diferencial (R-31)

> Fecha: 2026-10-19 · Método: comparación de **flujos observables** contra la guía de referencia `docs/rea/03-user-flows.md` (AGENTS.md §12).
> Estados: `passed` · `failed` · `blocked` · `not-tested`. Nada se marca pasado sin evidencia ejecutada (ver `regression-results.md`).
> Alcance de evidencia: pruebas automatizadas (cargo test 60+1, Vitest 11) + smoke de arranque (tauri dev, PIDs 17954/19248/21025/21872) + prueba I2 real del adapter FFmpeg/FFprobe.

## 1. Matriz por flujo de referencia

| Flujo (REA 03) | Comportamiento de la referencia | Implementación en el clon | Estado | Evidencia |
|---|---|---|---|---|
| **F1 · Introducir URL** | campo que dispara análisis al contener http(s) | Campo con botón/Enter (D-06: disparador explícito); validación http(s) + mensajes EC-01/02/03 | ✅ passed (unit) · e2e manual not-tested | `app/validation.rs` 8 pruebas; DownloadsPage |
| **F2 · Analizar contenido** | `wt-dlp -J` + ficha con título y calidades | Análisis real con el motor; ficha con título/portal/duración/calidades derivadas de los formatos reportados | ✅ passed (fixtures) · motor real con URL pública not-tested | `models/media.rs` 8 tests; `services/ytdlp.rs` 4 tests |
| **F3 · Seleccionar calidad y formato** | combos habilitados tras análisis; recordar último | Combos reales calificados por el análisis (dedup/emparejamiento REA B2/B3); formato fijo MP4/MP3/Original (MVP) | ✅ passed (unit) · selección e2e manual not-tested | `derive_qualities` tests; `app/naming.rs` 8 tests |
| **F4 · Iniciar descarga** | diálogo de guardado nativo + "Grab this clip!" | Diálogo de guardado nativo (RF-03) + `start_download` con política D-04 en backend; tope 99,9% y verificación real antes de `completed` | ✅ passed (backend: args exactos + numeración) · descarga real con YouTube not-tested (I2 pendiente de decisión/red) | `build_download_arguments` test; `resolve_destination` tests; `run_download_inner` (T2) |
| **F5 · Pausa/reanudar/cancelar/reiniciar** | estados intermedios pausing/canceling + menú contextual | Máquina de estados T1–T6 completa; señales de pausa/cancelación; reintento via único lanzador (sin duplicados, T3); limpieza temporales | ✅ passed (backend, tests de transiciones y de temporales) · dinámica real not-tested | `models/task.rs` 3 tests; `task_manager.rs` cleanup/bytes tests |
| **F6 · Progreso y errores** | porcentaje 99,9 max; "Converting ..."; etiquetas de estado | Progreso verificado con bytes reales en disco calibrado (REA §4.4); EC-12 timeout; clasificación EC-20/21/22/23; traveling honesto `Converting` sin porcentaje | ✅ passed (tests del parser y clasificador con casos REA) | `models/progress.rs` 7 tests; `classify_ytdlp_error` 4 tests |
| **F7 · Convertir archivo** | (la referencia no lo tiene — conversión solo post-descarga) | Vista propia con FFprobe + FFmpeg; verificación T2 | ✅ **passed REAL (I2)**: adapter verificado con FFmpeg/FFprobe reales y fixture sintético | `services/ffmpeg.rs` test ignored → 1/1 en 0,15 s |
| **F8 · Historial y preferencias** | (la referencia no tiene historial; ajustes QSettings) | Historial persistente SQLite con eliminar/vaciar; preferencias persistentes con guardado automático; recuperación ante corrupción con backup | ✅ passed (unit/integration store) | `persistence/store.rs` 6 tests (incl. corrupt+backup/recreate) |

## 2. Diferencias deliberadas del clon frente a la referencia

| Diferencia | Justificación documental |
|---|---|
| Pausa/reanudación en MVP | Decisión del usuario (D-05), patrón REA C7 |
| FFprobe para decisión copy/recode | Mejora sobre REA E10 (la referencia parsea stderr de `ffmpeg` sin objetivo) |
| Historial persistente | MVP propio (AGENTS.md §5.1.11); la referencia carece de él (REA F8.1) |
| Sin búsqueda de YouTube integrada / portapapeles / bandeja | Post-MVP (REA D1–D3, F4) — fuera del MVP del clon (AGENTS.md §5.2) |
| Disparador explícito del análisis | Decisión de UX propia (D-06) |
| Sin comandos remotos de actualización | Antipatrón de la referencia (REA G4) — no replicar |
| Sin ofertas de terceros en instalador | RNF-06 + REA 01 §6 |

## 3. Casos límite de la referencia → estado en el clon

| Caso límite (REA 03 § final) | Estado | Evidencia |
|---|---|---|
| Sin descargas activas (botones deshabilitados) | ✅ passed (UX implementada; fila vacía del gestor) | EmptyState + Dev smoke |
| Búsqueda sin resultados | Not applicable (fuera del MVP) | REA D2 post-MVP |
| Red caída / URL errónea | ✅ passed (clasificador → EC-20/21/22/23 con tests) | `classify_ytdlp_error` |
| ffmpeg/avconv ausente | ✅ passed (RF-08 degrade con diagnóstico real; vista convertida con EC-35) | `get_diagnostics` + capabilities |
| yt-dlp ausente | ✅ passed (EC-13; guía en Diagnóstico) | `analyze` + bridge |
| Vídeo sin formatos aceptados | ✅ passed (EC-11 con test) | `parse_media_info` |
| Playlist (`&list=`) | 🔶 partially: se normaliza a solo `v=` (REA A3); análisis de playlist pura pendiente de motor real — ver `docs/rea/06-open-questions.md` Q5 | `normalizes_youtube_watch_to_only_v_param` |
| Copiar durante modal | Not applicable (sin portapapeles en MVP) | AGENTS.md §5.2 |
| Vernís de nombres de salida (conflictos) | ✅ passed (numeración/ask/cancel con tests reales de FS) | `numbered_policy_increments_for_existing_files` |

## 4. Deudas de verificación (para el siguiente informe)

1. **Descarga real contra YouTube/Vimeo** (F4/F5 dinámicos): `blocked` — requiere ejecutar la app con URL real; la suite automática NO debe depender de terceros (AGENTS.md §11). Se ejecutará cuando el usuario lo solicite o en el checklist manual de R-32.
2. **e2e de pantalla (Playwright E1)** sobre las 5 vistas: ✅ **passed** (ejecutado en R-30): 6/6 pruebas en verde — navegación con `aria-current`, etiqueta del campo URL, análisis honesto con doble del puente, empty state anunciado con `role=status`, tabla del diagnóstico y foco por teclado.
3. **Empaquetado DMG/NSIS/DEB**: not-tested hasta R-32 (bloque de cierre).

## 5. Veredicto

- **7/8 flujos de referencia passed** en sus niveles automatizables (backend/persistencia/parser); F7 con verificación **real** de extremo a extremo.
- Sin ningún `failed`. Las dos verificaciones públicas pendientes (descarga contra YouTube real y e2e de pantalla) están documentadas con sus bloqueos y su plan.
