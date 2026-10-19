# Bajatelo — Descargas y conversión multimedia

Aplicación de escritorio independiente (Tauri 2 + Vue 3 + TypeScript + Rust) para analizar URLs de vídeo, descargar contenido autorizado mediante **yt-dlp** y convertir archivos locales con **FFmpeg**.

> Proyecto guiado por `AGENTS.md` (metodología REA). La especificación aprobada vive en `docs/spec/`; la investigación de la aplicación de referencia, en `docs/rea/`.

---

## Guía para el usuario final (instalador)

### 1 · Instalación en macOS

1. Obtén el DMG firmado por el proyecto: `src-tauri/target/release/bundle/dmg/Bajatelo_0.1.0_aarch64.dmg` (build real verificado: 2,93 MiB, monta y contiene `Bajatelo.app`; binario `arm64` — Apple Silicon. En Mac Intel hay que compilar en ese objetivo; ver abajo).
2. Monta el DMG y arrastra `Bajatelo.app` a `Aplicaciones`.
3. Primer arranque: macOS GateKeeper mostrará el aviso de app sin notarizar (la firma/notarización queda fuera del MVP, requiere certificado del usuario). Clic derecho → **Abrir** para permitirlo, o desactívalo en Preferencias de Seguridad.

### 2 · Motores incluidos: yt-dlp y FFmpeg van dentro (D-09 — opción híbrida)

**Bajatelo no requiere instalar motores a mano**: los instaladores incluyen

- **FFmpeg 9.0.2 / FFprobe** (estático, por plataforma — GPL con oferta de código fuente en el release)
- **yt-dlp 2026.08.19** (binario oficial, verificado con su `SHA2-256SUMS`) **con actualización asistida**: Configuración → "Actualizar motor" baja el release oficial más reciente de yt-dlp, comprueba su SHA-256 y lo instala en los datos de la app — los extractores quedan al día sin esperar una versión nueva de la app.

Si el equipo ya tiene yt-dlp/FFmpeg en el PATH (Homebrew/gestor del SO), se usa como **respaldo**. Configuración → Diagnóstico muestra qué binario se usa realmente (embebido o del sistema) y su versión. Sin conexión la app arranca normalmente con el motor embebido (la actualización necesita red; su fallo se reporta con EC-20).

### 3 · Instaladores para Windows y Linux (requisitos documentados; builds por verificación por SO)

Los hosts macOS no los generan: hay que construir en cada plataforma objetivo.

| Plataforma            | Comando                                                        | Requisitos                                                                                                                                           |
| --------------------- | -------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| Windows               | `npm run tauri build -- --bundles nsis`                        | Rust + Node ≥ 22; WebView2 en el equipo destino (Tauri ofrece bootstrapper); `icon.ico` ya incluido                                                  |
| Linux                 | `npm run tauri build -- --bundles deb appimage`                | Rust + Node ≥ 22; dependencias de compilación de webkit2gtk-4.1 (`libwebkit2gtk-4.1-dev`, `build-essential`…); `AppImageTool` lo descarga el bundler |
| macOS Intel/Universal | `rustup target add x86_64-apple-darwin` + build con ese target | genera binario para Mac Intel o universal2                                                                                                           |

### 4 · Actualización de la app

Sustituye `Bajatelo.app` por la nueva versión (el DMG nuevo). Los datos locales (historial, preferencias) viven fuera de la app y se conservan:

- macOS: `~/Library/Application Support/com.softnext.bajatelo/` (`bajatelo.db` + backups). Nota: tras el renombrado del identificador (`com.softnext.rea` → `com.softnext.bajatelo`), el primer arranque crea esta carpeta nueva; los datos de la carpeta antigua pueden eliminarse manualmente.

### 5 · Desinstalación

Elimina `Bajatelo.app` de Aplicaciones. Para borrar también los datos locales elimina la carpeta de arriba. La app no instala extensiones, demonios ni servicios.

### 6 · Solución de problemas

| Síntoma                                                    | Qué hacer                                                                                                                                               |
| ---------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| "El motor de descarga (yt-dlp) no está disponible" (EC-13) | Configuración → "Actualizar motor" (lo baja del release oficial y verifica su SHA-256); si el equipo quedó sin red, instala yt-dlp con el gestor del SO |
| "FFmpeg/FFprobe no está" (EC-35)                           | no debería ocurrir con el FFmpeg embebido del instalador; si pasa, Diagnóstico muestra qué binario falta y la guía por SO del gestor                    |
| La descarga falla con conexión (EC-20)                     | revisa red; el contenido debe ser accesible sin bloqueo (sin bypass de DRM/uces: ver seguridad abajo)                                                   |
| El sitio no lo permite o requiere acceso (EC-21)           | solo contenido autorizado: la app no evade paywalls ni DRM                                                                                              |
| "No se pudo abrir la carpeta/archivo" (EC-43)              | el archivo pudo moverse/borrarse; ábrelo manualmente                                                                                                    |
| La base de datos se restauró al arrancar (EC-41)           | se cuidó desde el backup automático; no perdiste operaciones previas completadas                                                                        |

**Privacidad:** sin telemetría ni cookies; los errores locales se guardan con la URL redactada (sin credenciales) y un detalle técnico truncado.

---

## Requisitos del sistema (desarrollo)

| Herramienta                           | Uso                   | Instalación (macOS)                                               | Instalación (Linux)                                      |
| ------------------------------------- | --------------------- | ----------------------------------------------------------------- | -------------------------------------------------------- |
| Node.js ≥ 22 + npm                    | frontend/build        | `nvm` o instalador oficial                                        | gestor de paquetes de la distro                          |
| Rust (toolchain estable)              | backend Tauri         | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` | igual                                                    |
| Xcode Command Line Tools (solo macOS) | compilar Rust         | `xcode-select --install`                                          | —                                                        |
| `webkit2gtk-4.1` (solo Linux)         | WebView de Tauri      | —                                                                 | `sudo apt install libwebkit2gtk-4.1-dev` (Debian/Ubuntu) |
| yt-dlp                                | motor de descarga     | `brew install yt-dlp`                                             | `pipx install yt-dlp` o gestor de la distro              |
| FFmpeg + FFprobe                      | conversión/inspección | `brew install ffmpeg`                                             | gestor de la distro                                      |

## Puesta en marcha (desarrollo)

```bash
npm install          # dependencias del frontend
npm run tauri dev    # aplicación en modo desarrollo (compila Rust la primera vez)
```

## Comandos

| Comando               | Qué hace                                            |
| --------------------- | --------------------------------------------------- |
| `npm run dev`         | solo frontend (Vite)                                |
| `npm run tauri dev`   | app completa en desarrollo                          |
| `npm run build`       | typecheck (vue-tsc) + build del frontend            |
| `npm run tauri build` | empaquetado (en macOS genera .app/.dmg; ver arriba) |
| `npm test`            | pruebas unitarias frontend (Vitest)                 |
| `npm run test:rs`     | pruebas Rust (`cargo test`)                         |
| `npm run lint`        | ESLint + Prettier check                             |
| `npm run format`      | formateo con Prettier                               |

## Empaquetado multiplataforma

Los binarios no se compilan en cruz por defecto; se empaqueta en cada SO objetivo:

```bash
# macOS
npm run tauri build -- --bundles dmg
# Windows (NSIS; requiere WebView2 en el objetivo o bootstrapper incluido)
npm run tauri build -- --bundles nsis
# Linux
npm run tauri build -- --bundles deb appimage
```

Revisión de licencias y verificación de builds: `docs/spec/mvp-backlog.md` (item R-32), `docs/qa/licensing-notes.md` y `AGENTS.md` §14. Avisos de terceros generados automáticamente en `THIRD-PARTY-NOTICES.md` (`scripts/third-party-notices.py`).

## Estructura

```text
src/          Vue 3 + TS — vistas (pages/), componentes, i18n (es), servicios (bridge), tipos
src-tauri/    Rust — commands/ (frontera UI), services/ (adapters), models/ (contratos)
docs/         rea/ (investigación) · spec/ (especificación) · qa/ (differential-report, regression-results, release-checklist, licensing-notes, manual-checklist)
tests/        fixtures compartidos (por poblar)
scripts/      utilidades reproducibles (p. ej. avisos de terceros)
THIRD-PARTY-NOTICES.md   licencias de dependencias embebidas
```

Contrato UI↔Rust: `src/types/contracts.ts` ↔ `src-tauri/src/models/` (sincronizados con tests).

## Estado del proyecto

- **MVP completo y empaquetado** (fase 5): descargas reales con cola/vivencias de progreso/pausa/cancelación (D-10–D-13), conversión local real (C-20/C-21), historial SQLite, diagnóstico con guía instalable, accesibilidad E2E y DMG macOS verificado.
- Fases 6 (validación diferencial) y 7 (preparación de release) ejecutadas con evidencia: `docs/qa/release-checklist.md`.
