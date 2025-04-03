<script setup lang="ts">
import { inject } from 'vue';
import { WS_STORE_KEY } from '../store/useWebSocket';
import type { SharedItem } from '../types';
import { useRouter } from 'vue-router';

const wsStore = inject(WS_STORE_KEY)!;
const router = useRouter()

// 文件图标映射
const fileIcons = {
  'image': '🖼️',
  'video': '🎬',
  'audio': '🎵',
  'pdf': '📄',
  'text': '📝',
  'code': '📊',
  'markdown': '📑',
  'document': '📃',
  'spreadsheet': '📊',
  'presentation': '📊',
  'archive': '🗂️',
  'other': '📄'
};

// 格式化文件大小
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
  console.log(`Viewing item id: ${item.id}, name: ${item.name}`);

  // 修改为打开预览页面
  if (item.id) {
    router.push({ name: 'Preview', params: { id: item.id } });
  } else {
    emit('view', item);
  }
};

const handleDelete = (item: SharedItem) => {
  emit('delete', item);
};

// 获取文件图标
const getFileIcon = (item: SharedItem) => {
  if (item.type === 'text') {
    return item.contentType === 'markdown' ? fileIcons.markdown : fileIcons.text;
  }
  return item.fileType && fileIcons[item.fileType as keyof typeof fileIcons]
    ? fileIcons[item.fileType as keyof typeof fileIcons]
    : fileIcons.other;
};

// 获取预览按钮文字
const getViewButtonText = (item: SharedItem) => {
  if (item.type === 'text') {
    return '查看';
  }

  if (item.fileType) {
    switch (item.fileType) {
      case 'image':
        return '查看图片';
      case 'video':
        return '播放视频';
      case 'audio':
        return '播放音频';
      case 'pdf':
        return '查看PDF';
      case 'document':
      case 'spreadsheet':
      case 'presentation':
        return '查看文档';
      default:
        return '预览';
    }
  }

  return '预览';
};
</script>

<template>
  <div class="bg-white dark:bg-gray-800 shadow rounded-lg overflow-hidden">
    <div
      class="p-4 bg-gray-50 dark:bg-gray-900 border-b border-gray-200 dark:border-gray-700 flex justify-between items-center">
      <h2 class="text-lg font-semibold text-gray-800 dark:text-white">分享列表</h2>
      <div class="text-sm text-gray-500">
        {{ wsStore.connectionStatus }}
        <span v-if="wsStore.loading" class="ml-2">
          <span class="animate-spin inline-block">⌛</span>
        </span>
      </div>
    </div>

    <div class="p-4">
      <div v-if="!wsStore.wsReady"
        class="bg-yellow-100 dark:bg-yellow-900 border-l-4 border-yellow-500 text-yellow-700 dark:text-yellow-300 p-4 rounded">
        <p>正在连接服务器，请稍候...</p>
      </div>
      <div class="space-y-4" v-else>
        <!-- 当有共享项时 -->
        <div v-if="wsStore.sharedItems.length > 0">
          <div v-for="item in wsStore.sharedItems" :key="item.id"
            class="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg p-4 shadow-sm hover:shadow transition">
            <div class="flex flex-col sm:flex-row sm:justify-between sm:items-center">
              <div class="mb-2 sm:mb-0">
                <div class="flex items-center">
                  <span class="mr-2 text-xl">{{ getFileIcon(item) }}</span>
                  <span class="font-semibold text-gray-800 dark:text-white">{{ item.name }}</span>
                  <span v-if="item.fileType"
                    class="ml-2 text-xs px-2 py-1 rounded-full bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300">
                    {{ item.fileType }}
                  </span>
                  <span v-if="item.contentType === 'markdown'"
                    class="ml-2 text-xs px-2 py-1 rounded-full bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300">
                    Markdown
                  </span>
                </div>
                <div class="text-gray-500 text-xs mt-1">
                  <span>分享者: {{ item.username }}</span>
                  <span class="mx-1">|</span>
                  <span>时间: {{ new Date(item.upload_time).toLocaleString() }}</span>
                  <span v-if="item.size" class="mx-1">|</span>
                  <span v-if="item.size">大小: {{ formatFileSize(item.size) }}</span>
                </div>
              </div>
              <div class="flex gap-2">
                <button @click="handleView(item)"
                  class="px-3 py-1 border border-blue-300 text-blue-500 rounded text-sm hover:bg-blue-50 transition"
                  :disabled="!wsStore.wsReady">
                  {{ getViewButtonText(item) }}
                </button>
                <button @click="handleDelete(item)"
                  class="px-3 py-1 border border-red-300 text-red-500 rounded text-sm hover:bg-red-50 transition"
                  :disabled="!wsStore.wsReady">
                  删除
                </button>
              </div>
            </div>
          </div>
        </div>

        <!-- 当没有共享项时 -->
        <div v-else
          class="bg-blue-100 dark:bg-blue-900 border-l-4 border-blue-500 text-blue-700 dark:text-blue-300 p-4 rounded">
          <p>还没有共享的文件或文本，快来分享吧！</p>
        </div>

      </div>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.markdown-preview h1 {
  @apply text-2xl font-bold mt-4 mb-2;
}

.markdown-preview h2 {
  @apply text-xl font-bold mt-3 mb-2;
}

.markdown-preview h3 {
  @apply text-lg font-bold mt-3 mb-1;
}

.markdown-preview p {
  @apply mb-2;
}

.markdown-preview pre {
  @apply bg-gray-100 dark:bg-gray-800 p-2 rounded overflow-x-auto my-2;
}

.markdown-preview code {
  @apply bg-gray-100 dark:bg-gray-800 px-1 py-0.5 rounded font-mono;
}

.markdown-preview a {
  @apply text-blue-500 underline;
}

.markdown-preview ul, 
.markdown-preview ol {
  @apply pl-6 my-2;
}

.markdown-preview li {
  @apply mb-1;
}

.markdown-preview blockquote {
  @apply border-l-4 border-gray-200 dark:border-gray-700 pl-4 text-gray-500 dark:text-gray-400 my-2;
}

.markdown-preview table {
  @apply w-full border-collapse my-2;
}

.markdown-preview th,
.markdown-preview td {
  @apply border border-gray-200 dark:border-gray-700 p-2 text-left;
}

.markdown-preview th {
  @apply bg-gray-100 dark:bg-gray-800;
}
</style>