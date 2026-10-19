# REA — 02. Inventario de funciones de ClipGrab

> Estado: BORRADOR FASE 1 · Fecha: 2026-10-19
> Clasificación (AGENTS.md §4.2): **[V]** Verificada · **[I]** Inferida · **[U]** Unknown · **[N/A]**.
> Última columna: encaje con el MVP del clon según AGENTS.md §5.1/§5.2 (✅ MVP · ➕ post-MVP · ❌ no replicar).

Evidencia: `SRC:<archivo>` = fuente oficial 3.9.14 (GPLv3); `WEB:<ruta>` = página oficial (2026-10-19).

## A. Análisis de URLs y metadatos

| # | Función observada en ClipGrab | Clasificación | Evidencia | Clon |
|---|---|---|---|---|
| A1 | Campo de URL que dispara el análisis automáticamente al contener una URL `http(s)://` | [V] | `SRC:mainwindow.cpp:70-77` | ✅ |
| A2 | Análisis vía `yt-dlp -J --no-playlist` → JSON con título, autor, duración, thumbnail y lista de formatos | [V] | `SRC:video.cpp:61-84, 150-387` | ✅ |
| A3 | Normalización de URLs de YouTube (limpia parámetros del `watch?v=`; convierte `youtu.be/<id>`) | [V] | `SRC:video.cpp:65-74` | ✅ |
| A4 | Detección de URLs "conocidas" para el portapapeles: 12 regex (YouTube/youtu.be/embed/playlist, Vimeo, Dailymotion/dai.ly, Facebook, ARD Mediathek, ZDF) | [V] | `SRC:clipgrab.cpp:755-775` | ➕ |
| A5 | Título del vídeo mostrado en el cuadro informativo al terminar el análisis | [V] | `SRC:mainwindow.cpp:340` | ✅ |
| A6 | Estados visibles durante el análisis: campo bloqueado + texto "Espere mientras…" | [V] | `SRC:mainwindow.cpp:73-76` | ✅ |
| A7 | Error de análisis → mensaje genérico ("No downloadable video could be found…"); sin detalles técnicos al usuario | [V] | `SRC:mainwindow.cpp:329-333` | ✅ (con catálogo de errores propio) |
| A8 | Soporte de playlists en el motor (`--flat-playlist`) usado por la búsqueda; la descarga de playlists completas no está expuesta en la UI | [V] | `SRC:video.cpp:48-59, 80` | ➕ (post-MVP según AGENTS.md §5.2) |

## B. Selección de formato y calidad

| # | Función | Clasificación | Evidencia | Clon |
|---|---|---|---|---|
| B1 | Combo de formatos: Original + MPEG4, WMV (Windows), OGG Theora, MP3 (audio only), OGG Vorbis (audio only), Original (audio only) — los modos ffmpeg solo aparecen si ffmpeg/avconv está disponible | [V] | `SRC:clipgrab.cpp:188-209`, `SRC:converter_ffmpeg.cpp:216-222` | ✅ (subconjunto MVP: MP4/MP3/Original) |
| B2 | Combo de calidades generado de los formatos reales reportados por yt-dlp; etiquetas `1080p (HD)`, `2160p (4K)`, `4320p (8K)`, `NNp60` (fps≥59), `unknown`, sufijo ` WebM` | [V] | `SRC:video.cpp:335-384` | ✅ |
| B3 | Deduplicación de calidades por resolución+fps; emparejamiento audio-vídeo compatible (m4a/aac con mp4; opus/ogg con webm) | [V] | `SRC:video.cpp:276-384` | ✅ |
| B4 | Recordar último formato elegido y última calidad (por resolución) entre sesiones | [V] | `SRC:mainwindow.cpp:89-95, 813-820` | ✅ |
| B5 | Opción "Use WebM if possible" (amplía contenedores aceptados; sin preferencia avc1/av01) | [V] | `SRC:video.cpp:210-215`, `SRC:mainwindow.cpp:178` | ➕ |
| B6 | Filtrado de "traducciones" de audio de YouTube: prioriza pistas marcadas "original" | [V] | `SRC:video.cpp:227-259` | ➕ |

## C. Descarga y cola de tareas

| # | Función | Clasificación | Evidencia | Clon |
|---|---|---|---|---|
| C1 | Botón "Grab this clip!" (o Enter) inicia la descarga tras elegir destino con diálogo nativo | [V] | `SRC:mainwindow.cpp:263-277` | ✅ |
| C2 | Nombre de archivo propuesto saneado (elimina `# % & { } \ < > * ? / $ ! ' " : @ + \`` y `|`) | [V] | `SRC:video.cpp:474-476` | ✅ |
| C3 | Opciones de destino: carpeta por defecto (última usada o Escritorio), "siempre guardar en la última ruta", "nunca preguntar nombre" | [V] | `SRC:mainwindow.cpp:127-129, 263-277` | ✅ |
| C4 | Cola "Current Downloads" con columnas Portal / Title / Quality / Target format / Progreso; la más reciente arriba | [V] | `SRC:download_list_model.cpp` | ✅ |
| C5 | Descargas en paralelo sin límite aparente (un proceso yt-dlp por tarea) | [V] código; [U] límite práctico | `SRC:clipgrab.cpp:678-694` | ✅ con límite configurable |
| C6 | Progreso: porcentaje con 1 decimal + bytes (B/KiB/MiB/GiB/TiB), calculado con tamaños reales en disco y calibrado con estimaciones de yt-dlp; tope 99,9% hasta confirmar fin | [V] | `SRC:video.cpp:418-450`, `SRC:download_list_model.cpp:94-103` | ✅ |
| C7 | Pausa (terminate del proceso; estados intermedios `pausing`→`paused`) y reanudación (relanzar `download()`) | [V] | `SRC:video.cpp:126-137, 589-592` | ✅ |
| C8 | Cancelación (estados `canceling`→`canceled` + limpieza de temporales) y confirmación de salida con descargas activas | [V] | `SRC:video.cpp:119-124, 593-597`, `SRC:mainwindow.cpp:492-510` | ✅ |
| C9 | Reinicio de tareas `canceled`/`error` desde el menú contextual | [V] | `SRC:video.cpp:139-143` | ✅ |
| C10 | Etiquetas de estado: Starting… / % / Converting… / Finished / Paused / Canceled / Waiting / Failed | [V] | `SRC:download_list_model.cpp:94-117` | ✅ |
| C11 | Descarga en carpeta temporal con plantilla `cg-youtube-dl-%(id)s-%(format_id)s.%(ext)s` + limpieza de temporales al terminar/cancelar | [V] | `SRC:video.cpp:99-104, 630-636` | ✅ |
| C12 | Notificaciones de bandeja: por descarga o al completar todas (configurable) | [V] | `SRC:mainwindow.cpp:522-528` | ➕ |
| C13 | Doble clic / menú contextual: abrir archivo descargado, abrir carpeta destino ("Show in Finder" en macOS), copiar enlace, abrir enlace en navegador | [V] | `SRC:mainwindow.cpp:703-766`, `SRC:clipgrab.cpp:778-809` | ✅ (Abrir archivo/carpeta del MVP) |

## D. Portapapeles, búsqueda y entrada alternativa

| # | Función | Clasificación | Evidencia | Clon |
|---|---|---|---|---|
| D1 | Monitoreo del portapapeles; si aparece una URL compatible → notificación (modo "ask") o auto-carga en Downloads (modo "always") | [V] | `SRC:clipgrab.cpp:705-713`, `SRC:mainwindow.cpp:409-422` | ➕ |
| D2 | Pestaña de búsqueda de YouTube integrada (debounce 1,5 s, resultados HTML con miniatura y duración, `ytsearch16:`), clic en resultado → Downloads | [V] | `SRC:mainwindow.cpp:599-671`, `SRC:clipgrab.cpp:838-849` | ➕ |
| D3 | Drag & drop de enlaces (text/uri-list) → campo de descarga | [V] | `SRC:mainwindow.cpp:768-779` | ➕ |
| D4 | Atajos Ctrl+1…4 para cambiar de pestaña | [V] | `SRC:mainwindow.cpp:227-242` | ✅ |

## E. Conversión

| # | Función | Clasificación | Evidencia | Clon |
|---|---|---|---|---|
| E1 | Conversión automática post-descarga (estados `converting`→`finished`) — no existe conversión de archivos locales independientes | [V] | `SRC:video.cpp:570-588, 618-623` | ✅ conversión local es MVP aparte (AGENTS.md §5.1.8) |
| E2 | Estrategia ffmpeg: copiar códecs compatibles (`-acodec copy`/`-vcodec copy`), si no recodificar (audio 256k; vídeo bitrate×1,2) | [V] | `SRC:converter_ffmpeg.cpp:99-150` | ✅ |
| E3 | Modos: MPEG4 (mp4/aac-mp3/h264-mpeg4), WMV (wmv2/wmav2), OGG Theora (ogv), MP3, OGG Vorbis, "Original audio only" (contenedor por códec) | [V] | `SRC:converter_ffmpeg.cpp:263-295` | ✅ MP4/MP3; ➕ resto |
| E4 | Etiquetado de metadatos con ffmpeg (`title`, `author`, `artist`) + diálogo modal para título/artista en modo audio | [V] | `SRC:converter_ffmpeg.cpp:152-154`, `SRC:metadata-dialog.ui` | ✅ |
| E5 | Conflicto de nombre de salida: `nombre-1.ext`, `nombre-2.ext`… (sin sobrescribir, sin preguntar) | [V] | `SRC:converter_ffmpeg.cpp:180-193`, `SRC:converter_copy.cpp:33-46` | ✅ |
| E6 | Modo "Original": copia del archivo sin reconvertir | [V] | `SRC:converter_copy.cpp:31-60` | ✅ |
| E7 | Detección de ffmpeg: bundle (macOS) → avconv → ffmpeg del sistema; test `ffmpeg -formats` y detección de HLS para DASH | [V] | `SRC:converter_ffmpeg.cpp:331-375` | ✅ (diagnóstico de dependencias, AGENTS.md §5.1.12) |
| E8 | Ruta de ffmpeg configurable (ajuste oculto `ffmpegPath`) | [V] | `SRC:converter_ffmpeg.cpp:38` | ➕ |
| E9 | Editor de presets de conversión (códecs, bitrate, fps) — presente en el .ui pero **eliminado en runtime** (`removeTab(2) //fixme`) | [V] | `SRC:mainwindow.ui:900-1116`, `SRC:mainwindow.cpp:203` | ❌ (post-MVP lejano) |
| E10 | FFprobe no se usa; la inspección del medio se hace con la salida de error de un `ffmpeg` sin objetivo | [V] | `SRC:converter_ffmpeg.cpp:50-78` | ✅ el clon usará FFprobe (mejora) |

## F. Preferencias, idioma y variedad

| # | Función | Clasificación | Evidencia | Clon |
|---|---|---|---|---|
| F1 | Ajustes persistentes (QSettings) aplicados al instante, sin botón guardar | [V] | `SRC:mainwindow.cpp` (slots de ajustes) | ✅ |
| F2 | Selector de idioma con ~35 idiomas, reinicio requerido, detección automática, soporte RTL | [V] | `SRC:clipgrab.cpp:35-183`, `SRC:main.cpp:69-86` | ✅ es primero |
| F3 | Traducción oficial al español con numerosas cadenas sin terminar | [V] | `SRC:clipgrab_es.ts` | ✅ superable |
| F4 | Bandeja del sistema: minimizar a bandeja, cerrar a bandeja, clic restaura | [V] | `SRC:mainwindow.cpp:435-444, 571-582` | ➕ |
| F5 | Proxy HTTP/SOCKS5 con autenticación (aplicado a Qt y a yt-dlp) | [V] | `SRC:clipgrab.cpp:735-753`, `SRC:youtube_dl.cpp:87-106` | ➕ |
| F6 | "Force IPv4" (pasa `--force-ipv4` a yt-dlp) | [V] | `SRC:youtube_dl.cpp:108-111` | ➕ |
| F7 | Sin historial persistente de descargas | [V] por ausencia | `SRC:mainwindow.ui` | ✅ el clon lo tendrá (MVP) |
| F8 | Opciones ocultas en 3.9.14: "Remember logins", "Ignore SSL errors" | [V] | `SRC:mainwindow.cpp:180-184` | ❌ |

## G. Mantenimiento interno de la app de referencia

| # | Función | Clasificación | Evidencia | Clon |
|---|---|---|---|---|
| G1 | Descarga automática de yt-dlp (GitHub releases) a la carpeta de datos de la app si falta o es < 2021.09.25; diálogo con progreso; rechazar = salir | [V] | `SRC:clipgrab.cpp:560-637` | ✅ diagnóstico + guía al usuario (sin cierre forzado) |
| G2 | Diagnóstico visible: versión y ruta de yt-dlp y Python en la pestaña About | [V] | `SRC:mainwindow.cpp:781-792` | ✅ (AGENTS.md §5.1.12) |
| G3 | Auto-actualización de la app: XML de clipgrab.org, notas en visor web, verificación SHA-1, abre instalador | [V] | `SRC:clipgrab.cpp:266-558` | ❌ post-MVP (AGENTS.md §5.2) |
| G4 | Canal remoto de comandos (message/open/set/unset/die) dentro del XML de actualización | [V] | `SRC:clipgrab.cpp:406-477` | ❌ no replicar (riesgo) |
| G5 | Migración de ajustes legacy al primer arranque | [V] | `SRC:main.cpp:53-65` | [N/A] |
| G6 | Arranque con `--start-minimized`; opciones estándar `--version`/`--help` | [V] | `SRC:main.cpp:45-51, 90-92` | ➕ |

## Resumen cuantitativo

- Funciones inventariadas: **48** (A:8, B:6, C:13, D:4, E:10, F:8, G:6, ajustado en tablas).
- Marcan camino directo al MVP del clon: A1-A3/A5-A7, B1-B4, C1-C11, C13, D4, E1-E8, F1-F3, F7, G1-G2.
- Post-MVP (AGENTS.md §5.2): A4/A8, B5-B6, C12, D1-D3, E9, F4-F6, G3.
- A no replicar: E9 en su forma actual, F8, G4, bundling de ofertas del instalador.
