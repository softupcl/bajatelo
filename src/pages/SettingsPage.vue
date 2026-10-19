<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { t } from '../i18n'
import { getBridge } from '../services/bridge'
import StatusBadge from '../components/StatusBadge.vue'
import type { ConflictPolicy, Preferences, ToolDiagnostics, ToolKind } from '../types/contracts'

/* Diagnóstico (RF-08) */
const diagnostics = ref<ToolDiagnostics[]>([])
const loading = ref(false)
const error = ref<string | null>(null)

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
    </form>
  </section>
</template>

<style scoped>
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
