# QA — Notas de licencias y redistribución (R-32 / RNF-07)

> Fecha: 2026-10-19 · Criterio del AGENTS.md §14: verificación de licencias **antes** de distribuir; documentado si cada dependencia la instala el usuario, se descarga o va embebida.

## 1. Decisiones de distribución (aprobadas)

- **DECISIÓN D-09/C ADOPTADA (2026-10-19, autorizada por el usuario; revoca parcialmente D-03):** los motores se **embeben** en los instaladores según la opción "híbrido" del diseño `docs/spec/engines-bundling.md`:
  - **FFmpeg/FFprobe 9.0.2** embebido por plataforma (win/linux: BtbN `autobuild-2026-10-19-13-06` GPL; darwin-aarch64: osxexperts 9.0 con SHA256 y fuente publicada).
  - **yt-dlp 2026.08.19** embebido por plataforma (binarios oficiales del repo yt-dlp, verificados contra su `SHA2-256SUMS`) **con actualización asistida** desde la app (botón en Configuración → `update_ytdlp`): baja el release oficial más reciente, verifica el SHA2-256SUMS oficial y lo instala en los datos de la app.
  - El PATH queda como respaldo (degradación RF-08 vigente) y el diagnostic real sigue mostrando la versión usada.
- **Oferta de código GPL (FFmpeg)**: los instaladores incluyen binarios GPL; el oferta del código fuente corresponde por versión:
  - FFmpeg 9.0.2: https://github.com/FFmpeg/FFmpeg/tree/n9.0.2 (tarball oficial)
  - Builds BtbN: https://github.com/BtbN/FFmpeg-Builds (scripts de configuración del build)
  - Enunciado incluido en los datos del draft-release y en `THIRD-PARTY-NOTICES.md` (re-generar con `scripts/third-party-notices.py`, que ya lista los crates).
- **Licencia del binario macOS (osxexperts)**: el proveedor marca sus builds "educational"; hay precedentes de redistribución comunitaria con hashes publicados (p. ej. `ffmpeg-static`, BSD-2). Se distribuyen tal cual, con la fuente FFmpeg enlazada y su SHA-256 verificado. Si se prefiere un proveedor alternativo, se sustituye en `scripts/fetch-engines.sh` (único punto de configuración).

## 2. Inventario de dependencias verificadas

| Componente | Licencia | Cómo llega | Nota de redistribución |
|---|---|---|---|
| Tauri (crates + JS api/cli) | MIT / Apache-2.0 | compilado con cargo / npm | redistribute OK con nota de copyrights |
| Vue 3 / Vite | MIT | npm (devDep/bundled en el dist del frontend) | OK |
| TypeScript / vue-tsc | Apache-2.0 | devDep solo compilación | OK |
| Vitest | MIT | devDep | OK |
| Playwright | MIT/Apache-2.0 | devDep (chromium solo en CI, no redistribuido por la app) | OK |
| ESLint / Prettier | MIT | devDep | OK |
| rusqlite + SQLite (bundled) | MIT (`rusqlite`) + SQLite Nu (bin embebido) | compilado | OK (nota de copyright SQLite) |
| tokio / chrono / url / serde | MIT / Apache-2.0 | compilado | OK |
| tauri-plugin-dialog / opener | MIT / Apache-2.0 | compilado | OK |
| **yt-dlp** | **Unlicense** (dominio público del código) | **instalado por el usuario** — no redistribuido | sin obligación al no redistribuir |
| **FFmpeg/FFprobe** | **LGPL/GPL según su build** (complejidad de códecs líquidas y GPL/extensiones oficiales) | **instalado por el usuario** — no redistribuido | sin obligación al no redistribuir; portar al build GPL del usuario no afecta la app |
| Chromium (WebView en dev,࿳ no empaquetado por la app) | BSD | Solo Playwright dev/CI | no redistribuido |

## 3. Referencias al antipatrón de la referencia (REA)

- El instalador de ClipGrab para Windows incluye **ofertas de terceros** (TheCassiniLabs/dotInstaller — REA `01-research.md` §6): el clon **no** replica bundling de ofertas (RNF-06).
- El checkpoint "verification of the SHA-1" de la referencia para updates: el MVP del clon no tiene auto-actualización (post-MVP X-09); si se añade, usar SHA-256 mínimo + canales firmados.

## 4. Pendiente antes de release público

1. ~~Generar THIRD-PARTY-NOTICES automático~~ → **resuelto en fase 7 (2026-10-19)**: `scripts/third-party-notices.py` genera `THIRD-PARTY-NOTICES.md` desde `Cargo.lock` + `package.json` (472 crates, 0 licencias desconocidas; sin peticiones de red).
2. Firma de binarios/notarización macOS — fuera del MVP (requiere certificado del usuario).
