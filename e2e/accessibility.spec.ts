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
    ['Acerca de', /Acerca de Bajatelo/],
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
  // El inicio de descarga es real via start_download (fase 6): botón habilitado.
  const startButton = page.getByRole('button', { name: 'Descargar', exact: true })
  await expect(startButton).toBeVisible()
  await expect(startButton).toBeEnabled()
  // MP4: solo "Calidad" de vídeo; sin selector de audio (fase 8, con el usuario).
  const selects = card.locator('select')
  await expect(selects).toHaveCount(2) // formato + calidad (vídeo)
  await expect(card.getByText('Calidad de audio')).toHaveCount(0)
})

test('solo MP3 muestra la calidad de audio y oculta la calidad de vídeo', async ({ page }) => {
  await page.locator('#video-url').fill('https://e2e.example.com/video')
  await page.getByRole('button', { name: 'Analizar', exact: true }).click()
  const card = page.locator('.media-card')
  await card.locator('select').first().selectOption('mp3')

  // Con MP3 el motor descarga bestaudio: la calidad de vídeo no aplica.
  await expect(card.getByText('Calidad de audio (kbps)')).toHaveCount(1)
  await expect(card.getByText('1080p (HD)')).toHaveCount(0)
  await expect(card.locator('select')).toHaveCount(2) // formato + tasa de audio
  const bitrateSelect = card.locator('select').nth(1)
  await expect(bitrateSelect).toContainText('128 kbps')
  const options = bitrateSelect.locator('option')
  await expect(options).toHaveCount(6)
  await expect(options.first()).toContainText('64 kbps')
})

test('el flujo de descarga crea una fila en la cola con estado, progreso y acciones', async ({
  page,
}) => {
  await page.locator('#video-url').fill('https://e2e.example.com/video')
  await page.getByRole('button', { name: 'Analizar', exact: true }).click()
  await page.getByRole('button', { name: 'Descargar', exact: true }).click()

  const queue = page.locator('.task-list')
  const row = queue.getByRole('listitem')
  await expect(row).toHaveCount(1)
  await expect(row.locator('.task-title')).toContainText('Vídeo demo')
  // Estado y progreso honestos del doble (no se fabrican datos fuera del modo e2e).
  await expect(row.locator('.task-progress')).toContainText(/12,4 %/)
  await expect(row.locator('.task-progress')).toContainText(/KiB/)
  await expect(row.locator('.task-progress')).toContainText(/restan/)
  // Acciones por estado (REA F5): pausar y detener en `downloading`.
  await expect(row.getByRole('button', { name: 'Pausar' })).toBeVisible()
  await expect(row.getByRole('button', { name: 'Detener' })).toBeVisible()
  await expect(row.getByRole('button', { name: 'Reanudar' })).toHaveCount(0)
})

test('el historial muestra su estado vacío anunciado a lectores de pantalla', async ({ page }) => {
  await page.getByRole('button', { name: 'Historial' }).click()
  await expect(
    page.getByRole('status').filter({ hasText: 'El historial está vacío' }),
  ).toBeVisible()
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
