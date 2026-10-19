# REA — 03. Flujos de usuario observables de ClipGrab

> Estado: BORRADOR FASE 1 · Fecha de investigación: 2026-10-19
> Convenio de evidencia: `WEB:<url>` = página oficial consultada el 2026-10-19; `SRC:<archivo>` = código fuente oficial 3.9.14 (GPLv3) extraído de `https://download.clipgrab.org/clipgrab-3.9.14.tar.gz` (enlazado desde la página oficial). Ruta local de extracción: fuera del repositorio (`$TMP/opencode/rea-reference/clipgrab-3.9.14/`).
> Convenio de clasificación: **[V]** Verificado · **[I]** Inferido · **[U]** Unknown · **[N/A]** No aplicable.

Contexto de observación: no hay aplicación en ejecución local. Los flujos se reconstruyen a partir de (a) la documentación oficial con capturas incluidas (`WEB:/faqs/howto-download-youtube-video`) y (b) el código fuente de la interfaz y la lógica. Los pasos marcados [V] están respaldados por ambos o por código inequívoco; los pasos [I] son interpretaciones del código pendientes de observar en una app en ejecución.

---

## F1. Introducir una URL

| # | Paso | Estado | Evidencia |
|---|------|--------|-----------|
| 1 | El usuario abre la pestaña **Downloads** (pestaña por defecto de la app, o Ctrl+2). | [V] pestaña índice 1; al arrancar se restaura la última pestaña usada (`MainTab`) | `SRC:mainwindow.ui` (tab_2 "Downloads"), `SRC:mainwindow.cpp:247` |
| 2 | La pestaña muestra un cuadro de instrucción: *"Please enter the link to the video you want to download in the field below."* (es: "Introduzca el enlace al vídeo que quiere en el campo siguiente.") | [V] | `SRC:mainwindow.ui` (downloadInfoBox), `SRC:clipgrab_es.ts` |
| 3 | El usuario pega o escribe la URL en el campo de texto (`downloadLineEdit`), que toma el foco al iniciar. | [V] | `SRC:mainwindow.cpp:87` (`setFocus`) |
| 4 | El campo solo reacciona si el texto empieza por `http://` o `https://`. | [V] | `SRC:mainwindow.cpp:70-71` |
| 5 | Vías alternativas de entrada de URL (todas terminan rellenando `downloadLineEdit`): pegado con URL compatible en el portapapeles (según ajuste), clic en un resultado de búsqueda, arrastrar-y-soltar de un enlace (drag & drop de `text/uri-list`). | [V] | `SRC:mainwindow.cpp:409-422, 599-605, 768-779` |
| 6 | Fallos observables en este paso: texto no-URL → no ocurre nada visible (sin mensaje de error); URL con esquema no-http(s) → ignorada. | [V] (comportamiento), [U] (si existe algún feedback adicional en el binario) | `SRC:mainwindow.cpp:70-71` |

## F2. Analizar el contenido (fetch de metadatos)

| # | Paso | Estado | Evidencia |
|---|------|--------|-----------|
| 1 | En cuanto el campo contiene una URL http(s) válida, el campo se bloquea (solo lectura) y el cuadro de instrucción cambia a: *"Please wait while ClipGrab is loading information about the video ..."* (es: "Espere mientras ClipGrab carga información sobre el vídeo..."). Los combos y el botón de descarga se deshabilitan. | [V] | `SRC:mainwindow.cpp:73-76`, `SRC:clipgrab_es.ts` |
| 2 | Internamente se ejecuta `yt-dlp -J --no-playlist <url>` (volcado JSON de metadatos). Las URLs de YouTube se normalizan antes: se eliminan parámetros sobrantes del `watch?v=` y `youtu.be/<id>` se convierte a `watch?v=<id>`. | [V] | `SRC:video.cpp:61-84` |
| 3 | Si el análisis tiene éxito: el infobox muestra el **título del vídeo en negrita**, se habilitan los combos de formato y calidad, se rellena el combo de calidad con las calidades encontradas y el foco vuelve al campo (editable de nuevo). | [V] | `SRC:mainwindow.cpp:335-362` |
| 4 | Las calidades se agrupan/deduplican por resolución+fps y se etiquetan como `NNNp`, `NNNp60` (fps≥59), con sufijos `(HD)` ≥700, `(4K)` ≥2000, `(8K)` ≥4000, ` WebM` para contenedor webm, o `unknown` si la resolución es 0. | [V] | `SRC:video.cpp:335-384` |
| 5 | Si la opción "Remember video quality" está activa (lo está por defecto), se preselecciona la mejor calidad cuya resolución no supere la última elegida por el usuario. | [V] | `SRC:mainwindow.cpp:349-361` |
| 6 | Si falla el análisis (JSON inválido, vacío, o yt-dlp devuelve error): el infobox muestra *"No downloadable video could be found. Maybe you have entered the wrong link or there is a problem with your connection."* (es: "No se encontró un vídeo descargable. Quizá ha introducido el enlace incorrecto o hay un problema con su conexión."), el campo se desbloquea y la UI de descarga queda deshabilitada. | [V] mensaje y estados; [U] textos exactos de error internos de yt-dlp (no se muestran al usuario en este flujo) | `SRC:video.cpp:150-161, 452-459`, `SRC:mainwindow.cpp:329-333` |

## F3. Seleccionar calidad y formato

| # | Paso | Estado | Evidencia |
|---|------|--------|-----------|
| 1 | Dos combos aparecen bajo el campo URL: **Format:** y **Quality:**, ambos deshabilitados hasta que el análisis termina. | [V] | `SRC:mainwindow.ui` (downloadComboFormat/downloadComboQuality, `enabled=false`), `SRC:mainwindow.cpp:457-464` |
| 2 | El combo Formato lista: `Original` + (si hay ffmpeg/avconv disponible) `MPEG4`, `WMV (Windows)`, `OGG Theora`, `MP3 (audio only)`, `OGG Vorbis (audio only)`, `Original (audio only)`. Si ffmpeg no está instalado, esos modos **no aparecen** en el combo. | [V] | `SRC:clipgrab.cpp:188-209`, `SRC:converter_copy.cpp:28`, `SRC:converter_ffmpeg.cpp:216-222, 331-375` |
| 3 | La última elección de formato se recuerda entre sesiones (`LastFormat`). | [V] | `SRC:mainwindow.cpp:89-95` |
| 4 | La elección de calidad es libre; la resolución elegida se recuerda si "Remember video quality" está activo. | [V] | `SRC:mainwindow.cpp:813-820` |
| 5 | No hay previsualización de tamaños de archivo en el combo de calidad en la pestaña principal (los tamaños se usan internamente para el porcentaje de progreso). | [V] | `SRC:download_list_model.cpp:94-117`, `SRC:video.h:34-54` |

## F4. Iniciar una descarga

| # | Paso | Estado | Evidencia |
|---|------|--------|-----------|
| 1 | El usuario pulsa **"Grab this clip!"** (es: "Descargar este vídeo") o pulsa Enter en el campo URL. | [V] | `SRC:mainwindow.ui` (downloadStart), `SRC:mainwindow.cpp:585-587` |
| 2 | Se abre un diálogo nativo **"Select Target"** para elegir nombre y carpeta destino, con el nombre saneado del vídeo como propuesta (carpeta por defecto: la última usada o el Escritorio). Si "Never ask for file name" está marcado, no se pregunta y se usa directamente la ruta configurada. | [V] | `SRC:mainwindow.cpp:263-277`, `SRC:clipgrab_es.ts` ("Seleccione el destino") |
| 3 | Si la carpeta destino indicada no existe, se desmarca "Never ask for file name" y se vuelve a pedir el destino. | [V] | `SRC:mainwindow.cpp:279-288` |
| 4 | Si "Use metadata" está activo y el formato elegido es solo-audio (índices 4-5: OGG Vorbis / Original audio only), se muestra un **diálogo modal de metadatos** (título/artista) que se aplicarán como etiquetas al archivo. | [V] | `SRC:mainwindow.cpp:296-310`, `SRC:metadata-dialog.ui` |
| 5 | La tarea se añade a la lista **"Current Downloads:"** (columnas: Portal, Title, Quality, Target format, progreso) y comienza a descargarse de inmediato. El campo URL se limpia y vuelve al estado inicial. | [V] | `SRC:clipgrab.cpp:678-694`, `SRC:mainwindow.cpp:312-318`, `SRC:download_list_model.cpp` |
| 6 | No existe botón de "añadir a cola sin iniciar": al encolar, la descarga arranca automáticamente. No se observa límite de descargas simultáneas en el código (cada `video` lanza su propio proceso yt-dlp). | [V] sin límite en el código; [U] comportamiento real con muchas descargas simultáneas | `SRC:clipgrab.cpp:678-694`, `SRC:video.cpp:39-46` |
| 7 | La descarga ejecuta `yt-dlp --newline --no-playlist --no-mtime -o <tmp>/cg-youtube-dl-%(id)s-%(format_id)s.%(ext)s -f <video> [<+audio>] <url>` en un directorio temporal. | [V] | `SRC:video.cpp:86-117` |

## F5. Pausar, reanudar, cancelar, reiniciar

| # | Paso | Estado | Evidencia |
|---|------|--------|-----------|
| 1 | La lista de descargas tiene 3 botones: **Pause selected download**, **Cancel selected download**, **Open the target folder of the selected download** — todos deshabilitados cuando no hay selección. | [V] | `SRC:mainwindow.ui` (downloadPause/downloadCancel/downloadOpen), `SRC:mainwindow.cpp:466-485` |
| 2 | Pausa: solo permitida en estado `downloading`; pasa por estado intermedio `pausing` mientras se termina el proceso (terminate) y acaba en `paused`. El botón/conmutador muestra "Resume download" cuando la tarea está pausada. | [V] | `SRC:video.cpp:119-137`, `SRC:mainwindow.cpp:722-724` |
| 3 | Reanudar: vuelve a lanzar la descarga con los mismos parámetros; los archivos `.part` de yt-dlp permiten continuar (la plantilla de salida es estable). | [V] mecanismo (re-llamada a `download()`), [I] la reanudación real depende del soporte de `.part` de yt-dlp | `SRC:video.cpp:133-137` |
| 4 | Cancelar: permitida en `downloading`/`paused`; estado intermedio `canceling`, se eliminan los archivos temporales y acaba en `canceled`. | [V] | `SRC:video.cpp:119-124, 593-597, 630-636` |
| 5 | Reiniciar: disponible solo para tareas `canceled` o `error`, desde el menú contextual ("Restart download"). | [V] | `SRC:video.cpp:139-143`, `SRC:mainwindow.cpp:716, 744-745` |
| 6 | Menú contextual de una fila: *Open downloaded file*, *Open target folder* / *Show in Finder* (macOS), *Pause/Resume download*, *Restart download*, *Cancel download*, *Copy video link*, *Open video link in browser*. Las acciones no aplicables al estado se ocultan (p. ej. de una tarea `finished` desaparecen pausa/reiniciar/cancelar). | [V] | `SRC:mainwindow.cpp:703-766` |
| 7 | Doble clic sobre una fila finalizada abre el archivo descargado. | [V] | `SRC:mainwindow.cpp:827-832` |
| 8 | Al cerrar la ventana con descargas en curso: diálogo de confirmación *"There is still at least one download in progress…"*; si se confirma la salida, se cancelan todas y la ventana se cierra solo cuando ya no queda ninguna activa. | [V] | `SRC:mainwindow.cpp:492-510` |

## F6. Progreso y errores durante la descarga

| # | Paso | Estado | Evidencia |
|---|------|--------|-----------|
| 1 | La columna de estado muestra: `Starting ...` (aún sin tamaño), `NN,N % (descargado/total)` con unidades B/KiB/MiB/GiB/TiB (nunca llega a "100.0" — se topa en 99,9 hasta terminar), `Converting ...`, `Finished`, `Paused`, `Canceled`, `Waiting`, `Failed`. | [V] | `SRC:download_list_model.cpp:94-117`, `SRC:clipgrab.cpp:811-822` |
| 2 | El progreso se calcula con los tamaños reales en disco de los archivos temporales, calibrados contra las estimaciones de tamaño que emite yt-dlp en su salida (`[download] N.N% of ~?N.N(T|G|M|K)iB`). | [V] | `SRC:video.cpp:389-450` |
| 3 | La conversión posterior (ffmpeg) se muestra como `Converting ...` **sin porcentaje** (no hay progreso granular de ffmpeg). | [V] | `SRC:download_list_model.cpp:105-106`, `SRC:converter_ffmpeg.cpp` (sin reporting de progreso) |
| 4 | Errores de yt-dlp (líneas `ERROR: …`) → estado `Failed` y el proceso se mata. | [V] | `SRC:video.cpp:452-459` |
| 5 | Los errores de conversión ponen la tarea en `Failed`; los errores globales (p. ej. ffmpeg ausente) se muestran en un `QMessageBox` modal con botón OK. | [V] | `SRC:video.cpp:625-628`, `SRC:clipgrab.cpp:655-672` |
| 6 | Mensajes de error documentados en el código: *"No installed version of avconv or ffmpeg could be found. Converting files and downloading 1080p videos from YouTube is not supported."*; *"The installed version of %1 is outdated. Downloading 1080p videos from YouTube is not supported."* (sin soporte HLS); error SSL durante la descarga de yt-dlp; *"Unable to write to %1"*; *"Error downloading youtube-dlp: %1"*. | [V] | `SRC:converter_ffmpeg.cpp:356-372`, `SRC:clipgrab.cpp:598, 622, 630` |
| 7 | Al finalizar todas las descargas: notificación de bandeja *"All downloads finished"* (o por descarga si el ajuste lo pide). | [V] | `SRC:mainwindow.cpp:522-528` |

## F7. Convertir un archivo local

| # | Paso | Estado | Evidencia |
|---|------|--------|-----------|
| 1 | **No existe en ClipGrab 3.9.14 un flujo de "convertir un archivo local ya existente"**: la conversión solo ocurre como fase automática posterior a una descarga (`downloading → converting → finished`). | [V] por ausencia en toda la UI y en la CLI (`--start-minimized` es la única opción) | `SRC:mainwindow.ui` (4 pestañas, sin "Convert"), `SRC:main.cpp:45-51` |
| 2 | Los presets personalizados de conversión (pestaña Settings → Conversion: códecs audio/vídeo, bitrate, fps, deshabilitar audio/vídeo) existen en la UI, pero la pestaña se elimina al construir la ventana (`removeTab(2)` con comentario `//fixme!`), por lo que en 3.9.14 **no es accesible**. | [V] | `SRC:mainwindow.cpp:203`, `SRC:mainwindow.ui` (tab_5 "Conversion") |
| 3 | Mecánica interna de conversión disponible (relevante para el clon): ffmpeg `-y -i <entrada>`, copia directa de códecs si ya son compatibles, o recodificación (audio 256k; vídeo bitrate×1.2), metadatos title/author/artist, resolución automática de contenedor para modo solo-audio. | [V] | `SRC:converter_ffmpeg.cpp:26-211` |
| 4 | El modo `Original` no reconvierte: copia el archivo temporal al destino con su extensión original. | [V] | `SRC:converter_copy.cpp:31-60` |
| 5 | Conflicto de nombre de salida: si `nombre.ext` existe, se usa `nombre-1.ext`, `nombre-2.ext`, … (sin preguntar ni sobrescribir). | [V] | `SRC:converter_ffmpeg.cpp:180-193`, `SRC:converter_copy.cpp:33-46` |

## F8. Historial y preferencias

| # | Paso | Estado | Evidencia |
|---|------|--------|-----------|
| 1 | **ClipGrab 3.9.14 no tiene historial persistente de descargas**: no hay pestaña/vista de historial en la UI; la lista "Current Downloads" solo contiene la sesión actual y las tareas terminadas se retiran de la lista si "Remove finished downloads from list" está activo (por defecto desactivado). | [V] por ausencia (análisis del .ui completo y del modelo de datos) | `SRC:mainwindow.ui`, `SRC:clipgrab.cpp:681-687` |
| 2 | Preferencias persistidas (QSettings, organización "ClipGrab" / dominio clipgrab.org), todas aplicadas al instante sin botón guardar: carpeta destino (`savedPath`, por defecto el Escritorio), "Always save at the last used path" (por defecto **sí**), "Never ask for file name" (por defecto no), "Use metadata" (no), Clipboard: always/never/ask (por defecto **ask**; en macOS se fuerza *always*), Notifications: after each/after all/never (por defecto *after all*), "Remember video quality" (sí), "Use WebM if possible" (no), "Remove finished downloads from list" (no), "Minimize to system tray" (no), "Force IPv4" (no), idioma (auto) con reinicio requerido, formato último (`LastFormat`), pestaña última (`MainTab`), calidad recordada (`rememberedVideoQuality`), proxy completo (host, puerto, tipo, usuario, contraseña). | [V] | `SRC:mainwindow.cpp:127-200, 365-407, 504-520, 530-582, 794-820, 835-837`, `SRC:main.cpp:40-42` |
| 3 | Existen ajustes ocultos/deshabilitados en la UI en 3.9.14: "Remember logins" y "Ignore SSL errors" (widgets ocultos con `hide()`). | [V] | `SRC:mainwindow.cpp:180-184` |
| 4 | El idioma se elige en Settings → Language y exige reiniciar la aplicación para aplicarse; incluye reconocimiento automático. | [V] | `SRC:mainwindow.ui` (tab_8), `SRC:main.cpp:69-86` |

---

## Matriz resumen de estados de tarea observados

Estado interno (`video::state`) → etiqueta visible en la lista (`SRC:download_list_model.cpp`):

| Estado interno | Etiqueta visible | ¿Aparece como fila? |
|---|---|---|
| `empty` | — (inicial) | no |
| `unfetched` | Waiting | sí |
| `fetching` | — (solo infobox "Please wait…") | — |
| `fetched` | — (listo para configurar) | — |
| `downloading` | `Starting ...` / `NN,N % (x/y)` | sí |
| `converting` | `Converting ...` | sí |
| `paused` | `Paused` | sí |
| `pausing` | (transitorio) | sí |
| `canceling` | (transitorio) | sí |
| `canceled` | `Canceled` | sí |
| `finished` | `Finished` | sí (se retira si el ajuste lo pide) |
| `error` | `Failed` (default) | sí |

## Casos límite y estados vacíos identificados

- **Sin descargas activas**: los 3 botones de la lista quedan deshabilitados. [V] `SRC:mainwindow.cpp:466-485`
- **Búsqueda sin resultados**: página de error embebida con "No results found." [V] `SRC:mainwindow.cpp:659-663`, `SRC:loading-failed.html`
- **Red caída / URL errónea**: mensaje genérico del infobox (F2.6); no se ofrece reintento desde el infobox (solo menú contextual Restart). [V]/[U]
- **ffmpeg/avconv ausente**: modos de conversión desaparecen del combo y aviso modal al arrancar; las descargas siguen siendo posibles en calidad estándar. [V] `SRC:converter_ffmpeg.cpp:331-375`
- **yt-dlp ausente u obsoleto (< 2021.09.25)**: al arrancar se ofrece descargarlo con un diálogo con barra de progreso; si se rechaza, la aplicación se cierra. [V] `SRC:clipgrab.cpp:560-637`, `SRC:main.cpp:94-101`
- **Vídeo sin formato aceptable** (p. ej. solo WebM con WebM desactivado): el análisis termina sin calidades → `error` → mensaje genérico. [V] `SRC:video.cpp:559-564`
- **Playlists / enlaces con `&list=`**: con `--no-playlist` se descarga solo el vídeo indicado; la búsqueda interna usa listas planas de 16 resultados (`ytsearch16:`). El soporte real de descarga de playlists completas no está expuesto en la UI. [V] `SRC:video.cpp:80, 48-59`; [U] comportamiento de URLs de playlist puras
- **Vídeo privado/de pago o con restricción de edad**: tratado como error genérico de yt-dlp → `Failed`. [I] (no hay tratamiento especial en el código)
- **Copiar vídeo mientras hay un diálogo modal abierto**: el aviso de portapapeles se suprime. [V] `SRC:mainwindow.cpp:409-411`
