<script setup lang="ts">
import { inject } from 'vue';
import { WS_STORE_KEY } from '../store/useWebSocket';
import MenuBar from './MenuBar.vue';

const wsStore = inject(WS_STORE_KEY)!;
</script>

<template>
  <header class="flex justify-between items-center mb-6 pb-3 border-b border-gray-200 dark:border-gray-700">
    <MenuBar />
    <h1 class="text-2xl font-bold text-gray-800 dark:text-white">局域网文件文本传输</h1>
    <div class="flex items-center">
      <span class="bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 px-3 py-1 rounded-md text-sm mr-3">
        {{ wsStore.serverAddress }}
      </span>
      
      <span :class="{
        'inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium': true,
        'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300': wsStore.connectionStatus.value === '已连接',
        'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-300': wsStore.connectionStatus.value === '连接错误',
        'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-300': wsStore.connectionStatus.value === '未连接'
      }">
        {{ wsStore.connectionStatus.value }}
      </span>
    </div>
  </header>
</template>
