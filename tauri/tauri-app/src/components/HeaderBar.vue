<script setup lang="ts">
import { inject } from 'vue';
import { WS_STORE_KEY } from '../store/useWebSocket';
import MenuBar from './MenuBar.vue';

const wsStore = inject(WS_STORE_KEY)!;
</script>

<template>
  <header class="flex justify-between items-center mb-6 pb-1 border-b border-gray-200 dark:border-gray-700 select-none" data-tauri-drag-region>
    <h1 class="text-gray-800 dark:text-white">局域网文件文本传输</h1>
    <div class="flex items-center">
      <span class="bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 px-3 py-1 rounded-md text-sm mr-3">
        本机ip
        {{ wsStore.serverAddress }}
      </span>
      
      <MenuBar />

    </div>
  </header>
</template>
