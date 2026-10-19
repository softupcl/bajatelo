import { defineConfig } from '@playwright/test'

/**
 * E1 del plan de pruebas (docs/spec/test-plan.md): Playwright contra el
 * frontend Vite con el backend simulado del puente (modulo e2e-fake).
 */
export default defineConfig({
  testDir: './e2e',
  timeout: 30_000,
  use: {
    baseURL: 'http://localhost:1420',
    headless: true,
  },
  webServer: {
    command: 'npm run dev',
    url: 'http://localhost:1420',
    reuseExistingServer: false,
    timeout: 60_000,
  },
  projects: [{ name: 'chromium', use: { browserName: 'chromium' } }],
})
