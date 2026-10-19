<script setup lang="ts">
import { computed } from 'vue'
import type { ToolStatus } from '../types/contracts'

const props = defineProps<{ status: ToolStatus }>()

const label = computed(() => {
  const labels: Record<ToolStatus, string> = {
    ok: 'Disponible',
    missing: 'No encontrada',
    incompatible: 'Incompatible',
  }
  return labels[props.status]
})
</script>

<template>
  <span class="badge" :class="`badge-${status}`" role="status">{{ label }}</span>
</template>

<style scoped>
.badge {
  display: inline-block;
  padding: 2px 10px;
  border-radius: 999px;
  font-size: 0.8rem;
  font-weight: 600;
}
.badge-ok {
  background: #d8f3dc;
  color: #14432a;
}
.badge-missing {
  background: #fde8d8;
  color: #5c3200;
}
.badge-incompatible {
  background: #f8d8da;
  color: #611a20;
}
</style>
