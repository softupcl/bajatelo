# QA — Checklist de release (R-32 y fase 7)

> Última actualización: 2026-10-19 (fase 7). Estados: `passed` (ejecutado y verificado) · `not-tested` · `blocked` · `outside-MVP`.
> Regla AGENTS.md: nada se marca `passed` sin ejecución real; ningún artefacto se declara listo si no se generó y probó en su SO.

## 1 · Compilación de producción y pruebas (enunciado fase 7, punto 1)

| Item | Estado | Evidencia |
|---|---|---|
| Compilación frontend de producción (`npm run build`) | ✅ passed | `dist/assets/index-*.js` 100,34 kB (gzip 37,06 kB), 0 errores vue-tsc |
| Compilación Rust release (`cargo build --release`) | ✅ passed | `Finished release target` en 47,24 s, sin warnings |
| Suite de pruebas completa | ✅ passed | `cargo test` 67/67 (+I2 real 1/1 opt-in) · Vitest 11/11 · Playwright 7/7 · lint/typecheck limpios |
| Registros de errores sin información sensible | ✅ passed | `TaskError::new` redacta credenciales de URLs incrustadas (`redact_credentials_in_text`, 4 tests); historial guarda URL redactada (CA-07.3); sin `println` con datos de usuario en el código |

## 2 · Instaladores (enunciado fase 7, punto 2)

| Item | Estado | Evidencia |
|---|---|---|
| DMG macOS aarch64 generado | ✅ passed | `npm run tauri build -- --bundles dmg` → `Bajatelo_0.1.0_aarch64.dmg` (2,94 MiB), registro real de build (renombrado: fue `REA_0.1.0`) |
| DMG monta y contiene Bajatelo.app | ✅ passed | `hdiutil attach`: volumen con `Bajatelo.app` + `Applications`; Info.plist verificado (`CFBundleName=Bajatelo`, `CFBundleIdentifier=com.softnext.bajatelo`); `hdiutil detach` limpio |
| Arquitectura | ✅ passed (aarch64) | `file`: `Mach-O 64-bit executable arm64`; Mac Intel requiere build (instrucciones en README §3) |
| Sin rutas absolutas del equipo en el binario | ✅ passed | `otool -L` sin referencias a rutas de usuario; código usa `env::temp_dir` + `app_data_dir` (grep: 0 coincidencias `/Users/`) |
| NSIS Windows | ✅ passed (generado en CI runner nativo, verificado como NSIS válido) → **ejecución en Windows real: not-tested** | run `39313558162`; asset `Bajatelo_0.1.0_x64-setup.exe` (2,2 MB) descargado + `file` OK |
| DEB/AppImage Linux | ✅ passed (generados en runner nativo, tipos verificados) → **prueba en escritorio Linux real: not-tested** | run `39313558162`; `deb` 3,6 MB + `AppImage` 78 MB |
| Firma/notarización macOS | outside-MVP | requiere certificado del usuario (licensing-notes §4) |

## 3 · Motores: distribución y actualización (enunciado fase 7, punto 3)

| Item | Estado | Evidencia |
|---|---|---|
| Política: yt-dlp/FFmpeg instalados por el usuario, no redistribuidos (D-03) | ✅ passed (diseño) | `README.md` §2; `docs/qa/licensing-notes.md` §1 |
| Documentación de actualización de motores (`brew upgrade`/`pipx`/distro) | ✅ passed | `README.md` §2 |
| La app no gestiona ni reemplaza versiones de motores | ✅ passed (diseño) | adapter solo localiza (`tool_locator`) con diagnóstico |

## 4 · Diagnóstico (enunciado fase 7, punto 6)

| Item | Estado | Evidencia |
|---|---|---|
| Detección real de presencia/versión con timeout | ✅ passed | `get_diagnostics` con `--version` y timeout (tests del locator) |
| **Asistente accionable** cuando falta/queda obsoleto un motor | ✅ passed (nuevo en fase 7) | Vista Configuración: guía por herramienta y SO (`brew`/`winget`/distro) con aviso de degradación (RF-08); i18n es |
| Degradación de funciones sin motores | ✅ passed | EC-13/EC-35 del catálogo en las vistas |

## 5 · Documentación de usuario (enunciado fase 7, punto 7)

| Item | Estado | Evidencia |
|---|---|---|
| Instalación macOS (DMG real) | ✅ passed | README §1 |
| Actualización y desinstalación (incluye borrado de datos) | ✅ passed | README §4–§5 |
| Solución de problemas por código del catálogo | ✅ passed | README §6 |
| Instaladores Windows/Linux: requisitos por SO | ✅ passed (documentado) | README §3 |

## 6 · Licencias (enunciado fase 7, puntos 3 y 9)

| Item | Estado | Evidencia |
|---|---|---|
| Inventario de licencias de dependencias embebidas | ✅ passed | `THIRD-PARTY-NOTICES.md` generado por `scripts/third-party-notices.py` (472 crates con licencia, 0 desconocidas; deps JS de runtime) |
| Obligaciones por licencia (redistribución del bundle) | ✅ passed | MIT/Apache/ISC/BSD/0BSD cumplidas con el fichero de avisos; SQLite (bundled) en dominio público (aviso conservado) |
| yt-dlp (Unlicense) / FFmpeg (LGPL/GPL) NO redistribuidos | ✅ passed | sin obligaciones al no distribuirlos (D-03) |
| Sin bundling de ofertas de terceros (antipatrón de la referencia) | ✅ passed | configuración de bundle limpia; REA `01-research.md` §6 |

## 7 · Publicación (condiciones)

| Item | Estado |
|---|---|
| Publicar o firmar artefactos | **repo público creado y pushed (2026-10-19, autorizado por el usuario): github.com/softupcl/bajatelo**; release v0.1.0 generado en CI como **draft** (no visible); firmarla sigue pendiente de certificado |
| Descarga real con contenido autorizado | not-tested — checklist manual §2 (requiere red del usuario) |
| Ejecución del .exe en un Windows real | not-tested — para cumplimiento pleno del enunciado (AGENTS.md "efectiva en la plataforma reclamada") |
| Publicar el draft release | manual del usuario |
