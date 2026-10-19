# Especificación — Backlog del MVP

> Fase 2 · Fecha: 2026-10-19 · Estado: BORRADOR — pendiente de aprobación
> Formato (AGENTS.md §13): ID · título · valor · prioridad (P0/P1/P2) · dependencias · criterios de aceptación (CA) · pruebas · estado/bloqueadores.
> Implementación en orden de dependencia (slices verticales: UI+Rust+tests juntos).

## Bloque 0 — Preparación del entorno (decisión D-08 — requiere aprobación)

| ID | Item | Prioridad | Notas |
|---|---|---|---|
| B-00 | Aprobar especificación y decisiones D-01…D-08 | P0 | ✅ **COMPLETADO** (2026-10-19) |
| B-01 | Instalar toolchain: rustup/cargo estable, yt-dlp, ffmpeg+ffprobe (gestor de SO o paquetes oficiales); inicializar git; crear proyecto Tauri 2 + Vue 3 + TS con npm (D-01) y lockfile. SQLite llega embebido vía `rusqlite` (`bundled`) — sin instalación del SO | P0 | ✅ APROBADO (D-08) — primera acción de la Fase 3 |
| B-02 | CI local scriptado: `npm run test:ts / test:rs / test:e2e / build` | P0 | |

## Bloque 1 — Esqueleto y contratos

| ID | Item | Valor | Depende | CA | Pruebas |
|---|---|---|---|---|---|
| S-01 | Shell de app: 4 vistas (Descargas · Convertir · Historial · Diagnóstico) + navegación y layout responsive de ventana; i18n es centralizado | base navegable | B-02 | RF-10 CA-10.1–10.3 parcial | E1 humo + lint i18n |
| S-02 | Tipos compartidos TS↔Rust (TaskState, MediaInfo, QualityOption, errores del catálogo) + `bridge.ts` inyectable (fake incluido) | contrato único | S-01 | serialización cruzada OK | U1+U2 de tipos |
| S-03 | Máquina de estados con transiciones legales, incluida pausa (T1–T6 de `data-model.md`) | núcleo fiable | S-02 | todas las transiciones verificadas | U2 exhaustivo |

## Bloque 2 — Validación y diagnóstico (independencia del motor)

| ID | Item | Valor | Depende | CA | Pruebas |
|---|---|---|---|---|---|
| V-01 | Validación/normalización de URLs (RF-01) | primera defensa de seguridad | S-03 | CA-01.1–01.4 | U1/U2 tabla + E1 |
| V-02 | Diagnóstico de dependencias (RF-08): detección, `--version` con timeout, reporte, guía por SO; degradación de funciones | app útil sin motores | S-03 | CA-08.1–08.3 | I1 binarios falsos + E1 |
| V-03 | Persistencia SQLite (migraciones, integridad, backup) + Preferencias (RF-09) | configuración persistente | S-03 | CA-09.1–09.3 | U1/U2 + E1 |

## Bloque 3 — Descarga (núcleo)

| ID | Item | Valor | Depende | CA | Pruebas |
|---|---|---|---|---|---|
| D-10 | Análisis real por yt-dlp (RF-02): adapter `-J`, parseo, calidades honestas; estados `queued/inspecting/ready/failed` | análisis funcional | V-01, V-02 | CA-02.1–02.3 | I1 fixtures + I2 opt-in |
| D-11 | UI de la vista Descargas: campo URL, ficha de metadatos, combos formato/calidad/destino (RF-03) con degradación por diagnóstico | flujo completo visible | D-10, V-03 | CA-03.1–03.3, CA-10.x | E1 con fake |
| D-12 | Ejecución de descarga real (RF-04/RF-05): cola, proceso con argumentos fijos, progreso/velocidad/ETA, **pausa/reanudación (D-05)**, cancelación con kill+limpieza, reintento sin duplicados, cierre con confirmación | funcionalidad esencial | D-11 | CA-04.1–04.3, CA-05.1–05.5 | I1 kill/pausa/timeout/duplicados + E1 |
| D-13 | Historial automático de descargas (RF-07) | trazabilidad local | D-12 | CA-07.1–07.3 | U1/U2 + E1 |

## Bloque 4 — Conversión local

| ID | Item | Valor | Depende | CA | Pruebas |
|---|---|---|---|---|---|
| C-20 | Vista Convertir: selección nativa, inspección con FFprobe, ejecución FFmpeg con verificación de resultado real (RF-06) | valor propio del clon | V-02, D-12 | CA-06.1–06.4 | I1 + I2 (fixtures de medios) + E1 |
| C-21 | Historial y estados de conversión integrados (EC-30…EC-35) | coherencia | C-20 | estados completos | U2 + E1 |

## Bloque 5 — Cierre de MVP

| ID | Item | Valor | Depende | CA | Pruebas |
|---|---|---|---|---|---|
| R-30 | Accesibilidad y estados exhaustivos (RF-10) en las 4 vistas | calidad de producto | D-13, C-21 | CA-10.1–10.3 | E1 + auditoría manual |
| R-31 | Validación diferencial REA (AGENTS.md §12): contrastar con `docs/rea/03-user-flows.md` y registrar `docs/qa/differential-report.md` | evidencia de paridad | R-30 | registros con estado por flujo | checklist QA |
| R-32 | Empaquetado por SO (NSIS/DMG/DEB/AppImage) + revisión de licencias (RNF-06/07) | release-ready | R-30 | builds reales verificados | builds + checklist manual |

## Post-MVP (P2 — solo con aprobación; base REA/AGENTS.md)

| ID | Item | Justificación documental |
|---|---|---|
| X-02 | Portapapeles (detección de URLs compatibles) | REA D1 |
| X-03 | Drag & drop de URLs | REA D3 |
| X-04 | Formatos extra: WMV, OGG Theora/Vorbis, WebM opcional | REA B1/B5, E3 |
| X-05 | Notificaciones del sistema y bandeja | REA C12, F4 |
| X-06 | Proxy y forzar IPv4 | REA F5/F6 |
| X-07 | Metadatos ID3 editables al descargar audio | REA E4 (el MVP escribirá metadatos básicos automáticos sin diálogo) |
| X-08 | Sidecar de motores (bundling) con revisión de licencias FFmpeg | architecture §1; D-03 alternativa |
| X-09 | Auto-actualización de la app | AGENTS.md §5.2 (nunca el canal de comandos remotos de la referencia — REA G4) |

## Estado y bloqueadores actuales

- Especificación y decisiones D-01…D-08: **APROBADAS** (2026-10-19). La pausa (D-05) está integrada en el MVP dentro de D-12; SQLite (D-07) reemplaza a JSON como persistencia.
- Bloques 1–5: `not-started`. Siguiente acción: ejecutar **B-01** (instalación de entorno aprobada) y después B-02/S-01.
- Riesgos documentados: disponibilidad del entorno Rust (mitigable reinstalando), comportamientos de yt-dlp reales (mitigado por niveles I1/I2 del plan de pruebas), empaquetado multiplataforma (verificación real por SO en R-32).
