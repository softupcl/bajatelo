<script setup lang="ts">
import { t } from '../i18n'
import EmptyState from '../components/EmptyState.vue'
</script>

<template>
  <section aria-labelledby="history-title">
    <h1 id="history-title">
      {{ t('history').title }}
    </h1>
    <p class="lead">
      {{ t('history').intro }}
    </p>
    <EmptyState :title="t('history').empty" :description="t('history').emptyHint" />
  </section>
</template>
