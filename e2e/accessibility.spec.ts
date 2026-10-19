import { test, expect } from '@playwright/test'

/**
 * R-30 / E1: accesibilidad, navegación por teclado y estados de las 5 vistas.
 * El backend queda simulado vía 'rea-e2e=1' (doble del puente: e2e-fake.ts).
 */
test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => localStorage.setItem('rea-e2e', '1'))
  await page.goto('/')
})

test('las cinco vistas son navegables con aria-current correcto', async ({ page }) => {
  const nav = page.locator('.app-nav')
  await expect(nav).toHaveAttribute('aria-label', 'Navegación principal')

  const pairs: Array<[string, RegExp]> = [
    ['Descargas', /Descargas/],
    ['Convertir', /Convertir archivo local/],
    ['Historial', /Historial/],
    ['Configuración', /Configuración/],
    ['Acerca de', /Acerca de REA/],
  ]
  for (const [label, heading] of pairs) {
    await page.getByRole('button', { name: label }).click()
    await expect(page.getByRole('heading', { level: 1 })).toHaveText(heading)
    const active = page.getByRole('button', { name: label })
    await expect(active).toHaveAttribute('aria-current', 'page')
  }
})

test('el campo URL tiene etiqueta y lugar de análisis accesibles', async ({ page }) => {
  const input = page.locator('#video-url')
  await expect(input).toBeVisible()
  await expect(page.locator('label[for="video-url"]')).toHaveText(/URL del vídeo/)
  // D-06: el análisis es explícito con el botón.
  const button = page.getByRole('button', { name: 'Analizar', exact: true })
  await expect(button).toBeDisabled() // sin URL: deshabilitado honesto
  await input.fill('https://e2e.example.com/video')
  await expect(button).toBeEnabled()
})

test('con el doble del puente el análisis muestra la ficha y las calidades del contrato', async ({
  page,
}) => {
  await page.locator('#video-url').fill('https://e2e.example.com/video')
  await page.getByRole('button', { name: 'Analizar', exact: true }).click()
  const card = page.locator('.media-card')
  await expect(card.getByRole('heading', { level: 2 })).toHaveText(/Vídeo demo/)
  // Calidades honestas derivadas del fixture (REA B2): 1080p (HD) incluida.
  await expect(card.locator('select').nth(1).locator('option')).toHaveCount(3)
  await expect(card.locator('select').nth(1).locator('option').first()).toContainText('1080p (HD)')
  // Honesto: cola de descargas aún no integrada (D-12 pendiente de red real).
  await expect(card.locator('role=note')).toContainText(/D-12/)
})

test('el historial muestra su estado vacío anunciado a lectores de pantalla', async ({ page }) => {
  await page.getByRole('button', { name: 'Historial' }).click()
  await expect(page.getByRole('status').filter({ hasText: 'El historial está vacío' })).toBeVisible()
})

test('el diagnóstico en modo E2E lista la herramientas använ con estados', async ({ page }) => {
  await page.getByRole('button', { name: 'Configuración' }).click()
  const table = page.locator('.diag-table')
  await expect(table).toBeVisible()
  await expect(table.locator('tbody tr')).toHaveCount(3)
  await expect(table.getByText('2026.e2e')).toBeVisible()
})

test('el orden de foco permite recorrer la navegación con teclado', async ({ page }) => {
  // Primera vista activa por defecto; Tab desde el body llega a los botones de navegación.
  await page.keyboard.press('Tab')
  const focused = await page.evaluate(() => document.activeElement?.textContent ?? '')
  expect(focused?.trim()).toBe('Descargas')
  await page.keyboard.press('ArrowDown') // no obligatorio; cambia a reforzar con Enter sobre Convertir
  await page.getByRole('button', { name: 'Convertir', exact: true }).focus()
  await page.keyboard.press('Enter')
  await expect(page.getByRole('heading', { level: 1 })).toHaveText(/Convertir archivo local/)
})
