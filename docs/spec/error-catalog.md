# Especificación — Catálogo de errores

> Fase 2 · Fecha: 2026-10-19 · Estado: BORRADOR — pendiente de aprobación
> Reglas (AGENTS.md §10; lección REA A7 — la referencia muestra mensajes genéricos): todo error tiene (1) código estable, (2) mensaje de usuario en español accionable, (3) detalle técnico opcional para soporte, (4) acción sugerida. Nunca stack traces crudos al usuario. Los códigos viajan Rust → UI y al historial (`HistoryEntry.errorCode`).
> Clasificación de severidad: **I** info · **W** advertencia · **E** error de tarea · **C** crítico de app.

## 1. Validación de entrada (bloquea iniciar)

| Código | Severidad | Mensaje (es) | Causa raíz | Acción sugerida |
|---|---|---|---|---|
| EC-01 | W | "Introduce una URL para continuar." | campo vacío | Escribir o pegar la URL |
| EC-02 | W | "La URL no es válida. Debe comenzar por http:// o https://" | esquema no soportado (`file:`, `javascript:`, `ftp:`, ruta local) | Copiar de nuevo la dirección desde el navegador |
| EC-03 | W | "La URL no parece dirigirse a un sitio web válido." | host vacío/inválido | Verificar la dirección |

## 2. Análisis de metadatos

| Código | Severidad | Mensaje | Causa raíz | Acción |
|---|---|---|---|---|
| EC-10 | E | "No se pudo analizar el vídeo. Es posible que el enlace sea incorrecto o que el sitio no sea compatible." | el motor devuelve error/JSON vacío (REA A7) | Revisar el enlace; probar en el navegador |
| EC-11 | E | "El vídeo se encontró, pero no tiene formatos descargables compatibles." | `formats` vacío o sin contenedores aceptados | Probar con otro vídeo u otro enlace |
| EC-12 | W | "El análisis está tardando demasiado y se detuvo." | timeout del proceso de análisis | Reintentar |
| EC-13 | E | "El motor de descarga (yt-dlp) no está disponible." | diagnóstico negativo | Abrir Diagnóstico y seguir la guía |

## 3. Descarga

| Código | Severidad | Mensaje | Causa raíz | Acción |
|---|---|---|---|---|
| EC-20 | E | "La descarga falló. Comprueba tu conexión a internet." | error de red del motor (REA F6.4) | Reintentar cuando haya conexión |
| EC-21 | E | "El sitio no permite descargar este contenido o requiere acceso restringido." | error de acceso/privacidad del motor | Verificar autorización del contenido |
| EC-22 | E | "El vídeo ya no está disponible (fue eliminado o es privado)." | no existe / privado | — |
| EC-23 | E | "El motor de descarga devolvió un error inesperado." | `ERROR:` no clasificado del motor (REA §4.4) | Reintentar; si persiste, ver detalle técnico |
| EC-24 | E | "No hay espacio suficiente en la carpeta de destino." | disco lleno (estimación vs libre) | Liberar espacio o cambiar destino |
| EC-25 | E | "No se puede escribir en la carpeta de destino." | permisos / destino inválido | Elegir otra carpeta |
| EC-26 | W | "El destino elegido no existe." | carpeta eliminada tras elegir (REA F3.3) | Volver a elegir destino |

## 4. Conversión local

| Código | Severidad | Mensaje | Causa raíz | Acción |
|---|---|---|---|---|
| EC-30 | E | "El archivo seleccionado no es un archivo multimedia válido." | FFprobe no lo reconoce | Verificar el archivo |
| EC-31 | E | "El archivo usa un códec no compatible con la conversión solicitada." | códec ausente en FFmpeg | Elegir otro formato destino |
| EC-32 | E | "La conversión falló por un error interno de FFmpeg." | exit != 0 sin caso conocido | Ver detalle técnico; reportar si persiste |
| EC-33 | W | "La conversión se detuvo." | cancelación | — |
| EC-34 | E | "No se generó el archivo de salida." | exit 0 pero salida inexistente/vacía (AGENTS.md §9) | Reintentar; reportar |
| EC-35 | E | "FFmpeg/FFprobe no están disponibles." | diagnóstico negativo | Abrir Diagnóstico |

## 5. Sistema / persistencia / interno

| Código | Severidad | Mensaje | Causa raíz | Acción |
|---|---|---|---|---|
| EC-40 | W | "No se pudo guardar el historial; se continuará sin registrar esta operación." | fallo de escritura en SQLite (disco/permisos) | Verificar permisos de la carpeta de datos |
| EC-41 | W | "Los datos locales estaban dañados y se restauraron o reiniciaron. No se perdió ninguna descarga ya completada." | `integrity_check` de SQLite fallida y sin backup válido | — |
| EC-42 | C | "La aplicación detectó un problema interno y debe reiniciarse." | error no recuperable de la capa Rust | Reiniciar; detalle técnico en log |
| EC-43 | W | "No se pudo abrir la carpeta/archivo." | `open` del SO fallido | Abrir manualmente |

## 6. Mapeo de errores del motor (fuente de verdad para EC-20/21/22/23)

El parser de stderr de yt-dlp clasifica (patrón REA §4.4 — regex de `ERROR:` — ampliado):
- patrones de red/DNS/timeout → EC-20; `login`/`private`/`authentication`/`age`/`copyright` → EC-21; `not exist`/`removed`/`unavailable`/`404` → EC-22; cualquier otro → EC-23.
- Toda línea de error se guarda (truncada, sin secretos) como detalle técnico del error de la tarea.

## 7. Estrategia de presentación

- Errores de tarea (E) → se muestran **en la fila de la tarea** (estado `failed` + mensaje) y al detalle; no modales que bloqueen la cola (diferencia respecto a la referencia, que usa modales — REA §6 errorHandler).
- Avisos de dependencia (W/C de §5) → vista Diagnóstico + banner no bloqueante en la vista afectada (patrón REA: degradar combos — REA B1).
- Críticos (C) → diálogo con reinicio controlado y guardado de estado previo.
- Todos los mensajes provienen del i18n (`i18n/errors.ts` + equivalente Rust), nunca literales dispersos.

## 8. Criterios de aceptación transversales

- CAT-1: cada código tiene test unitario de su clasificación (motor → código) con fixtures de salida de error.
- CAT-2: ningún error llega a la UI sin código del catálogo; el tipo Rust `TaskError` lo hace inviable por construcción.
- CAT-3: los detalles técnicos son opcionales, truncados y sin secretos (test de redacción).
