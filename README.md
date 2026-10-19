# REA — Descargas y conversión multimedia

Aplicación de escritorio independiente (Tauri 2 + Vue 3 + TypeScript + Rust) para analizar URLs de vídeo, descargar contenido autorizado mediante **yt-dlp** y convertir archivos locales con **FFmpeg**.

> Proyecto guiado por `AGENTS.md` (metodología REA). La especificación aprobada vive en `docs/spec/`; la investigación de la aplicación de referencia, en `docs/rea/`.

## Requisitos del sistema

| Herramienta                           | Uso                                                   | Instalación (macOS)                                               | Instalación (Linux)                                      |
| ------------------------------------- | ----------------------------------------------------- | ----------------------------------------------------------------- | -------------------------------------------------------- |
| Node.js ≥ 22 + npm                    | frontend/build                                        | `nvm` o instalador oficial                                        | gestor de paquetes de la distro                          |
| Rust (toolchain estable)              | backend Tauri                                         | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` | igual                                                    |
| Xcode Command Line Tools (solo macOS) | compilar Rust                                         | `xcode-select --install`                                          | —                                                        |
| `webkit2gtk-4.1` (solo Linux)         | WebView de Tauri                                      | —                                                                 | `sudo apt install libwebkit2gtk-4.1-dev` (Debian/Ubuntu) |
| yt-dlp                                | motor de descarga (opcional en dev; diagnóstico real) | `brew install yt-dlp`                                             | `pipx install yt-dlp` o gestor de la distro              |
| FFmpeg + FFprobe                      | conversión/inspección                                 | `brew install ffmpeg`                                             | gestor de la distro                                      |

La aplicación **no redistribuye** binarios de terceros (decisión D-03): el diagnóstico integrado (vista Configuración) detecta si están disponibles y guía su instalación.

## Puesta en marcha

```bash
npm install          # dependencias del frontend
npm run tauri dev    # aplicación en modo desarrollo (compila Rust la primera vez)
```

## Comandos

| Comando               | Qué hace                                           |
| --------------------- | -------------------------------------------------- |
| `npm run dev`         | solo frontend (Vite)                               |
| `npm run tauri dev`   | app completa en desarrollo                         |
| `npm run build`       | typecheck (vue-tsc) + build del frontend           |
| `npm run tauri build` | empaquetado (en macOS genera .app/.dmg; ver abajo) |
| `npm test`            | pruebas unitarias frontend (Vitest)                |
| `npm run test:rs`     | pruebas Rust (`cargo test`)                        |
| `npm run lint`        | ESLint + Prettier check                            |
| `npm run format`      | formateo con Prettier                              |

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

Revisión de licencias y verificación de builds: `docs/spec/mvp-backlog.md` (item R-32) y `AGENTS.md` §14.

## Estructura

```text
src/          Vue 3 + TS — vistas (pages/), componentes, i18n (es), servicios (bridge), tipos
src-tauri/    Rust — commands/ (frontera UI), services/ (adapters), models/ (contratos)
docs/         rea/ (investigación) · spec/ (especificación) · qa/ (por crear)
tests/        fixtures compartidos (por poblar)
```

Contrato UI↔Rust: `src/types/contracts.ts` ↔ `src-tauri/src/models/` (sincronizados con tests).

## Estado del proyecto

- Fase 3 (scaffold) completada: shell navegable con 5 vistas, estados vacíos honestos, diagnóstico real de dependencias.
- Bloque 2 del backlog completado: validación/normalización de URLs (V-01), persistencia SQLite con integridad/backup (V-03), análisis real con yt-dlp (D-10) y su UI (D-11).
- Pendiente (backlog en `docs/spec/mvp-backlog.md`): cola de descargas con progreso/cancelación (D-12), conversión local (C-20).
