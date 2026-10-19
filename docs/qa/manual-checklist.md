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

| Item | Estado | Notas |
|---|---|---|
| Análisis de una URL real de YouTube (o la lista de soporte oficial) | not-tested — ejecutar manualmente | motor 2026.8.19 instalado en el SO |
| Descarga completa → `completed` con archivo no vacío | not-tested | requiere la red del usuario; ver bloqueos del differential-report §4.1 |
| Pausa → Reanudar continúa los `.part` (Q3 del REA) | not-tested | valida la inferencia de la referencia (C7) |
| Cancelación no deja procesos ni temporales residuales | not-tested | `pgrep yt-dlp` + revisar `/tmp/rea` |
| Conversión local de un archivo del usuario (MP4→MP3) | ✅ equivalente automatizable (I2 real en verde) | el manual confirmará la UI |
| **Botón Detener durante una conversión real** → estado `cancelled`, sin salida parcial y sin proceso FFmpeg residual | not-tested — verificar manualmente tras la corrección de la 2ª pasada | mecanismo verificado en pruebas (proceso POSIX + fixture corto); se recomienda confirmar la percepción en UI |
| **Abrir archivo / Abrir carpeta** tras una conversión `completed` | not-tested — verificar manualmente | comando `open_item` (opener); requiere app nativa en ejecución |

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
