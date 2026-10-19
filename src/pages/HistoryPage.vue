<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { t } from '../i18n'
import { getBridge } from '../services/bridge'
import { formatBytes } from '../services/format'
import EmptyState from '../components/EmptyState.vue'
import type { HistoryEntry } from '../types/contracts'

const entries = ref<HistoryEntry[]>([])
const loading = ref(true)
const error = ref<string | null>(null)
const openError = ref(false)

async function refresh(): Promise<void> {
  loading.value = true
  error.value = null
  try {
    entries.value = await getBridge().historyList()
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

/** REA F5.6: abrir el archivo descargado o revelar su carpeta (fallo → EC-43). */
async function openEntry(path: string, reveal: boolean): Promise<void> {
  openError.value = false
  try {
    await getBridge().openItem(path, reveal)
  } catch {
    openError.value = true
  }
}

async function removeEntry(id: string): Promise<void> {
  await getBridge().historyRemove(id)
  await refresh()
}

async function clearAll(): Promise<void> {
  const confirmed = window.confirm(t('history').confirmClear)
  if (!confirmed) return
  await getBridge().historyClear()
  await refresh()
}

function statusLabel(status: HistoryEntry['status']): string {
  const h = t('history')
  switch (status) {
    case 'completed':
      return h.statusCompleted
    case 'failed':
      return h.statusFailed
    case 'cancelled':
      return h.statusCancelled
  }
}

onMounted(refresh)
</script>

<template>
  <section aria-labelledby="history-title">
    <h1 id="history-title">{{ t('history').title }}</h1>
    <p class="lead">{{ t('history').intro }}</p>

    <p v-if="error" role="alert" class="error-text">{{ error }}</p>

    <EmptyState
      v-if="!loading && !error && entries.length === 0"
      :title="t('history').empty"
      :description="t('history').emptyHint"
    />

    <div v-if="entries.length" class="history-list" role="list">
      <article v-for="entry in entries" :key="entry.id" class="entry" role="listitem">
        <div class="entry-main">
          <p class="entry-title">{{ entry.title }}</p>
          <p class="entry-meta">
            <span>{{
              entry.kind === 'download' ? (entry.portal ?? '—') : t('conversion').title
            }}</span>
            · <span>{{ statusLabel(entry.status) }}</span>
            <template v-if="entry.quality"> · {{ entry.quality }}</template>
            <template v-if="entry.finalSizeBytes">
              · {{ formatBytes(entry.finalSizeBytes) }}</template
            >
          </p>
          <p class="entry-path mono">{{ entry.destinationPath }}</p>
        </div>
        <div class="entry-actions">
          <template v-if="entry.status === 'completed' && entry.destinationPath">
            <button type="button" @click="openEntry(entry.destinationPath, false)">
              {{ t('downloads').openFile }}
            </button>
            <button type="button" @click="openEntry(entry.destinationPath, true)">
              {{ t('downloads').openFolder }}
            </button>
          </template>
          <button type="button" class="remove" @click="removeEntry(entry.id)">
            {{ t('history').remove }}
          </button>
        </div>
      </article>
      <p v-if="openError" role="alert" class="error-text">{{ t('conversion').openFail }}</p>
      <button type="button" class="clear" @click="clearAll">{{ t('history').clear }}</button>
    </div>
  </section>
</template>

<style scoped>
.error-text {
  color: #a12631;
}
.history-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-top: 12px;
}
.entry {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--surface);
  padding: 12px 16px;
}
.entry-title {
  font-weight: 600;
  margin: 0;
}
.entry-meta {
  color: var(--text-muted);
  font-size: 0.9rem;
  margin: 2px 0;
}
.entry-path {
  font-size: 0.8rem;
  color: var(--text-muted);
  margin: 0;
  word-break: break-all;
}
.entry-actions {
  display: flex;
  gap: 8px;
  align-items: center;
}

.remove,
.clear {
  background: transparent;
  color: #a12631;
  border: 1px solid currentColor;
}
.clear {
  align-self: flex-start;
}
.mono {
  font-family: ui-monospace, Menlo, monospace;
}
</style>
