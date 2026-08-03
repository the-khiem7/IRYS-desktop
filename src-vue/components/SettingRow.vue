<script setup lang="ts">
/** One labelled setting: title, optional explanation, and a control. */

defineProps<{ label: string; hint?: string }>()
</script>

<template>
  <div class="row">
    <div class="copy">
      <span class="label">{{ label }}</span>
      <span v-if="hint" class="hint">{{ hint }}</span>
    </div>
    <div class="control">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.row {
  display: flex;
  gap: 16px;
  align-items: center;
  justify-content: space-between;
  padding: 12px 0;
  border-bottom: 1px solid var(--border);
}

.row:last-child {
  border-bottom: 0;
}

.copy {
  display: grid;
  gap: 2px;
  min-width: 0;
}

.label {
  font-weight: 500;
}

.hint {
  font-size: 12px;
  color: var(--text-dim);
}

.control {
  flex-shrink: 0;
}
</style>
