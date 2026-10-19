# REA — 06. Preguntas abiertas y pendientes de verificación

> Estado: BORRADOR FASE 1 · Fecha: 2026-10-19
> Cada pregunta indica cómo podría resolverse y qué la bloquea. Los estados siguen AGENTS.md §4.2: **[Unknown]** = sin verificar; **[I]** = inferido.

## Q1. Aplicación de referencia en ejecución — sin observación dinámica

- **Pregunta:** ¿Cómo se comporta la app real en pantalla ante casos límite (URL inválida escrita a mano, pérdida de red a mitad de descarga, cancelación durante la conversión)?
- **Estado:** [Unknown] — no hay ClipGrab instalado en esta máquina (detección negativa verificada).
- **Cómo resolverlo:** que el usuario instale ClipGrab 3.9.15 (Windows/macOS) o 3.9.14 (Linux), o aporte capturas/grabaciones; entonces ejecutar REA dinámico y comparar con los flujos de `03-user-flows.md`.
- **Impacto si queda sin resolver:** los flujos F1–F8 quedan respaldados por fuente + documentación, pero sin confirmación visual de tiempos, animaciones y textos de error del binario.

## Q2. Diferencias 3.9.14 vs 3.9.15 vs 4.0.0-beta2

- **Pregunta:** ¿Qué cambió entre el fuente analizado (3.9.14) y los binarios actuales (3.9.15 / 4.0.0-beta2)? ¿La 4.0 usa Qt6 u otra UI?
- **Estado:** [Unknown]. La web oficial publica 3.9.15 y una beta 4.0 para macOS, pero solo enlaza fuente 3.9.14; no se localizó el repositorio de desarrollo (`clipgrabapp/clipgrab` → 404) y la búsqueda web no estuvo disponible en esta sesión.
- **Cómo resolverlo:** reintentar búsqueda del repositorio activo; comparar changelogs de `https://clipgrab.org/update/{win|mac}/` (el canal que la propia app consulta) o notas de la beta.
- **Impacto:** bajo para el MVP del clon (los patrones observados son estables en la serie 3.9.x [I]).

## Q3. Pausa/reanudación en la práctica

- **Pregunta:** ¿La reanudación continúa realmente los `.part` o reinicia? ¿yt-dlp moderno mantiene compatibilidad con la plantilla `-o` usada?
- **Estado:** mecanismo [V] (relanza `download()` con la misma plantilla); resultado dinámico [I] — depende de `--continue` implícito de yt-dlp y del no uso de `--no-part` (la app no pasa `--no-part`).
- **Cómo resolverlo:** prueba controlada con la app o con yt-dlp directo (cuando esté instalado en el entorno).

## Q4. Límite práctico de descargas simultáneas

- **Pregunta:** ¿Cuántas descargas en paralelo maneja bien? ¿Hay cola real o todo corre a la vez?
- **Estado:** el código no limita [V]; el comportamiento con N alto [Unknown].
- **Cómo resolverlo:** observación dinámica (Q1) o prueba de carga propia del clon.

## Q5. Comportamiento con URLs de playlist puras

- **Pregunta:** ¿Qué muestra la UI al pegar la URL de una playlist completa (no un vídeo con `&list=`)?
- **Estado:** [I] con `--no-playlist` yt-dlp devolverá JSON de playlist con `entries` → estado `unfetched`/"Waiting" sin calidades; el resultado visible exacto [Unknown].
- **Cómo resolverlo:** prueba dinámica (Q1) con una playlist pública de prueba.

## Q6. Entradas de la pestaña Settings en plataformas no-macOS

- **Pregunta:** ¿Qué sub-pestaña elimina exactamente `removeTab(2)` en Windows/Linux (¿"Clipboard"?, ¿"Conversion"?) y qué efecto tiene en la numeración?
- **Estado:** código [V] (elimina índice 2 del QTabWidget General al construir la ventana); efecto por plataforma [Unknown].
- **Cómo resolverlo:** observación dinámica en Windows/Linux (Q1). En el clon este `fixme` es irrelevante (se diseñará la propia estructura de ajustes).

## Q7. Acceso a vídeos privados de Facebook

- **Pregunta:** ¿Qué credenciales usa ClipGrab para "descargar vídeos privados de Facebook" (afirmación de la web) y qué implicaciones tiene?
- **Estado:** afirmación oficial [V] (`WEB:/faqs/supported-sites`); mecanismo [Unknown] — en 3.9.14 existe `login_dialog.ui` y un ajuste oculto "Remember logins", sin conectarse a la lógica principal.
- **Decisión para el clon:** AGENTS.md §7 prohíbe soporte de descargas autenticadas sin diseño de seguridad revisado → fuera de alcance; se documenta como diferencia deliberada.

## Q8. Estado actual del desarrollo y seguridad del canal de updates

- **Pregunta:** ¿Sigue activo el proyecto? ¿Se ha corregido el canal de comandos remotos del XML de actualización?
- **Estado:** [Unknown] (sin búsqueda web en esta sesión; el canal remoto existe en 3.9.14 [V]).
- **Cómo resolverlo:** reintento de búsqueda; revisión del XML de `clipgrab.org/update/` con un fetch de solo lectura.
- **Impacto:** nulo para el clon (no replicaremos ese mecanismo; G3/G4 quedan marcados como "no replicar" en `02-feature-inventory.md`).

## Q9. Compatibilidad de los patrones de yt-dlp con la versión actual

- **Pregunta:** ¿Los argumentos y el parseo de salida usados por ClipGrab (2021+) siguen siendo válidos con el yt-dlp actual?
- **Estado:** [I] probable que sí (los patrones `--newline`, `-J`, `-f A+B`, salida `[download]` son estables), pero sin verificar con el yt-dlp actual — que además **no está instalado en este entorno**.
- **Cómo resolverlo:** al instalar yt-dlp (fase de diagnóstico del clon), validar los patrones con pruebas unitarias propias.

## Q10. Historial: decisión de producto

- **Pregunta:** ClipGrab no tiene historial persistente; el MVP del clon sí lo exige (AGENTS.md §5.1.11). ¿Qué alcance exacto debe tener (entradas, retención, borrado)?
- **Estado:** decisión de diseño pendiente de la fase de especificación (depende de la aprobación del backlog).
- **Cómo resolverlo:** proponer modelo de datos en `docs/spec/data-model.md` en la fase siguiente.

---

## Bloqueadores actuales del entorno (resumen)

| Bloqueador | Afecta a | Mitigación |
|---|---|---|
| Sin app de referencia instalada | REA dinámico (Q1, Q4-Q7) | Usuario instala/provee capturas; o se acepta REA estático como base |
| Sin búsqueda web en la sesión | Q2, Q8 | Reintentar en otra sesión |
| yt-dlp/ffmpeg no instalados en el entorno | Q3, Q9 y cualquier prueba real futura | Instalación con aprobación del usuario (AGENTS.md: preguntar antes de cambios de dependencias) |
| cargo/rustc ausentes | Fase de scaffolding Tauri (no de esta fase) | Instalación con aprobación cuando llegue el momento |
| Repositorio sin git | Trazabilidad de cambios | Inicializar git antes de la fase 2 (solicitar aprobación) |
