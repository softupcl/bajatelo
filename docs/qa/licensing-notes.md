# QA — Notas de licencias y redistribución (R-32 / RNF-07)

> Fecha: 2026-10-19 · Criterio del AGENTS.md §14: verificación de licencias **antes** de distribuir; documentado si cada dependencia la instala el usuario, se descarga o va embebida.

## 1. Decisiones de distribución (aprobadas)

- **El aplicativo NO redistribuye binarios de terceros de medios** (D-03): yt-dlp y FFmpeg son instalados por el usuario (Homebrew/gestor de la distro) y la app solo los **localiza** con diagnóstico (RF-08). Esto elimina el problema de redistribución GPL de algunos builds de FFmpeg empaquetados con códecs externos.
- SQLite llega **compilado dentro** del binario (`rusqlite` feature `bundled`), en la misma modalidad de licencia pública de SQLite (SQLite Nu Public License — compatible con MIT/Apache apps).

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
