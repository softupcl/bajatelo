# REA — 04. Inventario de interfaz de usuario de ClipGrab

> Estado: BORRADOR FASE 1 · Fecha de investigación: 2026-10-19
> Evidencia: `SRC:<archivo>` = código fuente oficial 3.9.14 (GPLv3), extraído del tarball oficial (`https://download.clipgrab.org/clipgrab-3.9.14.tar.gz`). El análisis de UI proviene de `mainwindow.ui` (definición XML de Qt) y `mainwindow.cpp` — es la descripción exacta de la interfaz, no una reconstrucción visual. No hay capturas de la app en ejecución (no instalada localmente); las imágenes de la web oficial (`/assets/clipgrab-download.png`, `/assets/formats.png`) son evidencia complementaria de la versión 3.x. `WEB:<url>` = página oficial consultada el 2026-10-19.
> Clasificación: **[V]** Verificado · **[I]** Inferido · **[U]** Unknown.

> Nota REA / licencia: este inventario describe estructuras y textos funcionales con fines de análisis de usabilidad (AGENTS.md §4). El clon (REA) no debe copiar assets gráficos, marcas ni textos literales de ClipGrab; debe mantener una identidad propia (AGENTS.md §1, §10).

---

## 1. Ventana principal

- Título: *"ClipGrab - Download and Convert Online Videos"*. Tamaño por defecto **917×632 px**. [V] `SRC:mainwindow.ui:4-15`
- Estructura vertical: logo superior centrado → pestañas principales → pie con versión y enlace web (*"ClipGrab 3.9.14 | www.clipgrab.org"*). [V] `SRC:mainwindow.ui:98-1258`
- **4 pestañas principales** (QTabWidget `mainTab`): 1) Search, 2) Downloads, 3) Settings, 4) About. [V] `SRC:mainwindow.ui:141-1243`
- Atajos de teclado: **Ctrl+1…Ctrl+4** saltan a cada pestaña. [V] `SRC:mainwindow.cpp:227-242`
- Barra de menú definida pero **vacía** en el .ui (sin menús estándar: no hay Archivo/Edición visible en la definición). [V] `SRC:mainwindow.ui:1261-1270`; [U] si el binario añade menús en runtime
- Al arrancar se restaura la última pestaña usada. [V] `SRC:mainwindow.cpp:247`
- Icono de **bandeja del sistema** siempre visible con menú de activación (mostrar/ocultar ventana). [V] `SRC:mainwindow.cpp:54-59, 435-444`
- Drag & drop de enlaces sobre la ventana → pestaña Downloads con la URL. [V] `SRC:mainwindow.cpp:768-779`
- Fuente global "Segoe UI". [V] `SRC:mainwindow.ui:21-24`
- Color de acento de los cuadros informativos: cian `#00B4DE`. [V] `SRC:mainwindow.ui:37-43` (dato de observación, no de copia)

## 2. Pestaña "Search" (Búsqueda)

| Elemento | Tipo | Detalle |
|---|---|---|
| Cuadro informativo superior | QLabel azul | *"Enter keywords in the box below in order to search videos on YouTube"* [V] `SRC:mainwindow.ui:163-175` |
| Campo de palabras clave | QLineEdit | Filtra con debounce de 1,5 s; si el texto es una URL http(s) redirige a la pestaña Downloads [V] `SRC:mainwindow.cpp:599-614` |
| Visor web embebido | QWebEngineView (custom `CGWebEngineView`) | Muestra página de carga (`loading-progress.html`) y resultados como HTML propio: tarjetas con título, miniatura y duración (mm:ss / hh:mm:ss) [V] `SRC:mainwindow.cpp:616-665` |
| Comportamiento al hacer clic en un resultado | — | Rellena el campo de descarga y cambia a la pestaña Downloads [V] `SRC:mainwindow.cpp:667-671` |
| Estado vacío / sin resultados | Página embebida | `loading-failed.html` con el texto *"No results found."* [V] `SRC:mainwindow.cpp:659-663` |
| Sin menú contextual en el visor | — | `setContextMenuPolicy(Qt::NoContextMenu)` [V] `SRC:mainwindow.cpp:105` |

- Por defecto (campo vacío) muestra una lista inicial fija (playlist pública de YouTube). [V] `SRC:clipgrab.cpp:844-848`; [I] la utilidad de esta lista inicial para el usuario es discutible — pendiente de observar

## 3. Pestaña "Downloads" (Descargas) — flujo principal

Distribución (de arriba a abajo): [V] `SRC:mainwindow.ui:188-382`

1. **Cuadro informativo azul** (`downloadInfoBox`): instrucción inicial, "Espere mientras…" durante el análisis, o el **título del vídeo** en negrita cuando el análisis tiene éxito. [V] `SRC:mainwindow.cpp:74, 324, 330, 340`
2. **Campo de URL** (`downloadLineEdit`): ancho completo, con foco al iniciar; se bloquea durante el análisis. Enter = iniciar descarga. [V]
3. Fila de configuración: etiqueta **"Format:"** + combo (`downloadComboFormat`, máx. 210 px) · etiqueta **"Quality:"** + combo (`downloadComboQuality`, máx. 210 px) · botón primario **"Grab this clip!"** (`downloadStart`). Los tres deshabilitados hasta que hay un vídeo analizado. [V]
4. Etiqueta **"Current Downloads:"** sobre la lista. [V]
5. **Lista de descargas** (`downloadTree`, QTreeView): 5 columnas — *Portal* (redimensiona a contenido), *Title* (estira), *Quality*, *Target format*, *Estado/Progreso* (220 px). Filas con colores alternos, sin árbol jerárquico, con menú contextual. Orden: la más reciente arriba. [V] `SRC:download_list_model.cpp`, `SRC:mainwindow.cpp:81-86`
6. Fila de botones: **"Pause selected download"**, **"Cancel selected download"**, **"Open the target folder of the selected download"** — deshabilitados sin selección; Pause/Cancel se deshabilitan para tareas `finished`/`canceled`. [V] `SRC:mainwindow.cpp:473-485`
7. Menú contextual de fila (ver flujo F5 en `03-user-flows.md`): Open downloaded file / Open target folder ("Show in Finder" en macOS) / Pause-Resume / Restart / Cancel / Copy video link / Open video link in browser. [V] `SRC:mainwindow.cpp:703-766`
8. Doble clic en una fila finalizada abre el archivo con la app asociada. [V] `SRC:mainwindow.cpp:827-832`

### Diálogos asociados al flujo de descarga

- **"Select Target"** — diálogo nativo de guardado de archivo (nombre propuesto = título saneado). [V] `SRC:mainwindow.cpp:263-277`
- **Metadatos (audio)** — diálogo modal con campos *Title* y *Artist* precargados, cuando "Use metadata" está activo y el formato es solo-audio. [V] `SRC:metadata-dialog.ui`, `SRC:mainwindow.cpp:296-310`
- **Confirmación de salida** — *"There is still at least one download in progress… Do you really want to quit ClipGrab now?"* (Sí/No). [V] `SRC:mainwindow.cpp:492-510`
- **Login** (`login_dialog.ui`) y **mensajes** (`message_dialog.ui`): definidos; el de login queda sin conectarse en 3.9.14 (ajuste "Remember logins" oculto). [V]/[U] usos residuales

## 4. Pestaña "Settings" (Ajustes)

QTabWidget lateral (pestañas al oeste) con dos secciones: [V] `SRC:mainwindow.ui:383-1120`

### 4.1 General — sub-pestañas horizontales

1. **Target Path**: campo de ruta + botón **"Browse …"** (diálogo nativo de carpeta), checkbox *"Always save at the last used path."*, checkbox *"Never ask for file name"*. [V]
2. **Metadata**: checkbox *"Use metadata"* (etiquetas ID3 en MP3/OGG). [V]
3. **Clipboard** *(eliminada en macOS)*: radio *"Always download" / "Never download" / "Always ask"*. [V] `SRC:mainwindow.cpp:211-212`
4. **Notifications**: radio *"After each download" / "After all downloads have been completed" / "Never"*. [V]
5. **Proxy**: checkbox *"Use a proxy server"* → grupo con Hostname/IP, Port (0-99999), tipo (*HTTP Proxy / Socks5 Proxy*), checkbox *"Proxy requires authentication"* → usuario y contraseña (campo con eco PasswordEchoOnEdit). [V]
6. **Other**: *"Remember logins"* (oculto), *"Remember video quality"* (activo por defecto), *"Remove finished downloads from list"*, *"Minimize ClipGrab to the system tray"*, *"Use WebM if possible"*, *"Ignore SSL errors"* (oculto), *"Force IPv4"*. [V] `SRC:mainwindow.cpp:178-185`
7. Nota: la sub-pestaña de índice 2 (Clipboard) se elimina en runtime por `removeTab(2)` con comentario `//fixme!` — en Windows/Linux la numeración de sub-pestañas visibles se ve afectada. [V] código; [U] efecto exacto en cada SO (probablemente elimina "Clipboard" fuera de macOS, donde ya se eliminó antes) — pendiente de verificar

### 4.2 Language

- Cuadro informativo + combo con ~35 idiomas (incluido Español; detección automática por defecto) y aviso: hay que **reiniciar** para aplicar. Idiomas RTL (farsi) activan dirección derecha-a-izquierda. [V] `SRC:clipgrab.cpp:35-183`, `SRC:main.cpp:69-86`

### 4.3 Conversion (oculta en 3.9.14)

- Definida en el .ui: *"Experts can create custom presets for the video conversion here."* — combo de presets, botones "Add new preset"/"Remove selected preset", grupos **Audio** (códec, bitrate kb/s, "Disable Audio") y **Video** (códec, bitrate, frames/second, "Disable video"). Se elimina en runtime (`removeTab(2)`, `//fixme!`) y no es accesible. [V] `SRC:mainwindow.ui:900-1116`, `SRC:mainwindow.cpp:203`

## 5. Pestaña "About"

- Bloque de apoyo/donación con botón de PayPal (abre PayPal en el navegador). [V] `SRC:mainwindow.cpp:688-691`
- Bloque de agradecimientos: *"ClipGrab relies on the work of the Qt project, the ffmpeg team, and the youtube-dlp team."* [V] `SRC:mainwindow.ui:1129`
- Bloque de traducciones con enlace para contribuir. [V]
- Etiqueta técnica de diagnóstico de dependencias: **"youtube-dlp: <ruta> (<versión>) / Python: <ruta> (<versión>)"**. [V] `SRC:mainwindow.cpp:781-792`
- Pie de ventana con versión y web oficial. [V]

## 6. Diálogos de sistema / inicio

- **Descarga de yt-dlp** (`helper_downloader.ui`): al primer arranque (o si la versión < mínima), diálogo con explicación, barra de progreso y botones continuar/salir; cancelar = cerrar la aplicación. [V] `SRC:clipgrab.cpp:560-637`, `SRC:helper_downloader.ui`
- **Actualización de la app** (`update_message.ui`): diálogo con notas de versión renderizadas en un visor web, botones de confirmar/más tarde/omitir; durante la descarga muestra *"Downloading update … %1/%2 MBytes"* con barra; verifica SHA-1 antes de abrir el instalador. [V] `SRC:clipgrab.cpp:266-558`
- **Notificaciones de bandeja**: *"ClipGrab: Video discovered in your clipboard"* (al detectar URL compatible copiada, modo "ask"), *"Download finished"* / *"All downloads finished"*. [V] `SRC:mainwindow.cpp:409-428, 522-528`

## 7. Estados de la UI (habilitado/deshabilitado)

| Situación | Campo URL | Combos F/Q | "Grab this clip!" | Botones de lista |
|---|---|---|---|---|
| Inicio (sin URL) | editable, foco | off | off | off |
| Analizando | **bloqueado** | off | off | off |
| Vídeo analizado (fetched) | editable | **on** | **on** | según selección |
| Error de análisis | editable | off | off | según selección |
| Sin selección en la lista | — | — | — | off |
| Tarea finished/canceled seleccionada | — | — | — | solo "Open" |

[V] `SRC:mainwindow.cpp:457-485`, `SRC:mainwindow.ui`

## 8. Textos visibles (en inglés; traducción es-ES parcial en el propio ClipGrab)

Textos funcionales clave observados (para referencia de contenido, no para copiar literalmente): los ya citados en `03-user-flows.md` F1–F8, más:

- Estados de fila: `Starting ...`, `Converting ...`, `Finished`, `Paused`, `Canceled`, `Waiting`, `Failed`. [V] `SRC:download_list_model.cpp`
- Unidades: `B, KiB, MiB, GiB, TiB` (con espacio fino U+2009 antes de la unidad). [V] `SRC:clipgrab.cpp:811-822`
- Hallazgo de i18n: la traducción española oficial de 3.9.14 tiene **muchas cadenas sin terminar** (`type="unfinished"`, p. ej. "Paused", "Canceled", "Failed", "Download finished"), y hay dos traducciones distintas para la misma cadena de instrucción inicial. [V] `SRC:clipgrab_es.ts`
