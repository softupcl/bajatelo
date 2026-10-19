<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { t } from '../i18n'
import { getBridge } from '../services/bridge'
import { errorText, toTaskError } from '../services/errors'
import StatusBadge from '../components/StatusBadge.vue'
import type { ConflictPolicy, Preferences, ToolDiagnostics, ToolKind } from '../types/contracts'

/* Diagnóstico (RF-08) */
const diagnostics = ref<ToolDiagnostics[]>([])
const loading = ref(false)
const error = ref<string | null>(null)

/* Actualización del motor de descarga (D-09/C). */
const updatingEngine = ref(false)
const engineUpdate = ref<string | null>(null)
const engineUpdateError = ref<string | null>(null)

async function updateEngine(): Promise<void> {
  if (updatingEngine.value) return
  updatingEngine.value = true
  engineUpdate.value = null
  engineUpdateError.value = null
  try {
    const result = await getBridge().updateYtdlp()
    engineUpdate.value = t('settings').engineUpdated.replace('{version}', result.version)
    await check() // refresca el diagnóstico con el motor nuevo
  } catch (e) {
    const taskError = toTaskError(e)
    engineUpdateError.value = taskError
      ? `${errorText(taskError.code)} ${taskError.detail ?? ''}`.trim()
      : String(e)
  } finally {
    updatingEngine.value = false
  }
}

function labelFor(tool: ToolKind): string {
  return t('settings').tools[tool]
}

async function check(): Promise<void> {
  loading.value = true
  error.value = null
  try {
    diagnostics.value = await getBridge().getDiagnostics()
  } catch (e) {
    error.value = String(e)
    diagnostics.value = []
  } finally {
    loading.value = false
  }
}

onMounted(check)

/* Preferencias (RF-09 + V-03): carga y guardado inmediato. */
const prefs = ref<Preferences | null>(null)
const saving = ref(false)

onMounted(async () => {
  try {
    prefs.value = await getBridge().getPreferences()
  } catch (e) {
    error.value = String(e)
  }
})

async function update(patch: Partial<Preferences>): Promise<void> {
  if (!prefs.value || saving.value) return
  saving.value = true
  try {
    prefs.value = await getBridge().savePreferences({ ...prefs.value, ...patch })
  } finally {
    saving.value = false
  }
}

async function pickDestination(): Promise<void> {
  const dir = await getBridge().pickDirectory()
  if (dir) await update({ defaultDestination: dir })
}

/** SO para ajustar la guía instalable (sin dependencias externas). */
const OS_KEY: 'mac' | 'win' | 'linux' = (() => {
  if (typeof navigator === 'undefined') return 'mac'
  const ua = navigator.userAgent.toLowerCase()
  if (ua.includes('mac os')) return 'mac'
  if (ua.includes('windows')) return 'win'
  return 'linux'
})()

const COMANDOS: Record<ToolKind, Record<'mac' | 'win' | 'linux', string>> = {
  ytdlp: {
    mac: 'brew install yt-dlp (o: pipx install yt-dlp)',
    win: 'winget install yt-dlp (o el instalador oficial de yt-dlp)',
    linux: 'pipx install yt-dlp (o el paquete de tu distro)',
  },
  ffmpeg: {
    mac: 'brew install ffmpeg',
    win: 'winget install ffmpeg',
    linux: 'sudo apt install ffmpeg (o el paquete de tu distro)',
  },
  ffprobe: {
    mac: 'brew install ffmpeg (incluye ffprobe)',
    win: 'winget install ffmpeg (incluye ffprobe)',
    linux: 'sudo apt install ffmpeg (incluye ffprobe; o ffmpeg de tu distro)',
  },
}

const needsGuide = computed(() => diagnostics.value.some((d) => d.status !== 'ok'))

const guideSteps = computed(() =>
  diagnostics.value
    .filter((d) => d.status !== 'ok')
    .map((d) => {
      if (d.status === 'incompatible') {
        return {
          tool: d.tool,
          text: `${t('diagnostics').incompatibleHint} — ${COMANDOS[d.tool][OS_KEY]}`,
        }
      }
      return { tool: d.tool, text: COMANDOS[d.tool][OS_KEY] }
    }),
)
</script>

<template>
  <section aria-labelledby="settings-title">
    <h1 id="settings-title">{{ t('settings').title }}</h1>

    <h2>{{ t('settings').diagnosticsTitle }}</h2>
    <p class="lead">{{ t('settings').diagnosticsIntro }}</p>

    <button type="button" :disabled="loading" @click="check">
      {{ loading ? t('settings').checking : t('settings').check }}
    </button>

    <p v-if="error" role="alert" class="error-text">{{ error }}</p>

    <table v-if="diagnostics.length" class="diag-table">
      <thead>
        <tr>
          <th scope="col">Herramienta</th>
          <th scope="col">{{ t('diagnostics').statusTitle }}</th>
          <th scope="col">{{ t('diagnostics').versionTitle }}</th>
          <th scope="col">{{ t('diagnostics').pathTitle }}</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="d in diagnostics" :key="d.tool">
          <th scope="row">{{ labelFor(d.tool) }}</th>
          <td><StatusBadge :status="d.status" /></td>
          <td>{{ d.version ?? '—' }}</td>
          <td class="mono">{{ d.path ?? '—' }}</td>
        </tr>
      </tbody>
    </table>

    <!-- Asistente de diagnóstico (RF-08): qué instalar y cómo, por herramienta y SO. -->
    <div v-if="needsGuide" class="diag-guide" role="status" aria-live="polite">
      <h3>{{ t('diagnostics').guideTitle }}</h3>
      <p>{{ t('diagnostics').guideIntro }}</p>
      <ul>
        <li v-for="step in guideSteps" :key="step.tool">
          <strong>{{ labelFor(step.tool) }}:</strong> {{ step.text }}
        </li>
      </ul>
    </div>
    <p v-else-if="diagnostics.length" role="status">{{ t('diagnostics').allOk }}</p>

    <!-- Motores (D-09/C): actualización asistida de yt-dlp. -->
    <div class="engine-row">
      <div>
        <h3>{{ t('settings').engineTitle }}</h3>
        <p class="lead">{{ t('settings').engineIntro }}</p>
      </div>
      <button type="button" :disabled="updatingEngine" @click="updateEngine">
        {{ updatingEngine ? t('settings').engineUpdating : t('settings').engineUpdate }}
      </button>
    </div>
    <p v-if="engineUpdate" role="status" aria-live="polite" class="ok-text">{{ engineUpdate }}</p>
    <p v-if="engineUpdateError" role="alert" class="error-text">{{ engineUpdateError }}</p>

    <h2>{{ t('settings').prefsTitle }}</h2>
    <p v-if="prefs" class="lead" role="status" aria-live="polite">
      {{ saving ? t('settings').saving : t('settings').prefsSaved }}
    </p>

    <form v-if="prefs" class="prefs" @submit.prevent>
      <label class="pref-row">
        <span>{{ t('settings').prefs.defaultDestination }}</span>
        <span class="dest-value mono">{{ prefs.defaultDestination || '—' }}</span>
        <button type="button" @click="pickDestination">
          {{ t('settings').prefs.browse }}
        </button>
      </label>
      <label class="pref-row">
        <span>{{ t('settings').prefs.conflictPolicy }}</span>
        <select
          :value="prefs.conflictPolicy"
          @change="
            update({ conflictPolicy: ($event.target as HTMLSelectElement).value as ConflictPolicy })
          "
        >
          <option value="ask">{{ t('settings').prefs.conflictAsk }}</option>
          <option value="number">{{ t('settings').prefs.conflictNumber }}</option>
          <option value="cancel">{{ t('settings').prefs.conflictCancel }}</option>
        </select>
      </label>
      <label class="pref-row">
        <input
          type="checkbox"
          :checked="prefs.rememberQuality"
          @change="update({ rememberQuality: ($event.target as HTMLInputElement).checked })"
        />
        <span>{{ t('settings').prefs.rememberQuality }}</span>
      </label>
      <label class="pref-row">
        <input
          type="checkbox"
          :checked="prefs.rememberFormat"
          @change="update({ rememberFormat: ($event.target as HTMLInputElement).checked })"
        />
        <span>{{ t('settings').prefs.rememberFormat }}</span>
      </label>
      <label class="pref-row">
        <input
          type="checkbox"
          :checked="prefs.removeFinishedFromQueue"
          @change="update({ removeFinishedFromQueue: ($event.target as HTMLInputElement).checked })"
        />
        <span>{{ t('settings').prefs.removeFinished }}</span>
      </label>
      <label class="pref-row">
        <input
          type="checkbox"
          :checked="prefs.clipboardDetection"
          @change="update({ clipboardDetection: ($event.target as HTMLInputElement).checked })"
        />
        <span>{{ t('settings').prefs.clipboardDetection }}</span>
      </label>
    </form>
  </section>
</template>

<style scoped>
.diag-guide {
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--surface);
  padding: 14px 18px;
  margin: 14px 0 24px;
}
.diag-guide h3 {
  margin: 0 0 6px;
}
.engine-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--surface);
  padding: 14px 18px;
  margin-bottom: 10px;
  max-width: 640px;
}
.engine-row h3 {
  margin: 0 0 4px;
}
.engine-row .lead {
  margin: 0;
}
.engine-row button {
  align-self: center;
}
.ok-text {
  color: #14432a;
  font-weight: 600;
}
.diag-table {
  width: 100%;
  border-collapse: collapse;
  margin: 16px 0 32px;
}
.diag-table th,
.diag-table td {
  text-align: left;
  padding: 8px 10px;
  border-bottom: 1px solid var(--border);
}
.mono {
  font-family: ui-monospace, Menlo, monospace;
  font-size: 0.85rem;
  word-break: break-all;
}
.error-text {
  color: #a12631;
}
.prefs {
  display: flex;
  flex-direction: column;
  gap: 14px;
  max-width: 640px;
}
.pref-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
.pref-row > span:first-child {
  flex: 1;
}
.pref-row input[type='checkbox'] {
  justify-self: start;
}
.dest-value {
  flex: 1;
  text-align: right;
}
</style>
