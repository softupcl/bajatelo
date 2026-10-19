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
| 5 | `npm run lint` (ESLint + Prettier check) | ✅ `All matched files use Prettier code style!` · 0 errores — **regresión detectada y corregida el 2026-10-19 (2ª pasada)**: 2 errores `no-unused-vars` en `e2e-fake.ts` (exención `^_` añadida a `eslint.config.js`) + desvío de Prettier en `e2e/accessibility.spec.ts` y `src/services/e2e-fake.ts`; `test-results` añadido a `.prettierignore` | 2026-10-19 |
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
| Cancelación, reintentos y limpieza de procesos | ✅ backend: único lanzador (T3), señales T4/T6, cleanup bounded por prefijo de tarea; **2ª pasada: cancelación real durante FFmpeg verificada (kill de proceso POSIX + salida parcial borrada)** |
| Herramientas de medios ausentes y diagnóstico de versiones | ✅ `tool_locator` 5 tests + degradación RF-08 |
| Éxito y fallo de conversión | ✅ unit (compatibilidad códecs) + **I2 real** (MP4/MP3 con FFmpeg) |
| Conflictos de salida, paths inválidos, filenames saneados | ✅ 8 tests de naming (incl. traversal imposible por construcción) |
| Persistencia del historial y preferencias | ✅ 6 tests del store (round-trip, backup+recuperación, recreate) |
| Estados de UI vacíos/cargando/error/completado | ✅ **passed (R-30)**: Playwright E1 — 6/6 pruebas (`npx playwright test` en 1.5 s): navegación+aria-current, etiqueta del campo, catalogaención honesta del análisis, empty `role=status`, diagnóstico y foco por teclado | |

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

## Registro R-30 (2026-10-19)

| # | Comando exacto | Resultado | Fecha |
|---|---|---|---|
| 10 | `npm install -D @playwright/test` + `npx playwright install chromium` (1.64.1 / chromium-1248) | ✅ instalado | 2026-10-19 |
| 11 | `npx playwright test` (config playwright.config.ts; Vite dev 1420; doble del puente `rea-e2e=1`) | ✅ `6 passed (1.5s)` — navegación/aria-current, etiqueta+disabled honesto, ficha con calidades del fixture, empty `role=status`, diagnóstico, foco por teclado | 2026-10-19 |

## Registro de la revisión de fase 4/5 con enunciados del usuario (2026-10-19, 2ª pasada)

Revisión con los enunciados literales de fases 4 y 5. Defectos hallados y corregidos:

1. **Cancelación durante la conversión no honrada** (req. 5/9 del enunciado de fase 5): la señal T4 solo se sondeaba *antes* de FFmpeg; una cancelación durante la ejecución dejaba la tarea en `completed`. Corregido en `services/ffmpeg.rs` (`run_ffmpeg` con verificador `CancelCheck` sondeado cada 100 ms en `tokio::select!`; el proceso muere por `kill_on_drop`) y `app/task_manager.rs` (resultado → estado `cancelled` + salida parcial borrada + historial, en el runner de conversión local y en la fase `converting` de descargas). Error del catálogo EC-33. `cancel_task` ya admitía `Converting`; ahora el resultado es coherente.
2. **Requisito 10 (abrir archivo/carpeta convertidos) ausente**: el plugin `opener` estaba registrado pero sin flujo. Añadido comando Rust `open_item(path, reveal)` (rutas existentes, sin shell propio, fallo → EC-43), método `openItem` en el puente (`bridge.ts`, doble E2E incluido) y botones "Abrir archivo"/"Abrir carpeta" en la vista Convertir solo cuando el estado es `completed` con destino confirmado por `task://state`.
3. **Regresión de lint** (ya documentada en la fila 5): corregida.
4. **Aviso honesto**: la codificación del progreso de conversión sigue siendo indeterminada (RF-04/AGENTS.md §9: progreso solo cuando es derivable con fiabilidad). La conversión no exporta `time/frame` de FFmpeg; seguiría siendo tarea X futura si se pide explícitamente.

| # | Comando | Resultado | Estado |
|---|---|---|---|
| 12 | `cargo test --manifest-path src-tauri/Cargo.toml` (tras correcciones; +3 tests de cancelación real con proceso POSIX) | ✅ `63 passed; 0 failed; 1 ignored` | passed |
| 13 | `cargo test --manifest-path src-tauri/Cargo.toml -- --ignored` (I2 ampliado: EC-30 en no-multimedia; vídeo sin audio → MP4 ok y MP3 → EC-32; MP4/MP3 reales) | ✅ `1 passed; 0 failed` | passed |
| 14 | `npm run typecheck` / `npm run lint` / `npm test` / `npx playwright test` | ✅ limpio · `11 passed` · `6 passed (1.6s)` | passed |

## Registro de la fase 6 — validación diferencial con app en ejecución (2026-10-19, 3ª pasada)

Hallazgos corregidos (detalle en `differential-report.md` §7):

1. **D1**: el puente tipaba `analyzeUrl` como `MediaInfo` cuando el backend devuelve `AnalysisOutcome{taskId, media, error}` → en runtime real la ficha quedaba vacía y la tarea creada en la cola era invisible. Corregido con el contrato `AnalysisOutcome` en `contracts.ts`, puente y doble E2E.
2. **D2**: la vista Descargas no iniciaba descargas ni mostraba la cola (backend D-12 desconectado de la UI). Vista reescrita: botón `Descargar` → diálogo nativo → `start_download`, cola viva con `get_tasks` + eventos `task://state`/`task://progress`, acciones por fila (Pausar/Reanudar/Detener/Reintentar/Abrir carpeta, REA F5.1/F5.6), Historial con Abrir archivo/carpeta.

| # | Comando | Resultado | Estado |
|---|---|---|---|
| 15 | `cargo test --manifest-path src-tauri/Cargo.toml` (sin cambios backend en fase 6) | ✅ `63 passed; 0 failed; 1 ignored` | passed |
| 16 | `npm run typecheck` / `npm run lint` / `npm test` | ✅ limpio · `11 passed` | passed |
| 17 | `npx playwright test` (+ nuevo test de flujo de cola con el doble) | ✅ **`7 passed (1.4s)`** | passed |
| 18 | `npm run tauri dev` (smoke real con la UI de descarga integrada) | ✅ `Running target/debug/rea` (PID 27307); arranque correcto y parada limpia verificada con `ps` | passed |

## Registro de la fase 7 — preparación de release (2026-10-19)

| # | Comando | Resultado | Estado |
|---|---|---|---|
| 19 | `cargo test --manifest-path src-tauri/Cargo.toml` (+4 tests de redacción de credenciales en `TaskError`/URLs incrustadas) | ✅ **`67 passed; 0 failed; 1 ignored`** | passed |
| 20 | `npm run build` + `cargo build --release` | ✅ build de producción limpio (47,24 s release Rust; dist 100,34 kB) | passed |
| 21 | `npm run tauri build -- --bundles dmg` | ✅ `REA_0.1.0_aarch64.dmg` (2,93 MiB) **regenerado tras añadir el asistente de diagnóstico**, verificación con `hdiutil attach`: monta con `REA.app` + `Applications`, binario `arm64` (`file`), `otool -L` sin rutas de usuario, eject limpio; **smoke visual del binario del DMG**: arranca (PID 29403) y se detiene limpio | passed |

## Registro del renombrado REA → Bajatelo (2026-10-19, tras fase 7)

Solicitud del usuario: la app se llamará **Bajatelo**. Cambios verificables:

- `tauri.conf.json`: `productName → "Bajatelo"`, `identifier → com.softnext.bajatelo`, título de ventana.
- Crate/binario: `rea` → `bajatelo` (`Cargo.toml`, `bajatelo_lib`, `Cargo.lock` regenerado), `package.json`.
- Marca visible i18n (título de app, Acerca de), `index.html`, README.
- Internos renombrados: bd `bajatelo.db`, dir temporal `temp/bajatelo`, prefijo de temporales `bj-yt-*,` fixtures de test.
- La metodología REA (citas de `docs/rea/` en comentarios y docs) **no** se renombra: es otro nombre.

| # | Comando | Resultado | Estado |
|---|---|---|---|
| 24 | `cargo test` / Vitest / Playwright / lint / typecheck tras el renombrado | ✅ 67/67 (+I2 1/1) · 11/11 · 7/7 · limpios | passed |
| 25 | `npm run tauri build -- --bundles dmg` (renombrado) | ✅ `Bajatelo_0.1.0_aarch64.dmg` (2,94 MiB); `hdiutil` monta, `Info.plist`: `CFBundleName=Bajatelo`, `CFBundleIdentifier=com.softnext.bajatelo`; binario `bajatelo` arm64; `hdiutil detach` limpio | passed |
| 26 | Smoke del binario del DMG renombrado | ✅ arranca y se detiene limpio (PID 30853) | passed |
| 27 | `python3 scripts/third-party-notices.py` (cabecera y crate propio actualizados) | ✅ 472 crates, 0 desconocidas | passed |

## Registro de la fase 8 (solicitud de compresión) — tasa de bits de audio seleccionable (2026-10-19)

Solicitud tras la prueba manual del usuario: los MP3 a 256k "quedaban demasiado grandes". Añadida la **tasa de bits de audio** (kbps) seleccionable: `64 / 96 / 128 / 192 / 256 / 320`, aplicada al MP3 y al AAC recodificado de MP4; por defecto **128 kbps**; `Original` no se re-convierte y la sigue ignorando. Validación estricta por whitelist (fuera de lista → EC-23), cableado completo: solicitud → tarea (`selectedAudioBitrate`) → runner → FFmpeg (`-b:a`), en descarga y en conversión local. UI con selector en Descargas (oculto con Original) y en Convertir; i18n es.

| # | Comando | Resultado | Estado |
|---|---|---|---|
| 28 | `cargo test` (+test de `audio_bitrate_args`; whitelist) | ✅ **68/68** | passed |
| 29 | `cargo test -- --ignored` (I2; **nueva**: MP3 64k vs 320k con el mismo fixture → 320k > 4× tamaño de 64k) | ✅ **2/2** | passed |
| 30 | Vitest / Playwright / typecheck / lint | ✅ 11/11 · 7/7 · limpio | passed |
| 31 | `npm run tauri build -- --bundles dmg` (regenerado con el selector) | ✅ `Bajatelo_0.1.0_aarch64.dmg` (2,94 MiB) | passed |

## Registro — condicionales por formato en la selección (2026-10-19, fase 8b)

Feedback con captura: con MP3 aparecía la calidad de vídeo (irrelevante) y con MP4 el selector de audio. Comportamiento corregido y optimizado:

- **MP4 / Original**: solo el combo "Calidad" (vídeo). Sin selector de audio.
- **MP3**: solo "Calidad de audio (kbps)". Sin combo de calidad de vídeo.
- **Optimización real del motor (RF-03)**: con formato MP3 la descarga pasa a `-f bestaudio` (solo la pista de audio del origen) en lugar de bajar vídeo que luego se descartaba; la conversión a MP3 usa la tasa elegida. Menor consumo de red y tiempo.
- La calidad de vídeo deja de configurarse en la tarea al descargar MP3 (`selectedQualityId = null`, visible en cola).

| # | Comando | Resultado | Estado |
|---|---|---|---|
| 32 | `cargo test` (+ `motor_format_spec`: MP3→`bestaudio`; MP4→`vid+aud`; `build_download_arguments` con spec) | ✅ **69/69** | passed |
| 33 | `cargo test -- --ignored` (I2 real 2/2: probe+conversiones; bitrate 64k vs 320k) | ✅ I2 2/2 | passed |
| 34 | Vitest / typecheck / lint | ✅ 11/11 · limpios | passed |
| 35 | Playwright (+2 aserciones nuevas: selects visibles por formato, opciones 64–320 kbps, sin "1080p" con MP3) | ✅ **8/8** | passed |
| 36 | `npm run tauri build -- --bundles dmg` | ✅ `Bajatelo_0.1.0_aarch64.dmg` (2,94 MiB) — instalador con el comportamiento corregido | passed |

## Registro — CI oficial: release v0.1.0 en GitHub Actions (2026-10-19)

Tag `v0.1.0` push → run `39313558162`: **3/3 jobs success** (windows-latest, macos-latest aarch64, ubuntu-22.04). Draft-release con 4 assets, descargados y verificados con `file`:

| Asset | Tamaño | Verificación de tipo |
|---|---|---|
| `Bajatelo_0.1.0_x64-setup.exe` | 2,2 MB | PE32 + NSIS self-extracting (válido) |
| `Bajatelo_0.1.0_aarch64.dmg` | 2,9 MB | imagen disco macOS arm64 |
| `Bajatelo_0.1.0_amd64.deb` | 3,6 MB | Debian package 2.0 |
| `Bajatelo_0.1.0_amd64.AppImage` | 78 MB | ELF x86-64 static-pie |

Condición honesta (AGENTS.md): el `.exe` **no se ejecutó en Windows** aquí; generar/type-converted para NSIS en runner nativo. Publicación final del draft: manual, del usuario. Avisos cosmeticos del runner (Node 20 deprecation en actions, cola ARM64): no afectan el resultado.

## Registro — D-09/C: motores embebidos + actualización asistida (2026-10-19)

El usuario aprueba la opción C del diseño `docs/spec/engines-bundling.md` (FFmpeg embebido + yt-dlp embebido con updater). Implementación real ejecutada:

| # | Verificación | Resultado |
|---|---|---|
| 37 | `scripts/fetch-engines.sh darwin-aarch64` (idempotente, python3 portable) | ✅ yt-dlp 2026.08.19 verificado contra **SHA2-256SUMS oficial** (hash `0f192b7e…`, binario universal x86_64+arm64); ffmpeg/ffprobe osxexperts 9.0 con los SHA256 del proveedor (`591260…`/`e11c17…`); ejecutados directamente |
| 38 | `cargo test` (locator: prioridad embebido→datos-al-so→PATH; 4 tests del updater: asset_map/tag_name/sums/sha256-vector) | ✅ **75/75**; I2 real 2/2 |
| 39 | `npm run tauri build -- --bundles dmg` con `resources: ["engines/"]` | ✅ DMG **81,23 MiB**; montado: `Contents/Resources/engines/darwin-aarch64/{yt-dlp,ffmpeg,ffprobe}` ejecutables y operativos (`-version` ok dentro del volumen) |
| 40 | Smoke del binario del DMG con motores embebidos | ✅ arranca (PID 37335) y se detiene limpio |
| 41 | Workflow release.yml: paso `download pinned engines` por plataforma | escrito (win: BtbN `autobuild-2026-10-19-13-06` 9.0.2; linux idem; licencias en MANIFEST.txt) — se verificará en el próximo tag |

Licencias (GPL de FFmpeg): oferta de código fuente enlazada en el draft del release + `licensing-notes.md` §1; yt-dlp Unlicense sin condiciones. Los binarios NO se comitean (`.gitignore`).

## Registro — CI v0.1.1 con motores embebidos (2026-10-19, run 39331552878)

Tras la corrección del run fallido 39322157746 (PowerShell no parseaba la sintaxis bash en el paso de motores): **3/3 jobs success**, y el paso `download pinned engines` verificó los hashes en runners:

- yt-dlp 2026.08.19 verificado en las tres plataformas contra el `SHA2-256SUMS` oficial (`6667…` win · `0f19…` mac · `58162…` linux); ffmpeg/ffprobe extraídos de BtbN 9.0.2 (win/linux) y osxexperts (darwin), con su SHA real en los logs.
- Assets del draft v0.1.1 (descargados, `file` OK): `x64-setup.exe` **108 MB** (NSIS con motores), DMG 81 MB, DEB 175 MB, AppImage 236 MB.
- Versión 0.1.1 sincronizada en `tauri.conf.json`/`Cargo.toml`/`package.json`; releaseBody con la oferta de código GPL.
- Condición honesta: ejecutar el `.exe` en un Windows real / DEB-AppImage en escritorio Linux sigue `not-tested` (aquí se verifican generación, integridad y tipo).

## Registro — X-02 portapapeles (post-MVP aprobado por REA D1) — 2026-10-19

Primera función del backlog post-MVP: detección de URL compatible **copiada al portapapeles**, patrón observado en la referencia (REA D1, `mainwindow.cpp:409-422`):

- Banner **no bloqueante** con disparo explícito (D-06): "Analizar del portapapeles" / "Ignorar" (se silencia el resto de la sesión)
- Pre-chequeo de esquema http(s) en la UI; la validación autoritativa sigue en el backend (`analyze_url`); se omite si coincide con el campo o hay análisis en curso
- Preferencia `clipboardDetection` (activa por defecto) con checkbox en Configuración (RF-09)
- plugin oficial `tauri-plugin-clipboard-manager` con capability de solo-lectura (`allow-read-text`): mínimo privilegio, sin escritura de portapapeles
- Verificación: cargo 75/75 · Vitest 11/11 · Playwright 8/8 · lint/typecheck limpio · DMG 81,38 MiB con motores + X-02 dentro

## Registro — X-05 notificaciones al terminar (post-MVP, REA C12/F6.7) — 2026-10-19

- Al completar/fallar/cancelar la **última descarga (kind=Download)** de la cola: notificación local del SO ("Bajatelo — Todas las descargas terminaron"), replicando el patrón de la referencia (bandeja "All downloads finished", `mainwindow.cpp:522-528`)
- Punto único: `TaskManager::set_state` (cada transición emitida pasa por él); solo si quedan 0 descargas activas/pendientes
- Preferencia `notificationsOnFinish` (activa por defecto) con checkbox en Configuración
- plugin oficial `tauri-plugin-notification` con capability `notification:default`
- Verificación: cargo 75/75 · Vitest 11/11 · Playwright 8/8 · lint limpio · **DMG 0.1.1 final 81,48 MiB** con motores + X-02 + X-05
| 22 | `python3 scripts/third-party-notices.py` | ✅ `THIRD-PARTY-NOTICES.md` con 472 crates licenciados (0 desconocidas) + JS de runtime; reproducible y sin red | passed |
| 23 | Suite tras los cambios de fase 7 (asistente diagnóstico, i18n, README, ignores) | ✅ cargo 67/67 · Vitest 11/11 · Playwright 7/7 · lint/typecheck limpios | passed |

Notas honestas de fase 7:
- Los binarios del frontend van **comprimidos dentro del binario Tauri**: verificar el contenido del DMG por `grep` no es factible; se verificó con build de producción real, `hdiutil`, `file`, `otool -L` y smoke del binario.
- NSIS/DEB/AppImage: requisitos documentados (README §3), sin build (no hay hosts con esos SO).
- El binario DMG no está firmado ni notarizado (fuera del MVP; requiere certificado del usuario). Nada publicado.

El doble del puente (`src/services/e2e-fake.ts`) cumple los contratos de `src/types/contracts.ts`; el modo de pruebas solo se activa con la clave `rea-e2e=1`, fuera de uso normal de la app.
