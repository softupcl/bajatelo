<script setup lang="ts">
import { t } from '../i18n'
import EmptyState from '../components/EmptyState.vue'
import InfoBanner from '../components/InfoBanner.vue'
</script>

<template>
  <section aria-labelledby="downloads-title">
    <h1 id="downloads-title">
      {{ t('downloads').title }}
    </h1>
    <p class="lead">
      {{ t('downloads').intro }}
    </p>

    <InfoBanner :message="t('downloads').notIntegrated" />

    <div class="url-row">
      <label for="video-url" class="field-label">{{ t('downloads').urlLabel }}</label>
      <div class="url-controls">
        <input
          id="video-url"
          type="url"
          inputmode="url"
          autocomplete="off"
          :placeholder="t('downloads').urlPlaceholder"
          disabled
          aria-describedby="downloads-notice"
        />
        <button type="button" disabled>
          {{ t('downloads').analyze }}
        </button>
      </div>
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
</style>
