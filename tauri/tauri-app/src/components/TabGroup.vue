<script setup lang="ts">
import { ref, provide } from 'vue';

// Tab interface
interface Tab {
  id: string;
  title: string;
}

const props = defineProps<{
  tabs: Tab[];
  defaultTab?: string;
}>();

// Active tab state
const activeTab = ref(props.defaultTab || (props.tabs.length > 0 ? props.tabs[0].id : ''));

// Provide the active tab to child components
provide('activeTab', activeTab);

// Change active tab
const setActiveTab = (tabId: string) => {
  activeTab.value = tabId;
};

// Check if tab is active
const isActive = (tabId: string) => activeTab.value === tabId;
</script>

<template>
  <div class="tab-group">
    <!-- Tab headers -->
    <div class="flex border-b border-gray-200 dark:border-gray-700">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        @click="setActiveTab(tab.id)"
        :class="[
          'py-2 px-4 font-medium text-sm transition-all duration-200 focus:outline-none',
          isActive(tab.id)
            ? 'text-blue-600 dark:text-blue-400 border-b-2 border-blue-600 dark:border-blue-400'
            : 'text-gray-500 dark:text-gray-400 border-b-2 border-transparent hover:text-gray-700 dark:hover:text-gray-300'
        ]"
      >
        {{ tab.title }}
      </button>
    </div>
    
    <!-- Tab content -->
    <div class="mt-1 relative">
      <slot></slot>
    </div>
  </div>
</template>