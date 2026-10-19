# Estado del proyecto — Mapa de fases (documento vivo)

> Última actualización: 2026-10-19 · Este documento fija el orden de trabajo y el estado de cada fase, con su evidencia (commits y pruebas ejecutadas). Le sigue `docs/spec/mvp-backlog.md` (detalle de items) y `docs/qa/` (resultados).
> metodología: REA descrito en `AGENTS.md` §1; las fases siguen el orden sugerido en §16, adaptado a la trazabilidad real de esta sesión.

## 1. Mapa de fases

| Fase | Alcance definido en | Contenido | Commit(es) | Estado |
|---|---|---|---|---|
| **1 · Investigación REA** | Enunciado fase 1 + AGENTS.md §4.4 | `docs/rea/` completo (01-research … 06-open-questions): sitio oficial + código fuente GPLv3 de ClipGrab 3.9.14 analizado; limitaciones documentadas (sin app en ejecución local) | `b149417` | ✅ completa |
| **2 · Especificación** | Enunciado fase 2 + AGENTS.md §13 | `docs/spec/` completo (requirements, architecture, data-model, error-catalog, test-plan, mvp-backlog) + stack verificado con versiones reales del día; decisiones **D-01…D-08 aprobadas** (npm, TS estable, sin bundling de motores, política de conflicto preg untar, **pausa en MVP**, disparador explícito, **SQLite**, entorno a instalar) | `b149417` + ediciones de decisiones | ✅ completa — aprobada |
| **3 · Scaffold del proyecto** | Enunciado fase 3 + AGENTS.md §6 | Proyecto **Tauri 2 (2.12.4) + Vue 3 (3.5.44) + TS + Rust** generado con `create-tauri-app`; estructura `src/{features,layouts,pages,services,types,i18n}` + `src-tauri/{commands,services,models,capabilities}`; navegación de las 5 vistas; estados vacíos honestos; contratos UI↔Rust sincronizados; i18n es; **diagnóstico real** de dependencias (get_diagnostics); Vitest + ESLint + Prettier; capabilities mínimas | `5393ec6` | ✅ completa — **criterios de finalización cumplidos**: app inicia en `tauri dev` (PIDs verificados), frontend compila (vue-tsc+vite), Rust compila (cargo), pruebas iniciales pasan (7 Vitest + 5 Rust), instrucciones en README, sin ejecución arbitraria de comandos |
| **4 · Implementación del MVP** (post-scaffold; realizada por bloques del backlog aprobado en fase 2) | `docs/spec/mvp-backlog.md` B0–B5 | **Bloque 0** entorno (Rust 1.99.1, yt-dlp 2026.8.19, FFmpeg 9.0.2, git) · **Bloque 1** shell+contratos+máquina de estados · **Bloque 2** V-01 URLs, V-02 diagnóstico, V-03 SQLite · **Bloque 3** D-10/D-11 análisis+UI, D-12 cola con pausa/cancel/retry, D-13 historial · **Bloque 4** C-20/C-21 conversión local real | `130c144`, `6943cf3`, `386843b` | ✅ completa |
| **5 · Cierre del MVP** | Enunciados R-30/R-31/R-32 + AGENTS.md §12, §15 | R-31 validación diferencial (`docs/qa/differential-report.md` — 7/8 flujos passed, sin failed) · R-30 accesibilidad e2e (Playwright 6/6) · R-32 empaquetado real DMG (2,93 MiB, montado y verificado) + licencias (`docs/qa/licensing-notes.md`) | `23c0fa3`, `392c4b9`, `c047c7e` | ✅ completa |

> **Estado global: FASES 1–5 COMPLETADAS. El clon REA es un MVP funcional y empaquetado para macOS.**

## 2. Evidencia consolidada (todo ejecutado y capturado en `docs/qa/regression-results.md`)

- `cargo test` → **60/60** (0 warnings); + I2 real de FFmpeg/FFprobe con fixture sintético → **1/1** (opt-in `--ignored`)
- `npm test` (Vitest) → **11/11**
- `npx playwright test` (E1 accesibilidad/teclado/estados con doble del puente) → **6/6**
- `npm run typecheck` / `npm run lint` / `npm run build` → limpio, 0 errores/0 warnings
- `hdiutil` del DMG → checksums CRC32 verificadas, contiene `REA.app`; `tauri dev` smoke × 4 con arranques/paradas limpias
- BD del sistema en frío → `rea.db` con WAL, `user_version=1`, tablas `history/preferences/tasks`

## 3. Verificaciones pendientes (deudas documentadas)

| # | Deuda | Bloqueada por | Dónde resolverla |
|---|---|---|---|
| 1 | Descarga real contra YouTube (F4/F5 dinámicos con red) | ✅ resuelta (2026-10-19): prueba manual del usuario — vídeo y MP3 funcionando (ver manual-checklist §2) | — |
| 2 | Builds NSIS (Windows) y DEB/AppImage (Linux) | ✅ resuelta (2026-10-19): generados en CI (run 39313558162); ejecución behavioral en el SO objetivo: not-tested | release-checklist §2 |
| 3 | Firma/notarización macOS | requiere certificado del usuario | `docs/qa/licensing-notes.md` §4 |
| 4 | Playlists puras (REA Q5) | observación con motor real | prueba manual del checklist |
| 5 | VoiceOver con lector real | prueba humana | manual-checklist §3 |

## 4. Orden del trabajo futuro (post-MVP — solo con aprobación)

Backlog X-01…X-09 en `docs/spec/mvp-backlog.md`: portapapeles (X-02), drag & drop (X-03), formatos extra (X-04), notificaciones/bandera (X-05), proxy/IPv4 (X-06), metadatos ID3 editables (X-07), sidecar de motores en modo de revisión de licencias (X-08), auto-actualización con SHA-256 (X-09) y pausa ya quedó en MVP (D-05).

Regla de avance (AGENTS.md §2/§16): no se entra en un materially nueva fase sin revisar los resultados de la actual.

## 5. Historial de la confusión resuelta

La Fase 3 (scaffold) terminó en `5393ec6` con su informe; con las aprobaciones sucesivas ("procede") se continuó por los bloques del backlog **sin cambiar el nombre de fase**, lo que generó la duda resuelta por este documento. A partir de ahora, cualquier avance nuevo se anota aquí (§6 Registro).

## 6. Registro de cambios de estado

| Fecha | Cambio |
|---|---|
| 2026-10-19 | Documento creado; fases 1–5 marcadas completas con sus commits y evidencia |
| 2026-10-19 2 | 2ª pasada de revisión con los enunciados literales del usuario (fases 4 y 5): corregida la cancelación durante la conversión (EC-33, kill real de FFmpeg, salida parcial borrada), añadido `open_item` (req. abrir archivo/carpeta, EC-43) con botones en Convertir, I2 ampliado (EC-30, vídeo sin audio) y regresión de lint corregida. Suite completa en verde: 63/63 Rust, I2 1/1, Vitest 11/11, Playwright 6/6, lint/typecheck limpios. Evidencia en `docs/qa/regression-results.md` §"2ª pasada" |
| 2026-10-19 3 | **FASE 6** — validación diferencial ejecutando la app: hallados 2 defectos funcionales de alta severidad y corregidos: (D1) puente `analyzeUrl` mal tipado (`MediaInfo` vs `AnalysisOutcome`) que dejaba la ficha vacía en runtime real; (D2) UI de Descargas sin capacidad de iniciar descargas ni cola (backend D-12 desconectado). Acciones Abrir archivo/carpeta añadidas a Historial (REA F5.6). Evidencia y matriz por requisito en `docs/qa/differential-report.md` §6–§10; acciones en `docs/qa/regression-results.md` §fase 6. Suite final: cargo 63/63 (+I2 1/1), Vitest 11/11, Playwright **7/7**, lint/typecheck limpios, smoke real `tauri dev` con parada limpia |
| 2026-10-19 4 | **FASE 7** — release: build de producción; DMG macOS regenerado y verificado (monta, arm64, sin rutas absolutas, smoke del binario limpio); redacción sistemática de credenciales en errores (4 tests nuevos, 67/67); asistente de diagnóstico con guía por SO; `THIRD-PARTY-NOTICES.md` reproducible (472 crates); README de usuario final (instalación/actualización/desinstalación/troubleshooting) y requisitos NSIS/DEB/AppImage documentados; checklist de release en `docs/qa/release-checklist.md`. Pendientes: firmar/notarizar (certificado del usuario), builds Windows/Linux por host, publicación solo con autorización explícita |
| 2026-10-19 5 | **Renombrado REA → Bajatelo** (solicitud del usuario): productName/identifier de Tauri (`com.softnext.bajatelo`), crate `bajatelo`, marca en i18n de Acerca de, index.html y README; internos (bd, temp dir, prefijo `bj-yt-*`). Referencias a la metodología REA en `docs/rea/` y comentarios sin cambios. DMG `Bajatelo_0.1.0_aarch64.dmg` regenerado, verificado con `hdiutil`/`Info.plist` y smoke limpio; suite completa en verde (cargo 67/67+I2, Vitest 11/11, Playwright 7/7, lint/typecheck). Nota: cambio de identificador → nueva carpeta de datos (`com.softnext.bajatelo`) |
| 2026-10-19 6 | **Compresión de audio seleccionable** (feedback del usuario: MP3 a 256k demasiado grande): tasa de bits `64/96/128/192/256/320 kbps` para MP3 y AAC recodificado, por defecto 128k; whitelist con EC-23 fuera de lista; plomado completo comando→tarea→runner→FFmpeg (`-b:a`) en descarga y conversión; selector en Descargas y Convertir (i18n es). I2 real ampliado: MP3 320k > 4× el tamaño de 64k en el mismo fixture. Suite: cargo 68/68, I2 2/2, Vitest 11/11, Playwright 7/7, lint limpio; DMG regenerado |
| 2026-10-19 7 | **Condicionales por formato** (captura del usuario): MP4/Original → solo combo "Calidad" de vídeo; MP3 → solo "Calidad de audio (kbps)". El motor pasa a descargar `-f bestaudio` con MP3 (no baja el vídeo que se tiraría). `selectedQualityId` null para MP3. Suite: cargo **69/69**, I2 2/2, Vitest 11/11, Playwright **8/8**, lint/typecheck limpios; DMG regenerado y verificado |
| 2026-10-19 8 | **D-09/C aprobada y implementada**: motores embebidos en los instaladores (FFmpeg 9.0.2 estático por plataforma; yt-dlp 2026.08.19 verificado con su SHA2-256SUMS oficial) + botón "Actualizar motor" en Configuración (`update_ytdlp` con verificación SHA-256 al `<datos>/engines`) + prioridad embebido→datos→PATH en `tool_locator` + `fetch-engines.sh` reproducible + workflow CI con descarga de motores. Oferta de código GPL de FFmpeg documentada. Evidencia: manual-checklist §2 (prueba del usuario), filas 37–41: DMG final **81,23 MiB** con teclado operativa de motores embebidos verificado; suite: cargo 75/75 (+I2 2/2), Vitest 11/11, Playwright 8/8, lint/typecheck limpios |
| 2026-10-19 9 | **CI v0.1.1 con motores embebidos — 3/3 success** (run 39331552878): corregida la causa del fallo previo (`shell: bash` en el paso de motores; PowerShell no parsea bash), binarios Windows con `.exe` nativo, versión 0.1.1. Draft con 4 assets verificados por tipo/tamaño (exe 108 MB con motores); hashes de yt-dlp verificados contra SHA2-256SUMS oficial en runners. Pendiente honesto: ejecución behavioral de los instaladores en Windows/Linux reales |
| 2026-10-19 10 | **X-02 portapapeles implementada** (primer item post-MVP, patrón REA D1): banner no bloqueante con "Analizar del portapapeles", dismissal por sesión, preferencia clipboardDetection en Configuración, plugin oficial con solo lectura. DMG 0.1.1 final (81,38 MiB) con motores + X-02. Suite: cargo 75/75 · Vitest 11/11 · Playwright 8/8 · lint limpio. Pendientes restantes: prueba del .exe en Windows real, pausa/detener percepción real, playlists (Q5), VoiceOver humano |
| 2026-10-19 11 | **X-05 notificaciones implementada** (post-MVP, REA C12/F6.7): al terminar la última descarga de la cola, notificación local del SO; preferencia notificationsOnFinish en Configuración; plugin oficial de notificaciones. DMG 0.1.1 final **81,48 MiB** con motores + X-02 + X-05. Suite: cargo 75/75 · Vitest 11/11 · Playwright 8/8 · lint/typecheck limpio. Deudas humanas/otras-plataforma restantes: prueba del .exe en Windows real, percepción de pausa/detener, playlists Q5, VoiceOver persona real |
