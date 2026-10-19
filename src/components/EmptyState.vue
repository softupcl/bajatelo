<script setup lang="ts">
defineProps<{
  title: string
  description?: string
}>()
</script>

<template>
  <div class="empty-state" role="status">
    <p class="empty-title">
      {{ title }}
    </p>
    <p v-if="description" class="empty-hint">
      {{ description }}
    </p>
    <div v-if="$slots.default" class="empty-action">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.empty-state {
  border: 1px dashed var(--border);
  border-radius: 12px;
  padding: 36px 24px;
  text-align: center;
  background: var(--surface);
  margin-top: 12px;
}
.empty-title {
  font-weight: 600;
  margin: 0 0 6px;
}
.empty-hint {
  color: var(--text-muted);
  margin: 0 auto;
  max-width: 46ch;
}
.empty-action {
  margin-top: 14px;
}
</style>
