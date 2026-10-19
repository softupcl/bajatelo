<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { t } from '../i18n'
import { getBridge } from '../services/bridge'

const version = ref('…')

onMounted(async () => {
  try {
    version.value = await getBridge().getAppVersion()
  } catch {
    version.value = 'desconocida'
  }
})
</script>

<template>
  <section aria-labelledby="about-title">
    <h1 id="about-title">
      {{ t('about').title }}
    </h1>
    <p class="lead">
      <strong>{{ t('app').title }}</strong> — {{ t('app').tagline }} · {{ t('about').version }}
      {{ version }}
    </p>
    <p>{{ t('about').description }}</p>

    <h2>{{ t('about').creditsTitle }}</h2>
    <p>{{ t('about').credits }}</p>

    <h2>{{ t('about').licenseTitle }}</h2>
    <p>{{ t('about').license }}</p>
  </section>
</template>
