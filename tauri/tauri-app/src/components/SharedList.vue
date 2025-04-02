<script setup lang="ts">
import { useWebSocket } from '../store/useWebSocket';
import type { SharedItem } from '../types';

const wsStore = useWebSocket();

function formatFileSize(size: number) {
  if (size < 1024) return `${size} B`;
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(2)} KB`;
  if (size < 1024 * 1024 * 1024) return `${(size / (1024 * 1024)).toFixed(2)} MB`;
  return `${(size / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

const emit = defineEmits<{
  'view': [item: SharedItem];
  'delete': [item: SharedItem];
}>();

const handleView = (item: SharedItem) => {
  emit('view', item);
};

const handleDelete = (item: SharedItem) => {
  emit('delete', item);
};
</script>

<template>
  <div class="bg-white dark:bg-gray-800 shadow rounded-lg overflow-hidden">
    <div class="p-4 bg-gray-50 dark:bg-gray-900 border-b border-gray-200 dark:border-gray-700 flex justify-between items-center">
      <h2 class="text-lg font-semibold text-gray-800 dark:text-white">分享列表</h2>
      <div class="text-sm text-gray-500">
        {{ wsStore.connectionStatus }}
        <span v-if="wsStore.loading" class="ml-2">
          <span class="animate-spin inline-block">⌛</span>
        </span>
      </div>
    </div>

    <div class="p-4">
      <div v-if="!wsStore.wsReady" class="bg-yellow-100 dark:bg-yellow-900 border-l-4 border-yellow-500 text-yellow-700 dark:text-yellow-300 p-4 rounded">
        <p>正在连接服务器，请稍候...</p>
      </div>
      
      <div class="space-y-4" v-else>
        <!-- 当有共享项时 -->
        <div v-if="wsStore.sharedItems.length > 0">
          <div v-for="item in wsStore.sharedItems" :key="item.id"
            class="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg p-4 shadow-sm hover:shadow transition"
            :class="{ 'border-blue-500 dark:border-blue-500': wsStore.selectedItem && wsStore.selectedItem.id === item.id }">
            <div class="flex flex-col sm:flex-row sm:justify-between sm:items-center">
              <div class="mb-2 sm:mb-0">
                <div class="flex items-center">
                  <span class="mr-2 text-xl">{{ item.type === 'file' ? '📄' : '📝' }}</span>
                  <span class="font-semibold text-gray-800 dark:text-white">{{ item.name }}</span>
                  <span v-if="item.fileType"
                    class="ml-2 text-xs px-2 py-1 rounded-full bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300">
                    {{ item.fileType }}
                  </span>
                </div>
                <div class="text-gray-500 text-xs mt-1">
                  <span>分享者: {{ item.username }}</span>
                  <span class="mx-1">|</span>
                  <span>时间: {{ new Date(item.uploadTime).toLocaleString() }}</span>
                  <span v-if="item.size" class="mx-1">|</span>
                  <span v-if="item.size">大小: {{ formatFileSize(item.size) }}</span>
                </div>
              </div>
              <div class="flex gap-2">
                <button @click="handleView(item)"
                  class="px-3 py-1 border border-blue-300 text-blue-500 rounded text-sm hover:bg-blue-50 transition"
                  :disabled="!wsStore.wsReady">
                  {{ item.type === 'text' ? '查看' : '预览' }}
                </button>
                <button @click="handleDelete(item)"
                  class="px-3 py-1 border border-red-300 text-red-500 rounded text-sm hover:bg-red-50 transition"
                  :disabled="!wsStore.wsReady">
                  删除
                </button>
              </div>
            </div>

            <!-- 共享项内容预览（仅当该项被选中时显示） -->
            <div v-if="wsStore.selectedItem && wsStore.selectedItem.id === item.id"
              class="mt-4 border-t border-gray-200 dark:border-gray-700 pt-4">
              <div class="flex justify-between items-center mb-2">
                <h3 class="text-md font-medium text-gray-700 dark:text-gray-300">
                  {{ item.type === 'text' ? '文本内容' : '文件内容' }}
                </h3>
              </div>
              <div v-if="!wsStore.loading">
                <pre
                  class="bg-gray-50 dark:bg-gray-900 p-4 rounded-lg overflow-x-auto whitespace-pre-wrap break-words text-gray-800 dark:text-gray-300">{{ wsStore.itemContent }}</pre>
              </div>
              <div v-else class="flex justify-center items-center h-32 text-gray-500">
                <svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-blue-500" xmlns="http://www.w3.org/2000/svg" fill="none"
                  viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                  <path class="opacity-75" fill="currentColor"
                    d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z">
                  </path>
                </svg>
                加载中...
              </div>
            </div>
          </div>
        </div>

        <!-- 当没有共享项时 -->
        <div v-else class="bg-blue-100 dark:bg-blue-900 border-l-4 border-blue-500 text-blue-700 dark:text-blue-300 p-4 rounded">
          <p>还没有共享的文件或文本，快来分享吧！</p>
        </div>
      </div>
    </div>
  </div>
</template>