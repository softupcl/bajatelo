# REA — 01. Investigación de la aplicación de referencia ClipGrab

> Estado: FASE 1 COMPLETADA (investigación estática) · Fecha: 2026-10-19
> Producto objetivo: aplicación de escritorio independiente inspirada en la experiencia observable de ClipGrab (clon limpio-room, sin copiar código/marcas/assets).
> Clasificación de cada hallazgo (AGENTS.md §4.2): **[Verificada]** / **[Inferida]** / **[Unknown]** / **[No aplicable]**.

## 1. Alcance y método

### 1.1 Qué se ha investigado

| Fuente | Tipo de acceso | Resultado |
|---|---|---|
| Sitio oficial `https://clipgrab.org/` (página de inicio) | Lectura directa (fetch) — 2026-10-19 | ✅ Accesible. Versiones publicadas: Windows 3.9.15 (portable), macOS 4.0.0-beta2, Linux 3.9.14 (AppImage) + **código fuente 3.9.14 en tarball**. Declara explícitamente: *"ClipGrab is a GPLv3-licensed Open Source project"*. [Verificada] |
| Documentación oficial: `/faqs/supported-sites`, `/faqs/supported-formats`, `/faqs/howto-download-youtube-video`, `/faqs/why-is-clipgrab-not-working`, `/support`, `/legal` | Lectura directa (fetch) — 2026-10-19 | ✅ Accesibles (ver §3). [Verificada] |
| Código fuente oficial **3.9.14** (GPLv3) — `https://download.clipgrab.org/clipgrab-3.9.14.tar.gz` (enlace tomado de la página oficial) | Descargado y extraído en directorio temporal **fuera del repositorio** (`$TMP/opencode/rea-reference/clipgrab-3.9.14/`); inspeccionado con lectura/grep. No se modificó nada. | ✅ 34 archivos fuente C++/Qt, 6 ficheros `.ui`, 34 traducciones `.ts/.qm`, recursos. Licencia GPLv3 confirmada (`COPYING`). [Verificada] |
| Aplicación de referencia en ejecución | Detección local (/Applications, ~/Applications, PATH, Spotlight) | ❌ **No instalada en esta máquina** → sin REA dinámico (sin ejecutar la app, sin capturas de pantalla propias, sin observar procesos en vivo). [Verificada la ausencia] |
| Repositorio de desarrollo en GitHub (`clipgrabapp/clipgrab`) | Fetch | ❌ 404 — no existe en esa ruta. El desarrollo se distribuye vía tarballs oficiales; ubicación del repositorio activo: [Unknown] (búsqueda web no disponible en esta sesión) |

### 1.2 Herramientas REA disponibles / utilizadas

- ✅ Lectura de web oficial (fetch) y navegador Playwright disponible para observación web (no fue necesario para las páginas consultadas).
- ✅ Análisis de código fuente público (lectura/grep) — permitido por AGENTS.md §4.1 ("Public source code or packages whose licenses permit the intended inspection"); ClipGrab es GPLv3.
- ❌ App en ejecución → sin observación dinámica. Limitación registrada, no simulada (AGENTS.md §4.3).
- ❌ Búsqueda web (servicio cancelado en esta sesión) → limitación para localizar el repositorio activo y comentarios de versiones recientes.

### 1.3 Qué NO se ha hecho (por restricción de protocolo)

- No se ejecutó la app de referencia ni se descargó ningún contenido de terceros.
- No se alteró el tarball original; la extracción se hizo en copia temporal.
- No se copiarán assets, marcas ni textos al clon.

## 2. Perfil de la aplicación de referencia

- **ClipGrab 3.9.x** es una aplicación de escritorio **Qt5/C++** (módulos: core, gui, widgets, network, xml, **webenginewidgets**) construida con qmake. [Verificada: `SRC:clipgrab.pro`, `SRC:README`]
- Motor de análisis/descarga: **yt-dlp externo** invocado como proceso (preferentemente una copia descargada por la propia app, si no, la del sistema), ejecutado con un intérprete Python (incluido en el bundle en Windows/macOS; `python3` del sistema en Linux). [Verificada: `SRC:youtube_dl.cpp`]
- Conversión: **ffmpeg o avconv externos** (binario del bundle en macOS o del sistema), con un conversor alternativo "copy" que no reconvertir. [Verificada: `SRC:converter_ffmpeg.cpp`, `SRC:converter_copy.cpp`]
- Plataformas oficiales publicadas: Windows, macOS, Linux. [Verificada: página oficial]
- Licencia del proyecto de referencia: **GPLv3**. [Verificada: `SRC:COPYING` + página oficial]

## 3. Hallazgos de la documentación oficial

| Hallazgo | Clasificación | Evidencia |
|---|---|---|
| Sitios con soporte "oficial" declarado: **YouTube, Vimeo, Dailymotion, Facebook** (incluye vídeos privados de Facebook), más "miles de sitios" por detección avanzada no oficial. | Verificada (texto oficial) | `WEB:/faqs/supported-sites` |
| Calidades de YouTube declaradas: HD 1080p/720p y SD 480p/360p/240p. | Verificada (texto oficial) | `WEB:/faqs/supported-sites` |
| Formatos de salida declarados: **MPEG4, MP3, WMV, OGG Vorbis, OGG Theora, Original** (FLV/MPEG4/WebM sin reconvertir). | Verificada (texto oficial) | `WEB:/faqs/supported-formats` |
| MPEG4 "inteligente": si el origen ya es MPEG4, no reconvierte (calidad original). | Verificada (texto oficial) | `WEB:/faqs/supported-formats` |
| Búsqueda integrada de YouTube dentro de la app. | Verificada (texto oficial) | `WEB:/faqs/supported-sites` |
| Flujo oficial documentado (5 pasos): copiar enlace → pegar en pestaña Downloads → elegir formato/calidad → "Grab this clip!" → elegir nombre de archivo → descarga. | Verificada (texto oficial + capturas de la web) | `WEB:/faqs/howto-download-youtube-video` |
| Guía de problemas: actualizar la app (los cambios de los sitios rompen compatibilidad) y revisar conexión/proxy mal configurado. | Verificada (texto oficial) | `WEB:/faqs/why-is-clipgrab-not-working` |
| El instalador de Windows incluye ofertas de terceros gestionadas por TheCassiniLabs (dotInstaller), operado de forma independiente del proyecto. | Verificada (texto oficial) | `WEB:/legal` |
| Responsable legal: libre media labs GmbH (Alemania). Política de privacidad de la web: datos técnicos anónimos. | Verificada (texto oficial) | `WEB:/legal` |

## 4. Hallazgos principales del código fuente (resumen)

Detalle completo en `05-technical-findings.md`; aquí solo el índice:

1. Modelo de estados de tarea con 12 estados, incluidos estados intermedios `pausing`/`canceling`. [Verificada]
2. Análisis de URL = `yt-dlp -J` (JSON) con normalización previa de URLs de YouTube. [Verificada]
3. Descarga = `yt-dlp` a carpeta temporal con plantilla `cg-youtube-dl-*`; progreso calculado con tamaños reales en disco + estimaciones de yt-dlp. [Verificada]
4. Conversión = ffmpeg con estrategia "copiar códec si es compatible, si no recodificar"; conflicto de nombres con sufijos `-1`, `-2`… [Verificada]
5. Reconocimiento de URLs conocidas (portapapeles) mediante 12 expresiones regulares fijas (YouTube, youtu.be, Vimeo, Dailymotion, Facebook, ARD, ZDF). [Verificada]
6. Búsqueda integrada = `yt-dlp ytsearch16:` renderizada en HTML embebido. [Verificada]
7. Autogestión de dependencias: la app descarga yt-dlp desde GitHub si falta o está obsoleto y avisa si ffmpeg falta o es antiguo. [Verificada]
8. Sin historial persistente de descargas; ajustes en QSettings aplicados al instante. [Verificada]
9. Sistema de auto-actualización con verificación SHA-1 y canal de "comandos remotos" (ver riesgo en §6). [Verificada]
10. Traducción oficial al español incompleta en 3.9.14 (varias cadenas `unfinished`). [Verificada]

## 5. Coincidencias y diferencias con la base tecnológica propuesta del clon

| Aspecto | ClipGrab 3.9.14 | Clon propuesto (AGENTS.md §3) | Comentario |
|---|---|---|---|
| Shell de UI | Qt 5 Widgets + QtWebEngine | Tauri 2 + Vue 3 | Diferencia deliberada; los patrones de UI son trasladables |
| Motor de descarga | yt-dlp (proceso externo vía Python) | yt-dlp | **Mismo motor** — la integración (detección, versiones, errores) es directamente comparable |
| Conversión | ffmpeg/avconv externo | FFmpeg/FFprobe | Equivalente; el clon añade FFprobe para inspección |
| Localización | ~35 idiomas (es incompleto) | Español primero | El clon puede superar a la referencia en es-ES |
| Historial | No existe | MVP del clon (§5.1.11) | Diferencia de producto decidida por el backlog propio |

## 6. Riesgos y notas legales/seguridad detectadas en la referencia

- El instalador oficial de Windows incluye **ofertas de terceros** (TheCassiniLabs). El clon no debe replicar bundling de ofertas. [Verificada: `WEB:/legal`]
- El sistema de actualizaciones de ClipGrab procesa un XML remoto con **comandos ejecutables** (`message`, `open`, `set`, `unset`, `die`) que puede cambiar ajustes locales o cerrar la app. Patrón a **no replicar** por principio de mínimo privilegio. [Verificada: `SRC:clipgrab.cpp:406-477`]
- La contraseña del proxy se guarda en **texto plano** en QSettings. El clon debe evitarlo. [Verificada: `SRC:mainwindow.cpp:539`]
- La construcción de líneas de comando de ffmpeg se hace por **concatenación de strings** con comillas (no por lista de argumentos). Deficiencia a no replicar (riesgo de inyección con nombres de archivo maliciosos). [Verificada: `SRC:converter_ffmpeg.cpp:38-194`]

## 7. Elementos NO verificables en esta fase (remiten a `06-open-questions.md`)

- Comportamiento dinámico real de pausa/reanudación con archivos `.part`, tiempos de análisis, apariencia exacta en pantalla en 3.9.15/4.0-beta, y cambios recientes de compatibilidad con los portales.
