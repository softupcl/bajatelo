<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { t } from '../i18n'
import { getBridge } from '../services/bridge'
import StatusBadge from '../components/StatusBadge.vue'
import type { ToolDiagnostics, ToolKind } from '../types/contracts'

const diagnostics = ref<ToolDiagnostics[]>([])
const loading = ref(false)
const error = ref<string | null>(null)

function labelFor(tool: ToolKind): string {
  const tools = t('settings').tools
  return tools[tool]
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
</script>

<template>
  <section aria-labelledby="settings-title">
    <h1 id="settings-title">
      {{ t('settings').title }}
    </h1>

    <h2>{{ t('settings').diagnosticsTitle }}</h2>
    <p class="lead">
      {{ t('settings').diagnosticsIntro }}
    </p>

    <button type="button" :disabled="loading" @click="check">
      {{ loading ? t('settings').checking : t('settings').check }}
    </button>

    <p v-if="error" role="alert" class="error-text">
      {{ error }}
    </p>

    <table v-if="diagnostics.length" class="diag-table">
      <thead>
        <tr>
          <th scope="col">Herramienta</th>
          <th scope="col">
            {{ t('diagnostics').statusTitle }}
          </th>
          <th scope="col">
            {{ t('diagnostics').versionTitle }}
          </th>
          <th scope="col">
            {{ t('diagnostics').pathTitle }}
          </th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="d in diagnostics" :key="d.tool">
          <th scope="row">
            {{ labelFor(d.tool) }}
          </th>
          <td>
            <StatusBadge :status="d.status" />
          </td>
          <td>{{ d.version ?? '—' }}</td>
          <td class="mono">
            {{ d.path ?? '—' }}
          </td>
        </tr>
      </tbody>
    </table>

    <h2>{{ t('settings').prefsTitle }}</h2>
    <p class="lead">
      {{ t('settings').prefsPending }}
    </p>

    <form class="prefs" @submit.prevent>
      <label class="pref-row">
        <span>{{ t('settings').prefs.defaultDestination }}</span>
        <input type="text" value="" disabled />
      </label>
      <label class="pref-row">
        <span>{{ t('settings').prefs.conflictPolicy }}</span>
        <select disabled>
          <option>{{ t('settings').prefs.conflictAsk }}</option>
          <option>{{ t('settings').prefs.conflictNumber }}</option>
          <option>{{ t('settings').prefs.conflictCancel }}</option>
        </select>
      </label>
      <label class="pref-row">
        <input type="checkbox" disabled />
        <span>{{ t('settings').prefs.rememberQuality }}</span>
      </label>
      <label class="pref-row">
        <input type="checkbox" disabled />
        <span>{{ t('settings').prefs.rememberFormat }}</span>
      </label>
      <label class="pref-row">
        <input type="checkbox" disabled />
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
  max-width: 560px;
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
</style>
