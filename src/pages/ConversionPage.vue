<script setup lang="ts">
import { onBeforeUnmount, ref } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { t } from '../i18n'
import { getBridge } from '../services/bridge'
import { errorText, toTaskError } from '../services/errors'
import { formatBytes, formatDuration } from '../services/format'
import EmptyState from '../components/EmptyState.vue'
import type { LocalMediaInfo } from '../types/contracts'
import { AUDIO_BITRATES, type AudioBitrate } from '../types/contracts'

/** Ficha del archivo local (FFprobe real — RF-06). */
const pickedPath = ref<string | null>(null)
const localInfo = ref<LocalMediaInfo | null>(null)
const probingFile = ref(false)
const fileError = ref<{ text: string; detail?: string } | null>(null)

/** Conversión en curso (máquina de estados real del gestor — C-20). */
const format = ref<'mp4' | 'mp3'>('mp4')
/** Tasa de bits de audio (kbps) para el MP3/AAC resultante. */
const audioBitrate = ref<AudioBitrate>(128)
const converting = ref(false)
const activeTaskId = ref<string | null>(null)
const taskState = ref<string | null>(null)
const taskError = ref<string | null>(null)
const taskDone = ref(false)
const taskStopped = ref(false)
/** Destino final confirmado por el evento `task://state` en `completed`. */
const finalPath = ref<string | null>(null)
const openError = ref(false)
let unlistenState: (() => void) | null = null

const infoLabels = t('conversion').infoLabels

type InfoKey = keyof typeof infoLabels
const INFO_ORDER: InfoKey[] = ['container', 'videoCodec', 'audioCodec', 'duration', 'size']

function infoText(key: InfoKey): string {
  const info = localInfo.value
  if (!info) return '—'
  const value: Record<InfoKey, number | null | string> = {
    container: info.container,
    videoCodec: info.videoCodec,
    audioCodec: info.audioCodec,
    duration: info.durationSec,
    size: info.sizeBytes,
  }
  const raw = value[key]
  if (raw == null) return '—'
  if (key === 'duration') return formatDuration(Number(raw))
  if (key === 'size') return formatBytes(Number(raw))
  return String(raw)
}

async function chooseAndProbe(): Promise<void> {
  fileError.value = null
  const path = await getBridge().pickMediaFile()
  if (!path) return
  probingFile.value = true
  localInfo.value = null
  pickedPath.value = null
  taskState.value = null
  taskError.value = null
  taskDone.value = false
  taskStopped.value = false
  finalPath.value = null
  openError.value = false
  try {
    localInfo.value = await getBridge().probeLocalFile(path)
    pickedPath.value = path
  } catch (e) {
    const taskError = toTaskError(e)
    fileError.value = {
      text: taskError ? errorText(taskError.code) : errorText('EC-30'),
      detail: taskError?.detail,
    }
  } finally {
    probingFile.value = false
  }
}

async function beginConversion(): Promise<void> {
  if (!pickedPath.value || converting.value) return
  taskError.value = null
  taskDone.value = false
  taskStopped.value = false
  finalPath.value = null
  openError.value = false
  const ext = format.value
  const proposed = await getBridge().pickSaveDestination(
    `${fileNameOf(pickedPath.value)}.re.${ext}`,
  )
  if (!proposed) return

  converting.value = true
  taskState.value = 'inspecting'
  try {
    const taskId = await getBridge().startConversion(
      pickedPath.value,
      format.value,
      proposed,
      'ask',
      format.value === 'mp3' ? String(audioBitrate.value) : null,
    )
    activeTaskId.value = taskId
    unlistenState?.()
    unlistenState = null
    unlistenState = await listen<{ id: string; state: string; destination: string | null }>(
      'task://state',
      (event) => {
        const payload = event.payload
        if (payload.id !== taskId) return
        taskState.value = payload.state
        if (['completed', 'failed', 'cancelled'].includes(payload.state)) {
          converting.value = false
          taskDone.value = payload.state === 'completed'
          taskStopped.value = payload.state === 'cancelled'
          finalPath.value = payload.destination
          if (payload.state === 'failed') {
            // Mensaje del catálogo; detalle técnico disponible en Historial/logs (CAT-3).
            taskError.value = errorText('EC-32')
          }
          unlistenState?.()
          unlistenState = null
        }
      },
    )
  } catch (e) {
    const normalized = toTaskError(e)
    taskError.value = errorText(normalized?.code ?? 'EC-23')
    converting.value = false
  }
}

function fileNameOf(path: string): string {
  const base =
    path
      .replace(/\.[^.]+$/, '')
      .split('/')
      .pop() ?? 'conversion'
  return base || 'conversion'
}

/** Abre el archivo convertido o revela su carpeta (req. 10; fallo → EC-43). */
async function openResult(reveal: boolean): Promise<void> {
  if (!finalPath.value) return
  openError.value = false
  try {
    await getBridge().openItem(finalPath.value, reveal)
  } catch {
    openError.value = true
  }
}

async function stopConversion(): Promise<void> {
  if (!activeTaskId.value) return
  await getBridge().cancelTask(activeTaskId.value)
  // El estado final llegará por el evento task://state (T4).
}

onBeforeUnmount(() => {
  unlistenState?.()
  unlistenState = null
})
</script>

<template>
  <section aria-labelledby="conversion-title">
    <h1 id="conversion-title">{{ t('conversion').title }}</h1>
    <p class="lead">{{ t('conversion').intro }}</p>

    <button type="button" :disabled="probingFile" @click="chooseAndProbe">
      {{ probingFile ? t('conversion').probeSpinning : t('conversion').pickFile }}
    </button>

    <p v-if="fileError" role="alert" class="error-text">
      {{ fileError.text }}
      <code v-if="fileError.detail" class="mono">{{ fileError.detail }}</code>
    </p>

    <div v-if="localInfo" class="media-card" aria-live="polite">
      <h2>{{ fileNameOf(pickedPath ?? '') }}</h2>
      <table class="info-table">
        <tbody>
          <tr v-for="key in INFO_ORDER" :key="key">
            <th scope="row">{{ infoLabels[key] }}</th>
            <td :class="{ mono: key !== 'duration' && key !== 'size' }">{{ infoText(key) }}</td>
          </tr>
        </tbody>
      </table>

      <div class="convert-row">
        <label>
          <span class="field-label">{{ t('analysis').format }}</span>
          <select v-model="format" :disabled="converting">
            <option value="mp4">MP4 (H.264/AAC)</option>
            <option value="mp3">MP3 (solo audio)</option>
          </select>
        </label>
        <label v-if="format === 'mp3'">
          <span class="field-label">{{ t('conversion').audioBitrate }}</span>
          <select v-model.number="audioBitrate" :disabled="converting">
            <option v-for="kbps in AUDIO_BITRATES" :key="kbps" :value="kbps">
              {{ kbps }} kbps
            </option>
          </select>
        </label>
        <button type="button" :disabled="converting" @click="beginConversion">
          {{
            converting
              ? t('conversion').converting
              : taskDone
                ? t('conversion').again
                : t('conversion').convert
          }}
        </button>
        <button v-if="converting" type="button" class="stop" @click="stopConversion">
          {{ t('conversion').stop }}
        </button>
      </div>

      <p v-if="taskDone" role="status" class="ok-text">{{ t('conversion').done }}</p>
      <div v-if="taskDone && finalPath" class="convert-row" role="group">
        <button type="button" @click="openResult(false)">{{ t('conversion').openFile }}</button>
        <button type="button" @click="openResult(true)">{{ t('conversion').openFolder }}</button>
        <p v-if="openError" role="alert" class="error-text">{{ t('conversion').openFail }}</p>
      </div>
      <p v-if="taskStopped" role="status" aria-live="polite" class="lead">
        {{ t('conversion').stopped }}
      </p>
      <p v-if="taskError" role="alert" class="error-text">{{ taskError }}</p>
      <p v-if="converting" role="status" aria-live="polite" class="lead">
        {{ t('conversion').convertingNote }}
      </p>
    </div>

    <EmptyState
      v-if="!localInfo"
      :title="t('conversion').empty"
      :description="t('conversion').emptyHint"
    />
  </section>
</template>

<style scoped>
.media-card {
  border: 1px solid var(--border);
  border-radius: 12px;
  background: var(--surface);
  padding: 16px 20px;
  margin-top: 16px;
}
.media-card h2 {
  margin-top: 0;
}
.info-table {
  border-collapse: collapse;
  margin-bottom: 14px;
}
.info-table th,
.info-table td {
  text-align: left;
  border-bottom: 1px solid var(--border);
  padding: 5px 12px 5px 0;
}
.convert-row {
  display: flex;
  gap: 14px;
  align-items: center;
  flex-wrap: wrap;
}
.convert-row label {
  display: flex;
  flex-direction: column;
  min-width: 200px;
}
.stop {
  background: transparent;
  color: #a12631;
  border: 1px solid currentColor;
}
.error-text {
  color: #a12631;
  margin-top: 8px;
}
.ok-text {
  color: #14432a;
  font-weight: 600;
  margin-top: 8px;
}
.mono {
  font-family: ui-monospace, Menlo, monospace;
}
</style>
