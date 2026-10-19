# REA — 05. Hallazgos técnicos de ClipGrab

> Estado: BORRADOR FASE 1 · Fecha: 2026-10-19
> Base: código fuente oficial **ClipGrab 3.9.14** (GPLv3), extraído de `https://download.clipgrab.org/clipgrab-3.9.14.tar.gz` (enlazado desde la home oficial). Referencias `SRC:<archivo>:<línea>` relativas al árbol extraído (fuera del repositorio). Clasificación: **[V]** Verificada · **[I]** Inferida · **[U]** Unknown.
> Propósito: entender *cómo se comporta* la referencia para reproducir comportamientos observables con una arquitectura propia e independiente. **No** se copia código.

## 1. Stack y construcción

| Aspecto | Hallazgo | Clas. | Evidencia |
|---|---|---|---|
| Framework | Qt 5 Widgets; qmake; módulos `core gui widgets network xml webenginewidgets` | [V] | `SRC:clipgrab.pro`, `SRC:README` |
| Tamaño del núcleo | ~6.300 líneas C++/headers en total (mainwindow.ui: 1.306; clipgrab.cpp: 862; mainwindow.cpp: 837; video.cpp: 636; converter_ffmpeg.cpp: 376) | [V] | `wc -l` sobre el árbol |
| macOS | Bundle con Python.framework incluido (para yt-dlp), ffmpeg dentro del bundle, AppKit/Foundation, plist, icns | [V] | `SRC:youtube_dl.cpp:55-71`, `SRC:converter_ffmpeg.cpp:338-340`, `SRC:clipgrab.pro` |
| Windows | Python incluido (`python/python.exe` junto al ejecutable); instalador con ofertas de terceros (dotInstaller/TheCassiniLabs) | [V] | `SRC:youtube_dl.cpp:51-52`, `WEB:/legal` |
| Linux | Depende de `python3` del sistema | [V] | `SRC:youtube_dl.cpp:82-85` |

## 2. Arquitectura lógica observada

```
MainWindow (mainwindow.*)  ──UI puro: widgets, combos, lista, diálogos
      │ usa
ClipGrab (clipgrab.*)      ──coordinador: formatos, cola (downloads), ajustes (QSettings),
      │                        portapapeles, proxy, updates, descarga de yt-dlp, helpers
      ├─ video (video.*)   ──tarea: estados, fetchInfo (yt-dlp -J), download (yt-dlp),
      │                        parseo de progreso/errores, sanitización, conversión
      ├─ YoutubeDl (youtube_dl.*)  ──localización/instancia del motor yt-dlp (proceso Python)
      ├─ converter_copy / converter_ffmpeg (converter*) ──fábrica de formatos; "copy" y ffmpeg
      └─ DownloadListModel ──adapta la cola a QTreeView (5 columnas)
```

- La UI nunca llama a yt-dlp/ffmpeg directamente: pasa por `ClipGrab`/`video`/`converter`. Patrón alineado con AGENTS.md §6. [V]
- Formatos = pares (convertidor, modo). `converter_copy` aporta 1 modo ("Original"); `converter_ffmpeg` aporta 6; solo se registran si `isAvailable()`. [V] `SRC:clipgrab.cpp:188-209`
- Persistencia: `QSettings` (organización "ClipGrab", dominio clipgrab.org). [V] `SRC:main.cpp:40-42`

## 3. Modelo de estados de tarea (exacto)

`SRC:video.h:63`:

```
empty → fetching → fetched → downloading → converting → finished
                   ↓ (JSON inválido/vacío o ERROR:) → error
downloading ⇄ pausing → paused → (resume) → downloading
downloading/paused → canceling → canceled (+limpieza temporales)
canceled/error → (restart) → downloading
```

Detalles verificables:
- `enqueueDownload` exige estado `fetched` y evita duplicados; no hay "colar sin iniciar". [V] `SRC:clipgrab.cpp:678-694`
- `getRunningDownloadsCount` cuenta todo lo que no es `finished`/`error`/`canceled` — usado para la confirmación de salida y notificación final. [V] `SRC:clipgrab.cpp:715-721`
- `finished` solo se alcanza tras `handleConversionFinished` del conversor (se comprueba que la copia/ffmpeg haya acabado); **no se verifica** que el archivo final exista y no esté vacío — deficiencia observada. [V] `SRC:video.cpp:618-623`, `SRC:converter_copy.cpp:48-57`
- Etiquetas visibles por estado: ver tabla en `03-user-flows.md`. [V] `SRC:download_list_model.cpp:94-117`

## 4. Integración con yt-dlp (detalles operativos)

- Localización: primero `AppDataLocation/yt-dlp` descargado por la app; si no, `yt-dlp` del sistema; se valida ejecutando `--version`. [V] `SRC:youtube_dl.cpp:11-39`
- Ejecución: **`<python3> <yt-dlp> <args>`** (script, no binario). En Windows/macOS Python viene con la app; en Linux el del sistema. [V] `SRC:youtube_dl.cpp:45-118`
- Análisis: `-J --no-playlist <url>`; búsqueda: `-J --yes-playlist --flat-playlist ytsearch16:"<kw>"`. [V] `SRC:video.cpp:54-83`, `SRC:clipgrab.cpp:838-849`
- Descarga: `--newline --no-playlist --no-mtime -o <TempLocation>/cg-youtube-dl-%(id)s-%(format_id)s.%(ext)s -f <videoFormat>[+<audioFormat>] <url>`. [V] `SRC:video.cpp:93-116`
- Proxy: `--proxy <url>` (http/socks5 ± auth) y `--force-ipv4` según ajustes. [V] `SRC:youtube_dl.cpp:87-113`
- Actualización propia del motor: `yt-dlp --update` al arrancar (si no está deshabilitado por ajuste). [V] `SRC:clipgrab.cpp:639-647`
- Progreso: parseo de stdout línea a línea (`--newline`): `Destination:`, `has already been downloaded`, `Merging formats into`, `[download] N.N% of ~?N.N(K|M|G|T)iB`, `ERROR: …`. Tamaño total = suma de estimaciones por archivo; progreso real = tamaño en disco de `*.part` y finales. Filtrado anti-saltar: ignora cambios de estimación >10% para evitar porcentajes erráticos. [V] `SRC:video.cpp:389-450`
- Cancelación/pausa: `QProcess::terminate()` (SIGTERM) y espera al evento `finished`; luego limpieza. [V] `SRC:video.cpp:119-131`

## 5. Selección de calidad (algoritmo exacto)

A partir del JSON de yt-dlp (`SRC:video.cpp:207-384`):

1. Acepta formatos con `ext` en {mp4, m4a} (+{webm, opus} si "Use WebM"); separa solo-audio (`vcodec == "none"`).
2. Ordena vídeo por altura ↓, fps ↓, ext, preferencia de códec `avc1`/`av01` (no aplicada si WebM activo), acodec "none" primero; audio por `tbr` ↓.
3. Deduplica vídeo por (altura, fps).
4. Para cada formato de vídeo genera una calidad: nombre `NNNp` (+`60` si fps≥59; sufijos `(HD)`≥700, `(4K)`≥2000, `(8K)`≥4000; `unknown` si 0), tamaños declarados por yt-dlp, y un audio compatible emparejado por contenedor (m4a/aac/mp4 con mp4; webm/ogg/opus con webm) cuya posición se reparte proporcionalmente entre calidades.
5. Sin `formats` aceptables → estado `error`.

Etiqueta del portal en la lista: `extractor` (o `ie_key`) en minúsculas; "generic" → hostname; "youtube" → "YouTube". [V] `SRC:video.cpp:176-199, 528-531`

## 6. Conversión (comportamiento exacto)

- Diagnóstico: en macOS usa el ffmpeg del bundle; en el resto, prueba `avconv` y luego `ffmpeg`; corre `<bin> -formats` y guarda si soporta `hls` (proxy de "DASH"). Sin ffmpeg/avconv → modal de error y los 6 modos ffmpeg no se registran (queda "Original"). [V] `SRC:converter_ffmpeg.cpp:331-375`
- FFmpeg corre en un hilo (`ffmpegThread`); la invocación se compone **por concatenación de strings con comillas** (riesgo observado, no replicar). [V] `SRC:converter_ffmpeg.cpp:26-211`
- Inspección del medio: ejecuta `ffmpeg -y -i <input>` (sin salida) y parsea stderr con regex `Audio: (.*)`/`Video: (.*)`/bitrate. [V] `SRC:converter_ffmpeg.cpp:50-78`
- Reglas: códec aceptado → `-acodec copy`/`-vcodec copy`; si no, recodifica (audio `-ab 256k` o vorbis `-aq 9`; vídeo bitrate×1,2); "none" → `-an`/`-vn`; añade `-metadata title/author/artist`. [V] `SRC:converter_ffmpeg.cpp:99-154`
- Contenedores por modo: 0→mp4, 1→wmv, 2→ogv, 3→mp3, 4→ogg, 5→(derivado: ogg/mp3/m4a/opus). [V] `SRC:converter_ffmpeg.cpp:225-294`
- Conflicto de salida: si existe, sufijos `-1`, `-2`… nunca sobrescribe ni pregunta. [V]
- Modo "Original": `QFile::copy` a `target.<ext-original>` con la misma política de sufijos; error → `error writing data to <target>`. [V] `SRC:converter_copy.cpp:31-60`

## 7. Seguridad observada (patrones a evitar / a mejorar en el clon)

| Observación en la referencia | Clas. | Implicación para el clon (AGENTS.md §7) |
|---|---|---|
| Comandos ffmpeg construidos por concatenación de strings con comillas | [V] | Prohibido: usar lista de argumentos del proceso |
| Contraseña de proxy en QSettings en texto plano | [V] | No almacenar credenciales |
| XML de actualización con comandos remotos (`set`/`unset`/`open`/`die`) aplicados sin firma | [V] | No replicar; mínimos privilegio |
| Instalador Windows con ofertas de terceros | [V] | No replicar |
| Sanitización de nombre de archivo por lista negra de caracteres | [V] | En el clon: lista negra + reserva de nombres + chequeo de path traversal |
| Verificación de actualización por SHA-1 (débil hoy) pero sí verifica el binario descargado | [V] | El clon, si algún día descarga herramientas, debe verificar (SHA-256 mínimo) |
| Certificados Mozilla CA incluidos en el recurso para SSL | [V] | [N/A] en Tauri (usa el almacén del sistema/rustls) |

## 8. Detalles de producto observables (para la paridad de UX)

- progreso tope 99,9% hasta confirmación real de finalización (evita "100% falso"). [V] `SRC:download_list_model.cpp:101-102`
- unidades binarias humanizadas con espacio fino U+2009; duraciones hh:mm:ss/mm:ss (vacío si 0). [V] `SRC:clipgrab.cpp:811-836`
- el análisis reutiliza un solo "vídeo actual"; nueva URL sustituye al anterior salvo que ya esté encolado. [V] `SRC:clipgrab.cpp:242-264`
- "Waiting" aparece para entradas tipo playlist/url plana (p. ej. resultados de búsqueda no descargados aún). [V] `SRC:video.cpp:176-188`
- Restore de contexto: última pestaña, último formato, última calidad (por resolución), última carpeta. [V]
- En macOS: botón "Open target folder" de la lista se oculta (se usa menú contextual/Finder) y portapapeles forzado a "always". [V] `SRC:mainwindow.cpp:209-215`

## 9. Versiones publicadas vs. fuente analizada

- Web oficial (2026-10-19): Windows 3.9.15, macOS 4.0.0-beta2, Linux 3.9.14 AppImage. La web enlaza el **fuente 3.9.14** como código oficial. [V]
- Por tanto, hallazgos = versión estable 3.9.14; posibles deltas menores en 3.9.15 y cambios mayores en la beta 4.0 (posible Qt6/nueva UI): [Unknown] → `06-open-questions.md`.
