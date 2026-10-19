# Diseño — Incrustación de motores yt-dlp y FFmpeg (propuesta D-09, pendiente de aprobación)

> Fecha: 2026-10-19 · Estado: BORRADOR para decisión del usuario (revoca parcialmente la decisión D-03 "no redistribuir motores").
> Motivación: la experiencia del usuario requiere que la app funcione completa sin instalar nada a mano.

## 1 · Hechos de licencia (verificados)

| Motor | Licencia | ¿Se puede redistribuir? | Condiciones |
|---|---|---|---|
| **yt-dlp** | **Unlicense** (dominio público) | ✅ sí, sin condiciones | conservar el aviso (ya en `THIRD-PARTY-NOTICES.md`). **Ojo**: los binarios autocontenidos (`yt-dlp.exe` Windows, `yt-dlp_macos`) son PyInstaller; existen builds oficiales por arquitectura. |
| **FFmpeg/FFprobe** | **GPL v2+ o LGPL según el build** | ✅ sí | redistribuir binario GPL exige **ofrecer el código fuente correspondiente** (enlace o copia) y sus avisos; LGPL exige lo mismo + aviso pleno. El MP4 del clon recodifica con libx264 la mayoría de builds estáticos son GPL. **Importante**: la app invoca un proceso ajeno (no enlaza), por lo que la GPL del motor **no infecta** el resto del proyecto. |

Fuentes oficiales de builds estáticos (a fijar por versión + SHA-256):
- yt-dlp: releases GitHub oficiales (`yt-dlp.exe` x64, `yt-dlp_macos`, `yt-dlp` zipapp Linux).
- FFmpeg: builds estáticos comunitarios reconocidos (BtbN para win/linux, macOS por arquitectura; fijar proveedor y verificar la licencia real del build en la revisión de release).

## 2 · Opciones

### Opción A — Empotrado en el instalador (bundled)
Cada job de CI descarga el build de motor correspondiente a su SO/arq en el paso de build y lo coloca en el bundle:
- macOS: `Bajatelo.app/Contents/Resources/engines/`
- Windows/Linux: subdirectorio junto al ejecutable.
El `tool_locator` prueba **primero el motor embebido** y cae al PATH si falta (degradación vigente).

- **Pros**: fuera-de-la-caja total; sin descargas en primer uso; funciona sin red.
- **Contras**: instalador ~+90–110 MB (ffmpeg estático ~44–78 MB + yt-dlp ~12–17 MB); los extractores de sitios avanzan rápido → el motor va quedando obsoleto hasta nueva versión de la app.
- **Licencias a cumplir**: source-offer GPL de ffmpeg + SHA-256 de los binarios fijados en el workflow (rastro de integridad).

### Opción B — Descarga asistida en primer uso (patrón verificado de la referencia)
La referencia ClipGrab **ofrece descargar yt-dlp al arrancar** con barra de progreso (evidencia [V] REA 03, caso límite yt-dlp ausente). El clon replicaría: banner/asistente cuando faltá algo → descarga **pinned + SHA-256** a la carpeta de datos de la app → locator la prefiere.

- **Pros**: instalador en su tamaño actual (~3 MB); motores actualizables de forma independiente (extractores siempre al día); el usuario autoriza la descarga explícitamente.
- **Contras**: requiere red en primer uso; es más código (descargador con verificación de hash, progreso y errores de conectividad — EC-20).

### Opción C — Híbrido (recomendación del diseño)
- FFmpeg **empotrado en el build** (cambia poco: los códecs son estables).
- yt-dlp **con actualización asistida**: embebido en el instalador la primera vez y **botón de actualización del motor** en Configuración que baja el release oficial pinned + SHA-256 (extractores al día siempre).
- Administra: locator con prioridad bundle → data-dir → PATH; todo con pruebas I1/I2 y failing honestos.

## 3 · Seguridad aplicable (todo lo anterior)

- Versiones **pinned** del release oficial + **verificación SHA-256** en el build/download (log del hash; sin secretos).
- Motores en ubicaciones de app (Resources/directorio de datos), nunca ejecutables controlados por terceros al vuelo; PATH queda como fallback último.
- Sin e-vitación de accesos: los motores se usan tal como su proyecto los distribuye.
- Manifiesto de motores visible en Configuración (versión embebida/instalada + estado).

## 4 · Efectos estimados

| Opción | Esfuerzo | Riesgo (fragilidad build) | Experiencia offline primera vez |
|---|---|---|---|
| A (todo embebido) | pequeño (workflow + locator) | bajo | ✅ completa |
| B (descarga asistida) | medio (feature D-15 completa con pruebas) | bajo-medio | ❌ necesita red |
| C (híbrido) | medio | bajo | ✅ (extractores luego actualizables) |

## 5 · Decisión requerida (D-09)

**RESUELTA (2026-10-19): el usuario autorizó la opción C (híbrido).** Implementada:
- `scripts/fetch-engines.sh` (descarga fijada + verificación de hashes: SHA2-256SUMS oficial de yt-dlp; SHA256 del proveedor de FFmpeg macOS; BtbN win/linux).
- `tauri.conf.json` → `resources: ["engines/"]` (motores dentro del bundle).
- `tool_locator`: prioridad embebido → `<datos>/engines` → PATH.
- Comando `update_ytdlp` + botón "Actualizar motor" en Configuración (release oficial más reciente + SHA-256 → `<datos>/engines`).
- Workflow CI con paso de descarga de motores por plataforma.
La reversión parcial de D-03 queda registrada aquí con la fecha y el OK del usuario.
