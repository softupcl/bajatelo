import { es } from './es'

/**
 * i18n mínimo para el MVP: locale fijo "es" (RF-09/RF-10),
 * preparado para añadir más idiomas sin tocar las vistas.
 */
const locales = { es } as const
export type Locale = keyof typeof locales

let current: Locale = 'es'

export function setLocale(locale: Locale): void {
  current = locale
}

export function getLocale(): Locale {
  return current
}

/** Acceso tipado a las cadenas: t('nav.downloads'). */
export function t<S extends keyof typeof es>(section: S): (typeof es)[S] {
  return locales[current][section]
}

/** Export para tests: claves requeridas por sección. */
export const sections = Object.keys(es) as Array<keyof typeof es>
