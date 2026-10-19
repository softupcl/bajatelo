<script setup lang="ts">
import { ref } from 'vue'
import AppLayout from './layouts/AppLayout.vue'
import DownloadsPage from './pages/DownloadsPage.vue'
import ConversionPage from './pages/ConversionPage.vue'
import HistoryPage from './pages/HistoryPage.vue'
import SettingsPage from './pages/SettingsPage.vue'
import AboutPage from './pages/AboutPage.vue'

/**
 * Navegación por estado reactivo (sin vue-router): en una app de escritorio
 * las vistas no necesitan URLs; decisión documentada en docs/spec/architecture.md.
 */
type PageId = 'downloads' | 'conversion' | 'history' | 'settings' | 'about'
const page = ref<PageId>('downloads')
</script>

<template>
  <AppLayout v-model:active="page">
    <DownloadsPage v-if="page === 'downloads'" />
    <ConversionPage v-else-if="page === 'conversion'" />
    <HistoryPage v-else-if="page === 'history'" />
    <SettingsPage v-else-if="page === 'settings'" />
    <AboutPage v-else />
  </AppLayout>
</template>
