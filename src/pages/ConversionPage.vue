<script setup lang="ts">
import { t } from '../i18n'
import EmptyState from '../components/EmptyState.vue'
import InfoBanner from '../components/InfoBanner.vue'
</script>

<template>
  <section aria-labelledby="conversion-title">
    <h1 id="conversion-title">
      {{ t('conversion').title }}
    </h1>
    <p class="lead">
      {{ t('conversion').intro }}
    </p>

    <InfoBanner :message="t('conversion').notIntegrated" />

    <EmptyState :title="t('conversion').empty" :description="t('conversion').emptyHint">
      <button type="button" disabled>
        {{ t('conversion').pickFile }}
      </button>
    </EmptyState>
  </section>
</template>
