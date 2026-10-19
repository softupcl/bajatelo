# QA — Informe de validación diferencial (R-31)

> Fecha: 2026-10-19 · Método: comparación de **flujos observables** contra la guía de referencia `docs/rea/03-user-flows.md` (AGENTS.md §12).
> Estados: `passed` · `failed` · `blocked` · `not-tested`. Nada se marca pasado sin evidencia ejecutada (ver `regression-results.md`).
> Alcance de evidencia: pruebas automatizadas (cargo test 63+1, Vitest 11, Playwright 7) + **smoke real de la app en `tauri dev`** (PID 27307: arranque y parada limpios) + prueba I2 real del adapter FFmpeg/FFprobe.

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

---

# Informe de la FASE 6 · Validación diferencial con apps en ejecución (2026-10-19)

> Método del enunciado de fase 6: ejecutar la app, recorrer los flujos documentados, contrastar navegación/estados/controles/mensajes, comprobar que la descarga y la conversión son reales, ejecutar pruebas y **corregir lo reproducible**. Todo resultado proviene de ejecuciones de esta sesión (AGENTS.md §2); no hay capturas inventadas ni descargas no realizadas.

## 6. Matriz de validación por requisito (identificador · esperado · observado · evidencia · estado · severidad · corrección)

| ID | Esperado | Observado | Evidencia | Estado | Severidad | Corrección |
|---|---|---|---|---|---|---|
| **RF-01** | URL validada con protocolo http(s); rechazo de `file:`/`javascript:`/rutas locales con EC-01/02/03 | Comportamiento correcto en los 8 tests +座谈 E2E (etiqueta/desactivación honesta) | `validation.rs` 8 tests; e2e `accessibility.spec.ts` #2 | ✅ aprobado | — | — |
| **RF-02** | Análisis real (`yt-dlp -J`) con título/duración/portal/formatos reportados | **Hallazgo D1**: el puente tipaba como `MediaInfo` lo que el backend devuelve (`AnalysisOutcome{taskId,media,error}`) → **en runtime real la ficha quedaba vacía tras analizar**; el e2e pasaba porque el doble devolvía `MediaInfo` directo | Diff código `bridge.ts` vs `commands/tasks.rs::analyze_url`; corrección verificada con e2e #3 con contrato del doble actualizado | ❌ fallido → ✅ **corregido** | **Alta (funcional)** | Puente tipado con `AnalysisOutcome` (contrato añadido a `contracts.ts`); doble E2E actualizado; prueba de regresión E2E |
| **RF-03** | Combos de formato/calidad con opciones reales; destino con diálogo nativo; política de conflicto (RF-03/D-04) | Combos correctos; **no existía forma de iniciar descarga** (sin botón `startDownload`, sin `pickSaveDestination` ni cola en la UI de Descargas) — el backend D-12 estaba probado pero desconectado de la UI (hallazgo D2) | Código `DownloadsPage.vue` pre-corrección (solo análisis + banner "descarga pendiente"); backend con 6 comandos sin llamador | ❌ fallido → ✅ **corregido** | **Alta (funcional)** | **Reescritura de la vista Descargas**: botón Descargar → diálogo → `start_download(taskId, calidad, formato, destino, política)` desde preferencias; cola viva con `get_tasks` + eventos `task://state`/`task://progress`; acciones por fila Pausar/Reanudar/Detener/Reintentar/Abrir carpeta (REA F5.1/F5.6); E2E nuevo #7 |
| **RF-04** | Progreso con % verificado por bytes en disco, velocidad, tamaño y ETA; conversión sin % (honesto) | Corregido al construir la cola: % calibrado contra bytes reales (REA §4.4), tope 99,9% hasta confirmar fin; velocidad/ETA del motor; etiquetas REA F6.1 | Parser 7 tests + e2e #7 (fila con progresos del doble); descarga real contra red **not-tested** (ver §8) | ⚠️ parcial → ✅ (automatizable) | Media | — |
| **RF-05** | Cola con transiciones T1–T6; pausa/reanudación; cancelación con limpieza; reintento sin duplicados | Máquina de estados probada; **cancelación durante conversión honrada (corregido en la pasada anterior)**; ahora también accesible desde la UI (faltaba el cableado del puente) | 63 tests Rust; conversión con kill real | ✅ aprobado | — | — |
| **RF-06** | Conversión local real con FFprobe/FFmpeg; verificación de salida antes de success; abrir archivo/carpeta | I2 real en verde (probe + MP4 + MP3 + EC-30 + vídeo sin audio); `open_item` con EC-43; vista con botón Detener | `ffmpeg.rs` test I2 (opt-in); `ConversionPage.vue` | ✅ aprobado | — | — |
| **RF-07** | Historial con eliminar/vaciar y paridad REA F5.6 (abrir archivo/carpeta) | Acciones **Abrir archivo/Abrir carpeta añadidas** a las entradas `completed` del Historial | `HistoryPage.vue` + `open_item` | ✅ aprobado (acción nueva verificada por firma; click nativo = manual) | Baja | — |
| **RF-08** | Diagnóstico real sin motores con guía por SO | e2e #5 en verde (tabla con 3 herramientas y versiones del doble) | e2e accesibilidad #5 | ✅ aprobado | — | — |
| **RF-09** | Preferencias persistentes aplicadas | Lectura de `conflictPolicy` en el inicio de descarga (nuevo); resto de ajustes ya probados | `DownloadsPage.vue` (nuevo) + `store.rs` 6 tests | ✅ aprobado | — | — |
| **RNF-01** | Ejecución con lista de argumentos, sin shell concatenado | `build_download_arguments` exacto con test; `open_item` ejecuta vía plugin sin shell | `task_manager.rs` tests; `convert.rs::open_item` | ✅ aprobado | — | — |

## 7. Defectos corregidos durante la fase 6 (prioridad del enunciado)

1. **[P1 · Funcional] D1 — análisis real devolvía datos no consumibles por la UI**: `analyzeUrl` del puente tipaba `MediaInfo`, pero el comando devuelve `AnalysisOutcome`. Corregido tipando el contrato y consumiendo `taskId`; el estado de tarea real (creado por el backend en el análisis) ya no se descarta.
2. **[P2 · Funcional] D2 — la UI de Descargas no iniciaba descargas ni mostraba la cola**: Vista reescrita con botón habilitado tras el análisis, diálogo de destino, cola viva con eventos tipados y acciones por fila (REA F5.1/F5.6). La prueba de regresión E2E #7 cubre fila, estado, progreso y acciones.
3. **[P3 · Cancelación/progreso] D3 (de la pasada anterior)** — cancelación honrada durante la conversión con kill y borrado de la salida parcial; verificado con proceso POSIX y con el fixture real I2.
4. **[P5 · Visual/documento]** el test E2E anterior que declaraba la cola "no integrada (D-12)" fue actualizado: ahora exige botón "Descargar" habilitado y fila con progreso.

## 8. Defectos restantes (diferencias documentadas, no reproducibles automáticamente)

| ID | Descripción | Severidad | Bloqueo |
|---|---|---|---|
| G1 | **Descarga real contra YouTube no ejecutada**: el ciclo completo con red (F4 → progreso → `completed`) requiere contenido autorizado del usuario y su conexión; la suite NO debe depender de terceros (AGENTS.md §11). El mecanismo está verificado por contrato con pruebas backend + E2E con doble. | Media | manual-checklist §2 |
| G2 | Progreso de conversión indeterminado (sin granualidad FFmpeg): comportamiento honesto pero mejorable con `-progress`. | Baja | mejora futura (X) |
| G3 | Ajustes `neverAskFilename` / `alwaysUseLastPath` no se aplican al diálogo de destino en el flujo nuevo; siempre se pide el nombre (más seguro por defecto). Prefijo de mejoras de configuración; no rompe CA. | Baja | backlog ajustes |
| G4 | Acciones del menú contextual de la referencia no replicadas: *Copy video link* / *Open video link in browser* (post-MVP X-02...). | Baja | post-MVP (AGENTS.md §5.2) |
| G5 | VoiceOver con lector real: not-tested (requires human) | Baja | manual-checklist §3 |

## 9. Registro de ejecución de la app (procedimiento fase 6, paso 1)

- `npm run tauri dev` → `Running target/debug/rea` con el ejecutable recién compilado del estado fase 5/6 (PID 27307), parada limpia verificada con `ps`/`kill`. Sin errores ni warnings en el log de arranque (`target/debug/rea` termina con éxito al detener).
- La suite automatizada: **cargo 63/63 (+I2 1/1), Vitest 11/11, Playwright 7/7, lint y typecheck limpios** (ver `regression-results.md` §"Fase 6").

## 10. Veredicto fase 6

- Sin estados `failed` permanentes; **2 defectos funcionales de alta severidad corregidos en esta pasada** (D1 y D2, sección 7).
- Descargas y conversiones reales automatizables: `passed`. La **descarga real contra YouTube con red y contenido autorizado del usuario** queda `blocked`/`not-tested` hasta su ejecución manual (G1, checklist §2). No se afirma haber probado ningún sitio con descarga autorizada: no se realizó.
