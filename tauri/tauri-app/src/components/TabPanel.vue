<script setup lang="ts">
import { inject, computed, ref } from 'vue';

const props = defineProps<{
  id: string;
}>();

// Inject the active tab from parent component
const activeTab = inject('activeTab', ref(''));

// Check if this panel is active
const isActive = computed(() => activeTab.value === props.id);
</script>

<template>
  <div
    class="transition-all transform duration-300 ease-in-out"
    :style="{ display: isActive ? 'block' : 'none' }"
    :class="{ 
      'opacity-100 translate-x-0': isActive,
      'opacity-0 translate-x-4': !isActive 
    }"
  >
    <slot></slot>
  </div>
</template>