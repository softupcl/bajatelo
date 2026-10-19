<script setup lang="ts">
import { computed, ref } from 'vue'
import { t } from '../i18n'
import { getBridge } from '../services/bridge'
import { errorText, toTaskError } from '../services/errors'
import { formatBytes, formatDuration } from '../services/format'
import EmptyState from '../components/EmptyState.vue'
import InfoBanner from '../components/InfoBanner.vue'
import type { MediaInfo, OutputFormat, TaskError } from '../types/contracts'

const url = ref('')
const analyzing = ref(false)
const info = ref<MediaInfo | null>(null)
const error = ref<string | null>(null)
const errorDetail = ref<string | null>(null)
const selectedQualityId = ref<string | null>(null)
const selectedFormat = ref<OutputFormat>('mp4')

const canAnalyze = computed(() => url.value.trim().length > 0 && !analyzing.value)

async function analyze(): Promise<void> {
  if (!canAnalyze.value) return
  analyzing.value = true
  error.value = null
  errorDetail.value = null
  info.value = null
  try {
    info.value = await getBridge().analyzeUrl(url.value)
    selectedQualityId.value = info.value.qualities[0]?.id ?? null
  } catch (e) {
    const taskError: TaskError | null = toTaskError(e)
    error.value = taskError ? errorText(taskError.code) : errorText('EC-23')
    errorDetail.value = taskError?.detail ?? null
  } finally {
    analyzing.value = false
  }
}

const FORMAT_LABELS: Record<OutputFormat, string> = {
  mp4: 'MP4 (H.264/AAC)',
  mp3: 'MP3 (solo audio)',
  original: 'Original (sin reconvertir)',
}
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
        <label>
          <span class="field-label">{{ t('analysis').quality }}</span>
          <select v-model="selectedQualityId">
            <option v-for="q in info.qualities" :key="q.id" :value="q.id">
              {{ q.label
              }}<template v-if="q.estimatedBytes"> · {{ formatBytes(q.estimatedBytes) }}</template>
            </option>
          </select>
        </label>
      </div>

      <InfoBanner :message="t('analysis').downloadPending" />
    </div>

    <h2>{{ t('downloads').queueTitle }}</h2>
    <EmptyState :title="t('downloads').emptyQueue" :description="t('downloads').emptyQueueHint" />
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
</style>
