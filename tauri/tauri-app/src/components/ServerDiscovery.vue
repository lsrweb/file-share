<script setup lang="ts">
import { useWebSocket } from '../store/useWebSocket';
import type { ServerInfo } from '../types';
import { onMounted, onUnmounted, ref } from 'vue';

const wsStore = useWebSocket();
const autoDiscoveryInterval = ref<number | null>(null);
const emptyDiscoveryCount = ref(0);
const MAX_EMPTY_DISCOVERIES = 5;

const emit = defineEmits<{
  'discover': [];
  'select-server': [server: ServerInfo];
}>();

// 自动发现服务
async function startAutoDiscovery() {
  if (autoDiscoveryInterval.value !== null) {
    return;
  }

  emptyDiscoveryCount.value = 0;
  await emit('discover');

  // 每分钟自动发现一次
  autoDiscoveryInterval.value = window.setInterval(async () => {
    const previousCount = wsStore.discoveredServers.length;
    await emit('discover');

    // 检查是否发现了新服务
    if (wsStore.discoveredServers.length === previousCount) {
      emptyDiscoveryCount.value++;
      console.log(`No new servers found (attempt ${emptyDiscoveryCount.value}/${MAX_EMPTY_DISCOVERIES})`);

      // 如果连续5次没有发现新服务，停止自动发现
      if (emptyDiscoveryCount.value >= MAX_EMPTY_DISCOVERIES) {
        console.log('Max empty discoveries reached, stopping auto-discovery');
        stopAutoDiscovery();
      }
    } else {
      // 发现新服务，重置计数器
      emptyDiscoveryCount.value = 0;
    }
  }, 60000); // 1分钟
}

// 停止自动发现
function stopAutoDiscovery() {
  if (autoDiscoveryInterval.value !== null) {
    window.clearInterval(autoDiscoveryInterval.value);
    autoDiscoveryInterval.value = null;
  }
}

// 手动发现服务（重置自动发现）
async function handleManualDiscover() {
  stopAutoDiscovery();
  emptyDiscoveryCount.value = 0;
  await emit('discover');
  startAutoDiscovery();
}

// 组件挂载时启动自动发现
onMounted(() => {
  startAutoDiscovery();
});

// 组件卸载时清理定时器
onUnmounted(() => {
  stopAutoDiscovery();
});
</script>

<template>
  <section class="bg-white dark:bg-gray-800 rounded-lg shadow-sm p-4">
    <h2 class="text-lg font-semibold mb-3 text-gray-800 dark:text-white">网络服务</h2>
    <div class="mb-3">
      <button @click="handleManualDiscover"
        class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition text-sm w-full disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center">
        <svg v-if="wsStore.isDiscovering" class="animate-spin -ml-1 mr-2 h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg"
          fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor"
            d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z">
          </path>
        </svg>
        {{ wsStore.isDiscovering ? '发现中...' : '刷新服务' }}
      </button>
    </div>

    <!-- 自动发现状态提示 -->
    <div v-if="autoDiscoveryInterval === null && emptyDiscoveryCount >= MAX_EMPTY_DISCOVERIES" 
         class="text-xs text-gray-500 dark:text-gray-400 mb-2 text-center">
      自动发现已停止，可手动刷新
    </div>

    <div v-if="wsStore.discoveredServers.length > 0" class="mt-2 space-y-2">
      <div v-for="server in wsStore.discoveredServers" :key="`${server.server_address}:${server.server_port}`"
        class="p-2 border border-gray-200 dark:border-gray-700 rounded hover:bg-gray-50 dark:hover:bg-gray-700 cursor-pointer"
        :class="{ 'border-blue-500 bg-blue-50 dark:bg-blue-900': wsStore.selectedServer && wsStore.selectedServer.server_address === server.server_address && wsStore.selectedServer.server_port === server.server_port }"
        @click="emit('select-server', server)">
        <div class="font-medium text-sm">{{ server.server_name }}</div>
        <div class="text-xs text-gray-500 dark:text-gray-400">{{ server.server_address }}:{{ server.server_port }}</div>
      </div>
    </div>
    <div v-else-if="!wsStore.isDiscovering" class="text-sm text-gray-500 dark:text-gray-400 text-center italic">
      未发现网络服务
    </div>
  </section>
</template>