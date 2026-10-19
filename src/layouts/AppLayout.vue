<script setup lang="ts">
import { t } from '../i18n'
import { ref } from 'vue'

type PageId = 'downloads' | 'conversion' | 'history' | 'settings' | 'about'

const nav = ref<Array<{ id: PageId; label: string }>>([
  { id: 'downloads', label: t('nav').downloads },
  { id: 'conversion', label: t('nav').conversion },
  { id: 'history', label: t('nav').history },
  { id: 'settings', label: t('nav').settings },
  { id: 'about', label: t('nav').about },
])

const active = defineModel<PageId>('active', { required: true })
</script>

<template>
  <div class="app-shell">
    <nav class="app-nav" aria-label="Navegación principal">
      <p class="app-brand">
        {{ t('app').title }}
      </p>
      <ul role="list">
        <li v-for="item in nav" :key="item.id">
          <button
            type="button"
            class="nav-button"
            :aria-current="active === item.id ? 'page' : undefined"
            :class="{ active: active === item.id }"
            @click="active = item.id"
          >
            {{ item.label }}
          </button>
        </li>
      </ul>
    </nav>
    <main id="main-content" class="app-main">
      <slot />
    </main>
  </div>
</template>

<style scoped>
.app-shell {
  display: flex;
  height: 100vh;
  overflow: hidden;
}
.app-nav {
  width: 190px;
  flex-shrink: 0;
  border-right: 1px solid var(--border);
  padding: 16px 10px;
  background: var(--surface);
}
.app-brand {
  font-weight: 700;
  font-size: 1.15rem;
  margin: 4px 10px 18px;
  letter-spacing: 0.02em;
}
.app-nav ul {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.nav-button {
  width: 100%;
  text-align: left;
  padding: 9px 12px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: var(--text);
  font-size: 0.95rem;
  cursor: pointer;
}
.nav-button:hover {
  background: var(--surface-hover);
}
.nav-button.active {
  background: var(--accent-soft);
  color: var(--accent-strong);
  font-weight: 600;
}
.nav-button:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
}
.app-main {
  flex: 1;
  overflow-y: auto;
  padding: 24px 28px;
}
</style>
