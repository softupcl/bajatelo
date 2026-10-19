import type { ErrorCode } from '../types/contracts'

/**
 * Textos del catálogo de errores (docs/spec/error-catalog.md) en español.
 * Todo TaskError que llega de Rust se traduce aquí — nunca texto crudo (AGENTS.md §10).
 */
const ERROR_TEXTS: Record<ErrorCode, string> = {
  'EC-01': 'Introduce una URL para continuar.',
  'EC-02': 'La URL no es válida. Debe comenzar por http:// o https://',
  'EC-03': 'La URL no parece dirigirse a un sitio web válido.',
  'EC-10':
    'No se pudo analizar el vídeo. Es posible que el enlace sea incorrecto o que el sitio no sea compatible.',
  'EC-11': 'El vídeo se encontró, pero no tiene formatos descargables compatibles.',
  'EC-12': 'El análisis está tardando demasiado y se detuvo.',
  'EC-13': 'El motor de descarga (yt-dlp) no está disponible. Abre Configuración → Diagnóstico.',
  'EC-20': 'La descarga falló. Comprueba tu conexión a internet.',
  'EC-21': 'El sitio no permite descargar este contenido o requiere acceso restringido.',
  'EC-22': 'El vídeo ya no está disponible (fue eliminado o es privado).',
  'EC-23': 'El motor de descarga devolvió un error inesperado.',
  'EC-24': 'No hay espacio suficiente en la carpeta de destino.',
  'EC-25': 'No se puede escribir en la carpeta de destino.',
  'EC-26': 'El destino elegido no existe.',
  'EC-30': 'El archivo seleccionado no es un archivo multimedia válido.',
  'EC-31': 'El archivo usa un códec no compatible con la conversión solicitada.',
  'EC-32': 'La conversión falló por un error interno de FFmpeg.',
  'EC-33': 'La conversión se detuvo.',
  'EC-34': 'No se generó el archivo de salida.',
  'EC-35': 'FFmpeg/FFprobe no están disponibles. Abre Configuración → Diagnóstico.',
  'EC-40': 'No se pudo guardar el historial; se continuará sin registrar esta operación.',
  'EC-41': 'Los datos locales estaban dañados y se restauraron o reiniciaron.',
  'EC-42': 'La aplicación detectó un problema interno y debe reiniciarse.',
  'EC-43': 'No se pudo abrir la carpeta o el archivo.',
}

export function errorText(code: ErrorCode): string {
  return ERROR_TEXTS[code] ?? 'Error desconocido.'
}

/** Normaliza un rechazo de invoke a TaskError cuando es posible (CAT-2). */
export function toTaskError(e: unknown): { code: ErrorCode; detail?: string } | null {
  if (
    typeof e === 'object' &&
    e !== null &&
    'code' in e &&
    typeof (e as { code: unknown }).code === 'string'
  ) {
    const candidate = e as { code: string; detail?: string }
    if (candidate.code in ERROR_TEXTS)
      return { code: candidate.code as ErrorCode, detail: candidate.detail }
  }
  return null
}
