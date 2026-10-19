# Especificación — Requisitos funcionales y no funcionales (MVP)

> Fase 2 · Fecha: 2026-10-19 · Estado: BORRADOR — pendiente de aprobación
> Trazabilidad: cada requisito referencia su base en `docs/rea/` (nº de función del inventario `02-feature-inventory.md`) y/o en AGENTS.md §5.1.
> Prioridades: **P0** = obligatorio para MVP · **P1** = deseable si el presupuesto de MVP lo permite · **P2** = post-MVP (solo con item de backlog aprobado).
> Stack verificado el 2026-10-19 (ver `architecture.md` §1): Tauri 2.12.4, Vue 3.5.44, TypeScript 5.x, Rust estable, yt-dlp 2026.08.19, FFmpeg estable 9.0.2, Vitest 5, Playwright 1.64.

---

## 1. Alcance del producto

Aplicación de escritorio multiplataforma (Windows, macOS, Linux) con identidad propia ("REA"), en español, para:

1. Analizar URLs de vídeo y mostrar los formatos/calidades realmente disponibles (REA: A2, B1–B3).
2. Descargar contenido autorizado del usuario mediante yt-dlp, con cola, progreso, cancelación y reintento (REA: C1–C11).
3. Convertir archivos locales con FFmpeg/FFprobe (AGENTS.md §5.1.8; no existe en la referencia — diferencia deliberada).
4. Mantener historial local de operaciones y preferencias (AGENTS.md §5.1.11; la referencia no tiene historial — diferencia deliberada).

Fuera de alcance MVP (AGENTS.md §5.2 + REA D1–D3, E9, F4–F6, G3): portapapeles, búsqueda integrada, drag & drop, notificaciones del sistema, proxy, minimizar a bandeja, WebM opcional, presets avanzados, auto-actualización, soporte de descargas autenticadas.

## 2. Requisitos funcionales

### RF-01 — Entrada y validación de URLs (P0)
- Base: REA A1, A3, A7 + AGENTS.md §5.1.1, §7.
- La vista principal incluye un campo de URL con botón "Analizar" y acción con Enter.
- Validación en el frontend **y** revalidación en Rust: solo esquemas `http`/`https` con host válido. `file:`, `javascript:`, rutas locales y cadenas vacías se rechazan con mensaje específico (EC-01…EC-03).
- Mientras se analiza, el campo queda deshabilitado y se muestra estado "Analizando…" (REA F2.1); siempre es recuperable (botón/habilitación) incluso si el análisis falla.
- Diferencia respecto a la referencia (REA A1: análisis automático al pegar): el análisis se dispara explícitamente. **Decisión-06**.
- Criterios de aceptación:
  - CA-01.1: pegar una URL `https://…` válida y pulsar Analizar inicia el análisis (estado visible).
  - CA-01.2: `file:///etc/passwd`, `javascript:alert(1)`, `ftp://x`, texto sin URL y vacío → mensaje de error específico, sin iniciar proceso.
  - CA-01.3: URL válida de YouTube con parámetros sobrantes (`&list=…`, `&t=…`) se normaliza antes del análisis (REA A3).
  - CA-01.4: tras un error de análisis, el campo queda editable y se puede reintentar sin reiniciar la app.
- Pruebas: unitarias de validación (tabla de casos), de estado de la vista; integración con motor falso.

### RF-02 — Análisis de metadatos y opciones disponibles (P0)
- Base: REA A2, B2–B3, F2 + AGENTS.md §5.1.2–5.1.3.
- El análisis ejecuta el motor (adapter) con argumentos cerrados: `yt-dlp -J --no-playlist <url>` (patrón REA §4, validado) desde Rust con lista de argumentos (nunca string concatenado).
- Se muestran: título, portal, duración, miniatura (si existe), y la lista de calidades derivada **exclusivamente** de los formatos reportados por el motor (sin opciones inventadas).
- Emparejamiento audio+vídeo compatible (REA B3: m4a/aac con mp4, opus/ogg con webm) y deduplicación por resolución+fps.
- Etiquetado de calidades tipo `1080p (HD)` / `2160p (4K)` / `NNp60` / `unknown` (REA B2).
- Si el motor devuelve error o JSON sin formatos → estado `failed` con error del catálogo (EC-10/EC-11) y la tarea no se marca nunca como lista.
- Criterios de aceptación:
  - CA-02.1: con JSON de análisis fijo (fixture), la UI muestra exactamente las calidades derivadas de ese JSON (test determinista).
  - CA-02.2: vídeo sin formatos aceptables → mensaje EC-11 y sin combos habilitados.
  - CA-02.3: ningún campo de la ficha queda vacío-sin-estado (título "desconocido" permitido, estado explícito).
- Pruebas: parser de JSON con fixtures (campos ausentes, vacíos, maliciosos), mapeo de calidades, estados.

### RF-03 — Selección de formato, calidad y destino (P0)
- Base: REA B1, B4, C1–C3, F3 + AGENTS.md §5.1.4, §5.1.9.
- Combo de formato de salida con los modos MVP: **MP4 (H.264/AAC si está disponible)**, **MP3 (solo audio)**, **Original (sin reconvertir)** — provenientes de capacidades reales del sistema (ver RF-08); si FFmpeg no está, solo "Original".
- Combo de calidad con las opciones de RF-02; prefijo de tamaños estimados cuando el motor los reporte.
- Selección de carpeta destino **solo** mediante diálogo nativo (plugin dialog de Tauri); nunca campo de texto libre de rutas; nombre de archivo propuesto saneado (REA C2).
- Política de conflicto explícita: preguntar / numerar automáticamente (`nombre-1.ext`) / cancelar (REA E5 + AGENTS.md §7 "no silent overwrites"). Por defecto: preguntar. **Decisión-04**.
- Criterios de aceptación:
  - CA-03.1: con FFmpeg presente se ofrecen los 3 formatos; ausente, solo "Original" y se muestra aviso (RF-08).
  - CA-03.2: el nombre propuesto no contiene caracteres prohibidos (`/ \ : * ? " < > | % $ ! ' @ + \`` # & { } =`) ni terminadores de路径 de macOS/Windows (REA C2, extendido).
  - CA-03.3: al elegir destino existente con conflicto se aplica la política configurada y nunca se sobrescribe silenciosamente.
- Pruebas: sanitizador, resolución de conflictos, estados habilitado/deshabilitado de combos.

### RF-04 — Descargas con progreso, velocidad, tamaño y ETA (P0)
- Base: REA C4–C6, F4, F6 + AGENTS.md §5.1.5–5.1.6.
- Descarga por `yt-dlp` con plantilla de salida en directorio temporal de la app (`cg-yt-…`, patrón REA §4.4) y `--newline` para progreso por línea; parámetros fijos y codificados en Rust.
- Progreso calculado desde stdout del motor (patrón REA §4) + tamaño de archivos `.part` en disco; se muestran: porcentaje (tope 99,9% hasta confirmación de fin — REA C6), bytes descargados/total (B/KiB/MiB/GiB/TiB), **velocidad instantánea** y **ETA** cuando el motor las reporta (`--newline` las emite); si no son fiables → estado indeterminado explícito (sin porcentaje falso).
- Estado `downloading` visible; varios números nunca "saltan" sin calibración (heurística REA §4: descartar cambios de estimación > 10%).
- Criterios de aceptación:
  - CA-04.1: con salida simulada del motor (fixture de líneas), el progreso/velocidad/ETA mostrados coinciden con lo calculado.
  - CA-04.2: el porcentaje no alcanza 100% hasta que el adapter confirme la finalización del proceso y (para conversión) del resultado.
  - CA-04.3: si el tamaño total no es detectable, se muestra progreso indeterminado, nunca 0% o 100% fijo.
- Pruebas: parser de líneas de progreso (velocidad, ETA, tamaños con sufijos K/M/G/T), acumulación multi-archivo, casos de líneas ruidosas.

### RF-05 — Cola de tareas, cancelación, reintento y errores (P0)
- Base: REA C4–C5, C7–C10, F5, F7 + AGENTS.md §5.1.5–5.1.7, §8.
- Cola con IDs únicos; tareas independientes (una fila por tarea, la más reciente arriba — REA C4).
- Máquina de estados: `queued → inspecting → ready → downloading → converting → completed | failed | cancelled`, con pausa `downloading → pausing → paused` y reanudación `paused → downloading` (**D-05 aprobada**: la pausa entra en el MVP siguiendo el patrón REA C7); transiciones legales documentadas en `data-model.md`; transiciones ilegales rechazadas y registradas.
- Cancelación: termina el proceso hijo del motor (kill real, no solo señal suave con timeout), limpia temporales y listeners; deja estado `cancelled` y el archivo parcial nunca se expone como completado (REA C8/F7 y AGENTS.md §8).
- Reintento: disponible para `failed`/`cancelled`; garantiza que no exista un proceso activo duplicado de la misma tarea (REA C9 y AGENTS.md §8).
- Al cerrar la app con tareas activas: confirmación (REA F5.8); al aceptar, se cancelan todas y la ventana cierra solo cuando no queda ningún proceso vivo.
- Criterios de aceptación:
  - CA-05.1: cada transición legal ocurre al ejecutar la acción correspondiente; intentar una transición ilegal (p. ej. `completed → downloading`) es imposible por diseño (tipos + tests).
  - CA-05.2: cancelar una tarea mata su proceso (verificación del PID/existencia en test de integración con proceso falso) y borra sus temporales.
  - CA-05.3: reintentar una tarea fallida no deja procesos duplicados (test de vida por PID).
  - CA-05.4: cerrar con N activas → confirmación; "sí" finaliza procesos y la app sale limpia (sin zombies).
  - CA-05.5: pausar una tarea activa deja estados `pausing → paused` (proceso terminado y verificado); reanudar la devuelve a `downloading` continuando los archivos parciales, sin proceso duplicado.
- Pruebas: máquina de estados (tabla de transiciones), cancelación/reintento con procesos falsos, cierre de app.

### RF-06 — Conversión de archivos locales (P0)
- Base: AGENTS.md §5.1.8–5.1.9, §9; referencia no lo tiene (diferencia deliberada, valor de producto propio).
- Vista "Convertir": seleccionar archivo local (diálogo nativo), FFprobe muestra contenedor/códecs/duración; elegir formato destino (MP4 H.264/AAC, MP3) y destino de salida con diálogo nativo.
- Validaciones previas: existencia del input, espacio disponible estimado, destino válido, conflicto de nombre (RF-03).
- Ejecución por FFmpeg con lista de argumentos; éxito solo si exit code = 0 **y** el archivo existe **y** tamaño > 0 (AGENTS.md §9); progreso determinado solo si se puede derivar de forma fiable (duración de FFprobe + tiempo/progreso de FFmpeg), si no, indeterminado (REA F6.3 como referencia de honestidad).
- El archivo de origen **nunca** se elimina (AGENTS.md §9); los archivos temporales se limpian.
- Cancelable en cualquier momento con limpieza (RF-05).
- Criterios de aceptación:
  - CA-06.1: conversión correcta de un fixture local pequeño (generado con FFmpeg en entorno de test) produce salida reproducible y estado `completed` con verificación de exit code + existencia + tamaño.
  - CA-06.2: input corrupto/códec desconocido → error específico del catálogo (EC-30/EC-31), sin estado falso `completed`.
  - CA-06.3: cancelación a mitad → proceso muerto, temporales limpios, origen intacto, estado `cancelled`.
  - CA-06.4: conflicto de nombre → se aplica la política; el origen nunca se sobrescribe.
- Pruebas: adapter de FFprobe/FFmpeg con binarios reales (test de integración marcado) y con fakes; manejo de errores.

### RF-07 — Historial local de operaciones (P0)
- Base: AGENTS.md §5.1.11; ausente en la referencia (REA F8.1) — diferencia deliberada.
- Cada operación (descarga o conversión) finalizada, fallida o cancelada se registra localmente: tipo, título, portal, formato/calidad, origen (URL redactada si procede), destino, tamaño final si existe, timestamps, resultado, código de error si aplica.
- Vista "Historial": lista cronológica con estado y filtros básicos (todas/completadas/fallidas); acciones: abrir archivo (si existe), abrir carpeta, copiar ruta, eliminar entrada (con confirmación).
- Sin telemetría ni envío externo; persistencia local según `data-model.md`.
- Criterios de aceptación:
  - CA-07.1: completar/fallar/cancelar una operación genera exactamente una entrada coherente con lo ocurrido.
  - CA-07.2: reiniciar la app conserva el historial; eliminar una entrada la retira de forma persistente.
  - CA-07.3: ninguna URL con credenciales (`user:pass@host`) se persiste íntegra — se redacta.
- Pruebas: persistencia SQLite con casos de corrupción (integridad fallida → backup/recuperación segura), migraciones y redacción de credenciales.

### RF-08 — Diagnóstico de dependencias (P0)
- Base: REA G1–G2, F6.6 + AGENTS.md §5.1.12.
- Al arrancar y bajo demanda (vista "Diagnóstico"): detectar yt-dlp y FFmpeg/FFprobe (PATH + ubicación gestionada por la app), ejecutar `--version`/`-version` con timeout, y mostrar versión, ruta y estado (ok / ausente / incompatible).
- Ausente/incompatible → mensaje accionable en español con instrucciones de instalación por SO; sin descargas automáticas en MVP (**Decisión-03**); las funciones afectadas se degradan con aviso visible (REA: los modos ffmpeg desaparecen del combo — patrón replicable).
- Criterios de aceptación:
  - CA-08.1: con motores presentes, diagnóstico muestra versión y ruta reales.
  - CA-08.2: con PATH simulado sin motores, diagnóstico informa ausencia y la UI deshabilita lo dependiente (descarga sigue posible solo si el motor de descarga existe; conversión requiere FFmpeg).
  - CA-08.3: un binario existente que falla al ejecutarse se reporta como "incompatible" con el detalle técnico disponible para soporte.
- Pruebas: detector con rutas falsas y binarios falsos (scripts de test), timeouts.

### RF-09 — Preferencias (P0)
- Base: REA B4, F1, F8.2 + AGENTS.md §5.1.11.
- Ajustes MVP: carpeta destino por defecto (última usada — REA F8.2), "nunca preguntar nombre de archivo" (REA C3), política de conflicto (RF-03), recordar última calidad y último formato (REA B4), idioma (fijo "es" en MVP con arquitectura i18n preparada), política de finalizados en cola (retirar de la lista — REA F8.2).
- Persistencia local al instante (sin botón guardar — patrón REA F8.2); vista "Preferencias" con estado vacío/valores por defecto documentados.
- Criterios de aceptación: CA-09.1: cambiar un ajuste y reiniciar lo conserva; CA-09.2: "restaurar valores por defecto" disponible; CA-09.3: nunca se persisten secretos.
- Pruebas: carga/guardado, valores por defecto, migración de esquema de preferencias.

### RF-10 — Interfaz accesible, responsive de ventana y en español (P0)
- Base: AGENTS.md §5.1.10, §5.1.13, §10; REA §7 (estados).
- Español en todos los textos (centralizados en módulo i18n); navegación por teclado completa, foco visible, etiquetas/roles ARIA, contraste adecuado, anuncios de estado para lectores de pantalla (aria-live) en cambios de progreso/estado.
- Estados de UI completos en cada vista: vacío, cargando, éxito, cancelado, error (AGENTS.md §10) — referencias de estados REA §7.
- Layout de ventana: 4 vistas (Descargas · Convertir · Historial · Diagnóstico/Ajustes) + vista principal de tarea; funcional de ~760×560 px hasta maximizado (la referencia usa 917×632 — REA §1; el mínimo propio se fija en 760 px para portátiles pequeños).
- Criterios de aceptación: CA-10.1: todo control accesible por teclado con orden lógico; CA-10.2: contraste AA en temas claro/oscuro; CA-10.3: cada vista tiene los 6 estados implementados y probables en tests.
- Pruebas: Playwright sobre el frontend (teclado, roles, aria-live), snapshot de estados.

## 3. Requisitos no funcionales

| ID | Requisito | Prioridad | Fuente |
|---|---|---|---|
| RNF-01 | Toda ejecución de procesos con lista de argumentos; prohibida la concatenación de comandos | P0 | AGENTS.md §7; REA §7 (antipatrón de la referencia) |
| RNF-02 | Mínimo privilegio en capabilities de Tauri (solo permisos necesarios declarados) | P0 | AGENTS.md §7 |
| RNF-03 | Sin telemetría, sin cookies/credenciales, sin descargas autenticadas | P0 | AGENTS.md §5.3, §7; REA Q7 |
| RNF-04 | Logs locales sin secretos; URLs redactadas; errores de proceso con detalle técnico opcional | P0 | AGENTS.md §7, REA A7 |
| RNF-05 | Tiempo de arranque de la UI < 2 s en hardware modesto; la detección de dependencias es asíncrona y no bloquea | P1 | buen nivel (la referencia bloquea con `waitForFinished` — REA §4) |
| RNF-06 | Empaquetado: NSIS/MSI (Win), DMG (macOS), DEB/AppImage (Linux) verificables en CI local; sin bundler de ofertas de terceros | P0 | AGENTS.md §14; REA §1 (antipatrón TheCassiniLabs) |
| RNF-07 | Cumplimiento de licencias documentado (Tauri MIT/Apache-2.0, Vue MIT, yt-dlp Unlicense, FFmpeg LGPL/GPL según build) antes de distribuir | P0 | AGENTS.md §14 |

## 4. Decisiones — APROBADAS (2026-10-19)

| ID | Decisión | Resolución aprobada |
|---|---|---|
| D-01 | Gestor de paquetes | ✅ **npm** |
| D-02 | Versión TypeScript | ✅ **TS 5.x estable** (ecosistema Vue/Vite verificado); TS 7 nativo se reevaluará más adelante |
| D-03 | Obtención de yt-dlp/FFmpeg | ✅ **Sin bundling en MVP**: diagnóstico + guía al usuario; verificación de hash si algún día se descarga |
| D-04 | Política de conflicto de archivos | ✅ **Preguntar por defecto**; opciones "numerar"/"cancelar" en preferencias |
| D-05 | Pausa de descargas | ✅ **EN MVP (P0)**: estados `pausing/paused` + reanudación, patrón REA C7 (RF-05/CA-05.5) |
| D-06 | Disparador del análisis | ✅ **Botón/Enter explícito** |
| D-07 | Persistencia historial/preferencias | ✅ **SQLite** (rusqlite embebido con feature `bundled`, WAL — ver `data-model.md` §3) |
| D-08 | Entorno de desarrollo | ✅ **Aprobado**: instalar rustup/cargo, yt-dlp, ffmpeg+ffprobe e inicializar git al iniciar la Fase 3 (backlog B-01) |

Con estas decisiones aprobadas, la especificación queda cerrada y la Fase 3 puede iniciarse ejecutando el bloque 0 del backlog (`mvp-backlog.md`).
