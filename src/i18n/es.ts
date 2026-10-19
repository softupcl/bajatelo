/**
 * Cadenas de interfaz en español (RF-10, AGENTS.md §10).
 * Centralizadas: ninguna vista debe contener literales de usuario.
 */
export const es = {
  app: {
    title: 'REA',
    tagline: 'Descargas y conversión multimedia',
  },
  nav: {
    downloads: 'Descargas',
    conversion: 'Convertir',
    history: 'Historial',
    settings: 'Configuración',
    about: 'Acerca de',
  },
  common: {
    empty: 'Nada por aquí todavía',
    action: 'Acción',
    cancel: 'Cancelar',
    retry: 'Reintentar',
    close: 'Cerrar',
    open: 'Abrir',
  },
  downloads: {
    title: 'Descargas',
    intro: 'Pega la URL de un vídeo para analizarla y elegir formato y calidad.',
    urlLabel: 'URL del vídeo',
    urlPlaceholder: 'https://…',
    analyze: 'Analizar',
    queueTitle: 'Descargas actuales',
    notIntegrated:
      'El análisis y la descarga todavía no están integrados (bloque 3 del backlog). Esta vista muestra el estado real: sin tareas.',
    emptyQueue: 'Aún no hay descargas',
    emptyQueueHint:
      'Cuando añadas una URL y empieces una descarga, verás aquí su progreso, velocidad y tiempo estimado.',
  },
  conversion: {
    title: 'Convertir archivo local',
    intro: 'Convierte archivos multimedia de tu equipo con FFmpeg (MP4 o MP3).',
    pickFile: 'Elegir archivo…',
    notIntegrated:
      'La conversión local todavía no está integrada (bloque 4 del backlog). Esta vista muestra el estado real: sin operaciones.',
    empty: 'Aún no hay conversiones',
    emptyHint: 'Cuando conviertas un archivo, verás aquí el resultado y su historial.',
  },
  history: {
    title: 'Historial',
    intro: 'Registro local de descargas y conversiones terminadas.',
    empty: 'El historial está vacío',
    emptyHint:
      'Cada operación terminada (completada, fallida o cancelada) aparecerá aquí. Persistencia local pendiente (V-03).',
  },
  settings: {
    title: 'Configuración',
    diagnosticsTitle: 'Diagnóstico de dependencias',
    diagnosticsIntro:
      'Herramientas externas necesarias para descargar y convertir. Comprueba que estén disponibles.',
    check: 'Comprobar de nuevo',
    checking: 'Comprobando…',
    toolOk: 'Disponible',
    toolMissing: 'No encontrada',
    toolIncompatible: 'Incompatible',
    tools: {
      ytdlp: 'yt-dlp (motor de descarga)',
      ffmpeg: 'FFmpeg (conversión)',
      ffprobe: 'FFprobe (inspección)',
    },
    prefsTitle: 'Preferencias',
    prefsPending:
      'La persistencia de preferencias llega con V-03. Los valores mostrados son los predeterminados.',
    prefs: {
      defaultDestination: 'Carpeta de destino por defecto',
      conflictPolicy: 'Cuando el archivo de destino ya exista',
      conflictAsk: 'Preguntar qué hacer',
      conflictNumber: 'Numerar automáticamente (nombre-1, nombre-2…)',
      conflictCancel: 'Cancelar la operación',
      rememberQuality: 'Recordar la última calidad elegida',
      rememberFormat: 'Recordar el último formato elegido',
      removeFinished: 'Retirar de la cola las descargas terminadas',
    },
  },
  about: {
    title: 'Acerca de REA',
    version: 'Versión',
    description:
      'REA es una aplicación de escritorio independiente para analizar URLs de vídeo, descargar contenido autorizado y convertir archivos multimedia locales.',
    creditsTitle: 'Software de terceros',
    credits:
      'REA se apoya en el trabajo de los proyectos yt-dlp, FFmpeg, Tauri y Vue. Gracias a sus comunidades.',
    licenseTitle: 'Licencia',
    license:
      'Aplicación propia. Revisa la licencia de cada herramienta de terceros antes de redistribuir binarios.',
  },
  diagnostics: {
    statusTitle: 'Estado',
    versionTitle: 'Versión',
    pathTitle: 'Ruta',
  },
} as const

export type Es = typeof es
