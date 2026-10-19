# QA — Checklist manual de release (R-32)

> Última actualización: 2026-10-19 · items que no son verificables automáticamente; los resultados se registran aquí con fecha y estado.

## 1. Empaquetado

| Item | Estado | Notas |
|---|---|---|
| DMG macOS construido (`npm run tauri build -- --bundles dmg`) | ✅ passed (2026-10-19) | `REA_0.1.0_aarch64.dmg` — **2,93 MiB** en `src-tauri/target/release/bundle/dmg/`; release build real en 1 m 01 s |
| El DMG monta y contiene la app | ✅ passed (2026-10-19, automático) | `hdiutil attach` verificado: checksums CRC32 OK; volumen con `REA.app` + `Applications`; eject limpio |
| NSIS Windows (`tauri build -- --bundles nsis`) | not-tested (requiere Windows) | build por SO (AGENTS.md: "eficactivo en la plataforma reclamada") |
| DEB/AppImage Linux (`tauri build -- --bundles deb appimage`) | not-tested (requiere Linux) | |
| Firma/notarización macOS | fuera del MVP | requiere certificado del usuario (licensing-notes §4) |

## 2. Prueba manual con contenido real (solo contenido autorizado del usuario)

> **Ejecución real del usuario (2026-10-19, reportada en la sesión):** "funciona perfecto descargar videos y audio desde YouTube". Estos items quedan **passed** con esa prueba; lo no cubierto se declara not-tested.

| Item | Estado | Notas |
|---|---|---|
| Análisis de una URL real de YouTube | ✅ passed (prueba manual del usuario, 2026-10-19) | motor 2026.8.19 del SO |
| Descarga completa vídeo → `completed` con archivo no vacío | ✅ passed (prueba manual del usuario) | YouTube; formats reales |
| Descarga de audio (MP3 con bestaudio) | ✅ passed (prueba manual del usuario) | — |
| Pausa → Reanudar continúa los `.part` (Q3 del REA) | not-tested — ejecutar manualmente | el mecanismo está en verde por código; la percepción del usuario aún no |
| Cancelación no deja procesos ni temporales residuales | not-tested — `pgrep yt-dlp` + revisar `$/temp/bajatelo` | |
| Conversión local de un archivo del usuario (MP4→MP3) | ✅ I2 real en verde; UI esperando confirmación manual | |
| **Botón Detener durante una conversión real** | not-tested | mecanismo verificado con proceso POSIX |
| **Abrir archivo / Abrir carpeta** tras una conversión | not-tested | requiere app nativa en ejecución |

## 3. Accesibilidad final con VoiceOver/lector de pantalla real

| Item | Estado |
|---|---|
| VoiceOver anuncia los estados de tarea (aria-live) | not-tested (manual) |
| Contraste con el tema oscuro del sistema | ✅ verificado por contraste AA del CSS básico; un aspecto visual manual pendiente |

## 4. Documentación de instalación para el usuario final

| Item | Estado |
|---|---|
| README.md: requisitos y puesta en marcha | ✅ |
| Guía si un motor falta (Diagnóstico → instrucciones por SO) | ✅ (mensaje EC-13/EC-35 + vista Diagnóstico) |
