<script setup lang="ts">
/**
 * Banner informativo honesto: se usa para estados "pendiente de integración",
 * nunca para simular funcionalidad que no existe.
 */
defineProps<{ message: string }>()
</script>

<template>
  <p class="notice" role="note">
    {{ message }}
  </p>
</template>

<style scoped>
.notice {
  background: var(--accent-soft);
  border-left: 4px solid var(--accent);
  border-radius: 6px;
  padding: 10px 14px;
  color: var(--text);
  margin: 0 0 16px;
}
</style>
