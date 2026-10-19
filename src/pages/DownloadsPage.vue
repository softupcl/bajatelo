<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { t } from '../i18n'
import { getBridge } from '../services/bridge'
import { errorText, toTaskError } from '../services/errors'
import { formatBytes, formatDuration } from '../services/format'
import EmptyState from '../components/EmptyState.vue'
import type {
  AnalysisOutcome,
  MediaInfo,
  OutputFormat,
  QueueTask,
  TaskError,
  TaskState,
} from '../types/contracts'
import { AUDIO_BITRATES, type AudioBitrate } from '../types/contracts'

const url = ref('')
const analyzing = ref(false)
const info = ref<MediaInfo | null>(null)
/** Id de la tarea creada por el análisis (vive en la cola del gestor, D-12). */
const analyzedTaskId = ref<string | null>(null)
const error = ref<string | null>(null)
const errorDetail = ref<string | null>(null)
const selectedQualityId = ref<string | null>(null)
const selectedFormat = ref<OutputFormat>('mp4')
/** Tasa de bits de audio para MP3/AAC (kbps); aplica a todos los formatos salvo Original. */
const selectedAudioBitrate = ref<AudioBitrate>(128)

const tasks = ref<QueueTask[]>([])
const starting = ref(false)

const canAnalyze = computed(() => url.value.trim().length > 0 && !analyzing.value)
const canStart = computed(
  () => info.value !== null && analyzedTaskId.value !== null && !starting.value,
)

async function analyze(): Promise<void> {
  if (!canAnalyze.value) return
  analyzing.value = true
  error.value = null
  errorDetail.value = null
  info.value = null
  analyzedTaskId.value = null
  try {
    // El comando devuelve AnalysisOutcome: la tarea ya está en la cola (D-12).
    const outcome: AnalysisOutcome = await getBridge().analyzeUrl(url.value)
    if (outcome.error || !outcome.media) {
      const code = outcome.error?.code ?? 'EC-10'
      error.value = errorText(code)
      errorDetail.value = outcome.error?.detail ?? null
      return
    }
    info.value = outcome.media
    analyzedTaskId.value = outcome.taskId
    selectedQualityId.value = outcome.media.qualities[0]?.id ?? null
  } catch (e) {
    const taskError: TaskError | null = toTaskError(e)
    error.value = taskError ? errorText(taskError.code) : errorText('EC-23')
    errorDetail.value = taskError?.detail ?? null
  } finally {
    analyzing.value = false
  }
}

async function startDownload(): Promise<void> {
  const infoCard = info.value
  const taskId = analyzedTaskId.value
  if (!infoCard || !taskId || starting.value) return
  starting.value = true
  error.value = null
  errorDetail.value = null
  try {
    const extension = selectedFormat.value === 'original' ? '' : `.${selectedFormat.value}`
    const proposed = await getBridge().pickSaveDestination(`${titleSafe()}${extension}`)
    if (!proposed) return
    const policy = (await getBridge().getPreferences()).conflictPolicy
    await getBridge().startDownload(
      taskId,
      selectedQualityId.value ?? '',
      selectedFormat.value,
      proposed,
      policy,
      selectedFormat.value === 'mp3' ? String(selectedAudioBitrate.value) : null,
    )
    // REA F4.5: el campo se limpia y la vista vuelve al estado inicial;
    // el progreso vive en la cola de abajo.
    url.value = ''
    info.value = null
    analyzedTaskId.value = null
    await refreshQueue()
  } catch (e) {
    const taskError = toTaskError(e)
    error.value = taskError ? errorText(taskError.code) : errorText('EC-23')
    errorDetail.value = taskError?.detail ?? null
  } finally {
    starting.value = false
  }
}

function titleSafe(): string {
  const raw = info.value?.title?.trim()
  return raw && raw.length > 0 ? raw : 'descarga'
}

/* Cola viva (RF-04/RF-05): snapshot al montar + eventos task://*. */

async function refreshQueue(): Promise<void> {
  tasks.value = await getBridge().getTasks()
}

let unlistenState: (() => void) | null = null
let unlistenProgress: (() => void) | null = null

async function subscribeQueue(): Promise<void> {
  try {
    unlistenState = await listen<QueueTask>('task://state', (event) => {
      const updated = event.payload
      const exists = tasks.value.some((x) => x.id === updated.id)
      tasks.value = exists
        ? tasks.value.map((x) => (x.id === updated.id ? updated : x))
        : [updated, ...tasks.value]
    })
    unlistenProgress = await listen<{ taskId: string; progress: QueueTask['progress'] }>(
      'task://progress',
      (event) => {
        const { taskId, progress } = event.payload
        tasks.value = tasks.value.map((x) =>
          x.id === taskId ? { ...x, progress: progress ?? x.progress } : x,
        )
      },
    )
  } catch {
    // Modo E2E en navegador sin Tauri: la cola se consulta con getTasks().
  }
}

type RowAction = 'pauseTask' | 'resumeTask' | 'cancelTask' | 'retryTask' | 'openItem'

async function rowAction(taskId: string, action: RowAction): Promise<void> {
  const bridge = getBridge()
  const task = tasks.value.find((x) => x.id === taskId)
  try {
    if (action === 'pauseTask') await bridge.pauseTask(taskId)
    if (action === 'resumeTask') await bridge.resumeTask(taskId)
    if (action === 'cancelTask') await bridge.cancelTask(taskId)
    if (action === 'retryTask') await bridge.retryTask(taskId)
    if (action === 'openItem') await bridge.openItem(task?.destination ?? '', true)
  } catch (e) {
    const taskError = toTaskError(e)
    if (taskError) {
      error.value = errorText(taskError.code)
      errorDetail.value = taskError.detail ?? null
    }
  }
}

/* Presentación de progreso y estados (REA F6.1: etiquetas honestas). */

function stateLabel(state: TaskState): string {
  const d = t('downloads')
  const labels: Record<TaskState, string> = {
    queued: d.stateQueued,
    inspecting: d.stateInspecting,
    ready: d.stateReady,
    downloading: d.stateDownloading,
    pausing: d.statePausing,
    paused: d.statePaused,
    converting: d.stateConverting,
    completed: d.stateCompleted,
    failed: d.stateFailed,
    cancelled: d.stateCancelled,
  }
  return labels[state]
}

function progressText(task: QueueTask): string {
  const d = t('downloads')
  const p = task.progress
  if (task.state !== 'downloading' && task.state !== 'converting') return stateLabel(task.state)
  if (!p || p.percent == null) return d.stateStarting
  const parts: string[] = []
  const percent = `${p.percent.toFixed(1).replace('.', ',')} %`
  if (p.bytesDownloaded != null && p.bytesTotal != null) {
    parts.push(`${percent} (${formatBytes(p.bytesDownloaded)} / ${formatBytes(p.bytesTotal)})`)
  } else {
    parts.push(percent)
  }
  if (p.speedBps != null) parts.push(`${formatBytes(p.speedBps)}/s`)
  if (p.etaSeconds != null) parts.push(`${d.etaLabel} ${formatDuration(p.etaSeconds)}`)
  return parts.join(' · ')
}

const canPause = (task: QueueTask): boolean => task.state === 'downloading'
const canResume = (task: QueueTask): boolean => task.state === 'paused'
const canCancel = (task: QueueTask): boolean =>
  ['queued', 'inspecting', 'ready', 'downloading', 'pausing', 'paused', 'converting'].includes(
    task.state,
  )
const canRetry = (task: QueueTask): boolean =>
  (task.state === 'failed' || task.state === 'cancelled') && task.qualities.length > 0
const canOpen = (task: QueueTask): boolean => task.state === 'completed' && !!task.destination

const FORMAT_LABELS: Record<OutputFormat, string> = {
  mp4: 'MP4 (H.264/AAC)',
  mp3: 'MP3 (solo audio)',
  original: 'Original (sin reconvertir)',
}

onMounted(async () => {
  await subscribeQueue()
  await refreshQueue()
})

onBeforeUnmount(() => {
  unlistenState?.()
  unlistenState = null
  unlistenProgress?.()
  unlistenProgress = null
})
</script>

<template>
  <section aria-labelledby="downloads-title">
    <h1 id="downloads-title">{{ t('downloads').title }}</h1>
    <p class="lead">{{ t('downloads').intro }}</p>

    <div class="url-row">
      <label for="video-url" class="field-label">{{ t('downloads').urlLabel }}</label>
      <div class="url-controls">
        <input
          id="video-url"
          v-model="url"
          type="url"
          inputmode="url"
          autocomplete="off"
          :placeholder="t('downloads').urlPlaceholder"
          :disabled="analyzing"
          @keyup.enter="analyze"
        />
        <button type="button" :disabled="!canAnalyze" @click="analyze">
          {{ analyzing ? t('analysis').analyzing : t('downloads').analyze }}
        </button>
      </div>
      <p v-if="error" role="alert" class="error-text">
        {{ error }}
        <button v-if="errorDetail" type="button" class="detail-toggle" @click="errorDetail = null">
          {{ errorDetail }}
        </button>
      </p>
      <p v-if="analyzing" role="status" aria-live="polite" class="lead">
        {{ t('analysis').loading }}
      </p>
    </div>

    <div v-if="info" class="media-card" aria-live="polite">
      <h2>{{ info.title ?? t('analysis').unknownTitle }}</h2>
      <p class="lead">
        <span v-if="info.portal">{{ info.portal }} · </span>
        <span v-if="info.durationSec">{{ formatDuration(info.durationSec) }} · </span>
        <span>{{ info.qualities.length }} {{ t('analysis').qualitiesAvailable }}</span>
      </p>

      <div class="pickers">
        <label>
          <span class="field-label">{{ t('analysis').format }}</span>
          <select v-model="selectedFormat">
            <option v-for="(label, value) in FORMAT_LABELS" :key="value" :value="value">
              {{ label }}
            </option>
          </select>
        </label>
        <label v-if="selectedFormat !== 'mp3'">
          <span class="field-label">{{ t('analysis').quality }}</span>
          <select v-model="selectedQualityId">
            <option v-for="q in info.qualities" :key="q.id" :value="q.id">
              {{ q.label
              }}<template v-if="q.estimatedBytes"> · {{ formatBytes(q.estimatedBytes) }}</template>
            </option>
          </select>
        </label>
        <label v-if="selectedFormat === 'mp3'">
          <span class="field-label">{{ t('downloads').audioBitrate }}</span>
          <select v-model.number="selectedAudioBitrate">
            <option v-for="kbps in AUDIO_BITRATES" :key="kbps" :value="kbps">
              {{ kbps }} kbps
            </option>
          </select>
        </label>
      </div>

      <button type="button" :disabled="!canStart" @click="startDownload">
        {{ starting ? t('downloads').starting : t('downloads').start }}
      </button>
    </div>

    <h2>{{ t('downloads').queueTitle }}</h2>
    <EmptyState
      v-if="tasks.length === 0"
      :title="t('downloads').emptyQueue"
      :description="t('downloads').emptyQueueHint"
    />
    <!-- REA C4: la más reciente arriba; acciones por estado (REA F5.1). -->
    <ul v-else class="task-list" role="list">
      <li v-for="task in tasks" :key="task.id" class="task-row" role="listitem">
        <div class="task-main">
          <p class="task-title">{{ task.title ?? task.sourceUrl ?? '—' }}</p>
          <p class="task-meta">
            <span>{{ task.portal ?? '—' }}</span>
            <template v-if="task.selectedQualityId">
              ·
              <span>{{
                task.qualities.find((q) => q.id === task.selectedQualityId)?.label ?? '—'
              }}</span>
            </template>
            <template v-if="task.selectedFormat">
              · <span>{{ FORMAT_LABELS[task.selectedFormat] }}</span>
            </template>
          </p>
          <p class="task-progress" :aria-live="task.state === 'downloading' ? 'polite' : undefined">
            {{ progressText(task) }}
          </p>
          <p v-if="task.error" role="alert" class="task-error">
            {{ errorText(task.error.code) }}
          </p>
        </div>
        <div class="task-actions" role="group">
          <button v-if="canPause(task)" type="button" @click="rowAction(task.id, 'pauseTask')">
            {{ t('downloads').pause }}
          </button>
          <button v-if="canResume(task)" type="button" @click="rowAction(task.id, 'resumeTask')">
            {{ t('downloads').resume }}
          </button>
          <button
            v-if="canCancel(task)"
            type="button"
            class="stop"
            @click="rowAction(task.id, 'cancelTask')"
          >
            {{ t('downloads').stop }}
          </button>
          <button v-if="canRetry(task)" type="button" @click="rowAction(task.id, 'retryTask')">
            {{ t('downloads').retry }}
          </button>
          <button v-if="canOpen(task)" type="button" @click="rowAction(task.id, 'openItem')">
            {{ t('downloads').openFolder }}
          </button>
        </div>
      </li>
    </ul>
  </section>
</template>

<style scoped>
.url-row {
  margin: 12px 0 24px;
}
.field-label {
  display: block;
  font-weight: 600;
  margin-bottom: 6px;
}
.url-controls {
  display: flex;
  gap: 10px;
}
.url-controls input {
  flex: 1;
}
.media-card {
  border: 1px solid var(--border);
  border-radius: 12px;
  background: var(--surface);
  padding: 16px 20px;
  margin-bottom: 8px;
}
.media-card h2 {
  margin-top: 0;
}
.pickers {
  display: flex;
  gap: 16px;
  flex-wrap: wrap;
  margin: 12px 0;
}
.pickers label {
  display: flex;
  flex-direction: column;
  min-width: 220px;
}
.error-text {
  color: #a12631;
  margin-top: 8px;
}
.detail-toggle {
  background: none;
  border: none;
  color: inherit;
  text-decoration: underline;
  font-size: 0.85rem;
  padding: 0;
}
.task-list {
  list-style: none;
  padding: 0;
  margin: 12px 0 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.task-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--surface);
  padding: 12px 16px;
  flex-wrap: wrap;
}
.task-title {
  font-weight: 600;
  margin: 0;
}
.task-meta {
  color: var(--text-muted);
  font-size: 0.9rem;
  margin: 2px 0;
}
.task-progress {
  font-variant-numeric: tabular-nums;
  margin: 2px 0;
}
.task-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}
.task-actions .stop {
  background: transparent;
  color: #a12631;
  border: 1px solid currentColor;
}
</style>
