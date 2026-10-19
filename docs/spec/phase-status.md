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
| 1 | Descarga real contra YouTube (F4/F5 dinámicos con red) | requiere red + contenido autorizado del usuario (no automatizable en la suite — AGENTS.md §11) | `docs/qa/manual-checklist.md` §2 (prueba manual) |
| 2 | Builds NSIS (Windows) y DEB/AppImage (Linux) | solo existen hosts con esos SO | build por SO (R-32) |
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
