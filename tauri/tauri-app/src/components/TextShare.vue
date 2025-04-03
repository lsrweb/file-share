<script setup lang="ts">
import { ref, inject } from 'vue';
import { WS_STORE_KEY } from '../store/useWebSocket';

const wsStore = inject(WS_STORE_KEY)!;
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
  <div class="p-4">
    <div class="relative">
      <!-- Improved textarea with custom scrollbar styling -->
      <textarea v-model="textContent" class="w-full min-h-[160px] p-4 pr-12 rounded-lg resize-none border border-gray-200 shadow-sm 
               dark:border-gray-700 dark:bg-gray-800 dark:text-gray-200 
               focus:ring-2 focus:ring-blue-500 focus:border-transparent focus:outline-none 
               transition-all duration-200
               scrollbar-thin scrollbar-thumb-gray-300 scrollbar-track-transparent dark:scrollbar-thumb-gray-600"
        placeholder="输入要分享的文本..."></textarea>

      <!-- Smaller floating send button -->
      <button @click="handleShare" :disabled="!wsStore.wsReady || !textContent.trim()" class="absolute bottom-3 right-3 rounded-full w-8 h-8 flex items-center justify-center
               bg-blue-600 text-white shadow-md hover:bg-blue-700 
               disabled:opacity-50 disabled:cursor-not-allowed
               transform hover:scale-105 transition-all duration-200" title="分享文本">
        <!-- Send icon (slightly smaller) -->
        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
          <path
            d="M10.894 2.553a1 1 0 00-1.788 0l-7 14a1 1 0 001.169 1.409l5-1.429A1 1 0 009 15.571V11a1 1 0 112 0v4.571a1 1 0 00.725.962l5 1.428a1 1 0 001.17-1.408l-7-14z" />
        </svg>
      </button>

      <!-- Status indicator when not connected -->
      <div v-if="!wsStore.wsReady"
        class="absolute top-2 right-2 text-xs px-2 py-0.5 rounded-full bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200">
        未连接
      </div>
    </div>
  </div>
</template>

<style>
/* Custom scrollbar styling */
.scrollbar-thin::-webkit-scrollbar {
  width: 6px;
}

.scrollbar-thin::-webkit-scrollbar-track {
  background: transparent;
}

.scrollbar-thumb-gray-300::-webkit-scrollbar-thumb {
  background-color: #d1d5db;
  border-radius: 3px;
}

.dark .dark\:scrollbar-thumb-gray-600::-webkit-scrollbar-thumb {
  background-color: #4b5563;
}

/* Firefox scrollbar styling */
.scrollbar-thin {
  scrollbar-width: thin;
  scrollbar-color: #d1d5db transparent;
}

.dark .dark\:scrollbar-thumb-gray-600 {
  scrollbar-color: #4b5563 transparent;
}
</style>