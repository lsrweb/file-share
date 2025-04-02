<script setup lang="ts">
import { ref } from 'vue';
import { useWebSocket } from '../store/useWebSocket';

const wsStore = useWebSocket();
const textContent = ref('');

const handleShare = () => {
  if (textContent.value.trim()) {
    wsStore.sendMessage({
      action: "shareText",
      content: textContent.value
    });
    textContent.value = '';
  }
};
</script>

<template>
  <section class="bg-white dark:bg-gray-800 rounded-lg shadow-sm p-4">
    <h2 class="text-lg font-semibold mb-3 text-gray-800 dark:text-white">分享文本</h2>
    <textarea v-model="textContent"
      class="w-full border dark:border-gray-600 rounded p-2 mb-3 focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-gray-300"
      rows="3" placeholder="输入要分享的文本..."></textarea>
    <button @click="handleShare" :disabled="!wsStore.wsReady"
      class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition text-sm w-full disabled:opacity-50 disabled:cursor-not-allowed">
      分享
    </button>
  </section>
</template>