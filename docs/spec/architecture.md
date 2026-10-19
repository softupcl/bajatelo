# Especificación — Arquitectura

> Fase 2 · Fecha: 2026-10-19 · Estado: BORRADOR — pendiente de aprobación

## 1. Stack verificado (2026-10-19)

Verificado contra registros oficiales (npm registry, crates.io, GitHub releases, endoflife.date). Pendiente de fijar versiones exactas en el `package.json`/`Cargo.toml` al iniciar la Fase 3 (se pinchan entonces con lockfile).

| Componente | Versión actual verificada | Notas |
|---|---|---|
| Tauri (crate Rust) | **2.12.4** | Estable serie 2.x |
| @tauri-apps/api / cli (npm) | **2.12.4 / 2.12.3** | Compatibles con el crate 2.x |
| Vue | **3.5.44** | Composición API + `<script setup>` |
| Vite / vue-tsc | 8.3.4 / 3.3.13 | Build del frontend |
| TypeScript | **5.x estable** (latest npm = 7.0.2 serie nativa nueva) | **D-02**: se adopta 5.x por madurez del ecosistema (vue-tsc/Vite); TS 7 se reevalúa más adelante |
| Vitest | **5.0.5** | Unitarias frontend y lógica TS |
| Playwright | **1.64.1** | E2E del frontend (WebView); con Tauri vía `tauri-driver`/WebDriver cuando aplique (AGENTS.md §3) |
| Rust | estable (toolchain a instalar — D-08) | `cargo test` para adapters |
| yt-dlp | **2026.08.19** (GitHub releases) | Motor de descarga; licencia Unlicense (compatible) |
| FFmpeg / FFprobe | **9.0.2** (rama estable 9.0) | Conversión/inspección; licencia según build (RNF-07) |
| SQLite (crate `rusqlite`, feature `bundled`) | versión compilada embebida | Persistencia local (**D-07 aprobada**); sin instalación del SO |

Requisitos de empaquetado verificados (docs oficiales Tauri 2, consultadas hoy):
- **Windows**: NSIS o MSI (WiX); WebView2 vía bootstrapper en línea o runtime fijo (+~180 MB); opción `minimumWebview2Version`.
- **macOS**: bundle `.app` + `.dmg` (`tauri build --bundles dmg`).
- **Linux**: DEB/RPM/AppImage; requiere `webkit2gtk-4.1` (libwebkit2gtk-4.1-dev en Debian/Ubuntu) en el entorno de build.
- **Sidecar** (`bundle.externalBin`): disponible y documentado (naming por target-triple, spawn con `tauri_plugin_shell`) — se reserva como alternativa a D-03 en una fase posterior, no en MVP.

## 2. Principios (AGENTS.md §6)

1. La UI nunca habla con yt-dlp/FFmpeg: solo con la capa de aplicación (comandos Tauri).
2. Los motores son procesos externos con ciclo de vida explícito (spawn/kill/timeout/código de salida).
3. Toda ejecución de procesos con **lista de argumentos** validada (RNF-01).
4. Sin abstracciones innecesarias; contratos tipados compartidos entre TS y Rust (definición única de tipos + serialización).
5. Estados honestos: nada se marca completado sin verificación (AGENTS.md §8–§9).

## 3. Capas y módulos

```
┌──────────────────────────────────────────────────────────────┐
│ UI — Vue 3 + TypeScript (src/)                               │
│  views/    Descargas · Convertir · Historial · Diagnóstico   │
│  components/  (sin lógica de negocio)                        │
│  services/    bridge.ts  → llamadas a comandos + eventos     │
│  i18n/        cadenas es centralizadas                       │
│  stores/      estado reactivo de cola/historial/preferencias │
└──────────────▲───────────────────────────────────────────────┘
               │ Comandos Tauri (invoke) + Eventos (emit)
┌──────────────▼───────────────────────────────────────────────┐
│ APP — Rust (src-tauri/src/)                                  │
│  commands/    urls.rs · tasks.rs · convert.rs · history.rs   │
│               prefs.rs · diagnostics.rs                      │
│  app/         task_manager (cola+estados) · validation.rs    │
│               progress.rs · naming.rs (saneado/conflictos)   │
│  services/    ytdlp.rs (adapter) · ffmpeg.rs (adapter)       │
│               ffprobe.rs (adapter)                           │
│  models/      task.rs · media_info.rs · errors.rs            │
│  persistence/ store.rs (SQLite: historial, prefs, cola)      │
│  platform/    dialogs.rs · open_target.rs (shell)            │
└──────────────────────────────────────────────────────────────┘
               │ spawn con argumentos (tauri_plugin_shell/std::process)
       yt-dlp        ffmpeg        ffprobe
```

- `commands/`: única frontera con la UI. Cada comando valida entrada, traduce a dominio y devuelve modelos tipados o errores del catálogo (`errors.rs` → códigos de `error-catalog.md`).
- `task_manager`: dueño de la cola y de la máquina de estados (RF-05). Un único hilo/actor de estado; los adapters emiten eventos de progreso que se reenvían a la UI (`emit`).
- `services/`: cada adapter conoce su herramienta (rutas, `--version`, argumentos, parseo de salida) y es sustituible por un fake en tests.
- `persistence/`: SQLite embebido vía `rusqlite` (feature `bundled`, WAL, migraciones numeradas) en el directorio de datos de la app (`app_data_dir`), con verificación de integridad y recuperación ante corrupción (RF-07). **D-07 aprobada**.

## 4. Flujo de una descarga (candidato de referencia)

1. UI → `invoke("analyze_url", {url})`. `validation.rs` valida/normaliza (RF-01; patrón REA A3).
2. `task_manager` crea tarea `queued → inspecting`; `ytdlp.rs` ejecuta `-J --no-playlist` con timeout; parseo JSON → `MediaInfo` + `QualityOption[]` (RF-02).
3. Estado `ready` + evento `task://ready`; la UI muestra ficha y combos (RF-03).
4. Usuario elige calidad/formato/destino → `invoke("start_download", {taskId, qualityId, formatId, dest})` → validación de destino y conflicto (naming.rs) → estado `downloading`.
5. `ytdlp.rs` lanza el proceso con argumentos fijos (plantilla temporal `cg-yt-<id>-<fmt>`, `--newline`, `--no-playlist`, `--no-mtime` — patrón REA §4); parsea stdout (progreso/velocidad/ETA) y emite `task://progress` (RF-04).
6. Al terminar el proceso con exit 0 → si el formato requiere conversión, `ffmpeg.rs` ejecuta la conversión (estado `converting`) con progreso fiable o indeterminado; verificación final (existe + tamaño>0) → `completed` (RF-06) → historial + limpieza de temporales.
7. Cualquier fallo/cancelación → ruta de limpieza garantizada (kill del árbol de proceso, temporales, listeners) → `failed`/`cancelled` (RF-05).

El flujo de conversión local (RF-06) reutiliza los pasos 6 y el naming/conflictos, sin pasos 1–5.

## 5. Estructura de repositorio propuesta (adaptación de AGENTS.md §6)

```
src/                      # Vue
  views/ components/ services/ stores/ i18n/ types/
src-tauri/
  src/ commands/ app/ services/ models/ persistence/ platform/
  capabilities/           # mínimo privilegio (RNF-02)
  tauri.conf.json
tests/                    # fixtures compartidos (JSON de análisis, medios propios)
docs/ rea/ spec/ qa/
```

Convenciones: nombres en inglés en código; cadenas de usuario en español vía i18n; errores con código + detalle técnico opcional; logs sin secretos (RNF-04).

## 6. Seguridad (resumen operativo — detalle en AGENTS.md §7)

- Capabilities mínimas: dialog, fs restringido a destinos elegidos, shell **solo** para open-file/open-folder con validación de ruta; sin `shell:execute` arbitrario; sin exposición de comandos a la UI.
- URLs: solo http(s); sin credenciales en logs/persistencia (redacción); nombres de archivo remotos tratados como no confiables (lista negra + reservas del SO + `..`).
- Sin cookies/tokens; sin descargas autenticadas (AGENTS.md §7; REA Q7).
- Binarios externos: solo de ubicaciones gestionadas/PATH; diagnóstico con timeout y sin ejecutar contenido de fuentes remotas.

## 7. Cumplimiento y empaquetado (RNF-06/07)

- Licencias a documentar antes de distribución: Tauri (MIT/Apache-2.0), Vue/Vite (MIT), yt-dlp (Unlicense), FFmpeg (según build del usuario — el clon **no redistribuye** binarios en MVP, D-03), Rust crates (MIT/Apache-2.0 dominante).
- Empaquetado objetivo MVP: NSIS (Windows), DMG (macOS), DEB + AppImage (Linux) — verificables con builds locales cuando D-08 se apruebe.
- No incluir bundler de ofertas de terceros (antipatrón del instalador de la referencia — REA `01-research.md` §6).
