<script setup lang="ts">
import { ref, onMounted, onUnmounted, watchEffect } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { WebviewWindow } from "@tauri-apps/api/window";

// 文件信息接口
interface FileInfo {
  name: string;
  path: string;
  is_dir: boolean;
}

// 共享项信息接口
interface SharedItem {
  id: string;
  name: string;
  type: 'file' | 'text';
  content?: string;
  path?: string;
  fileType?: string;
  size?: number;
  username: string;
  uploadTime: number;
  updateTime?: number;
}

// 服务器信息接口
interface ServerInfo {
  message_type: string;
  server_address: string;
  server_port: number;
  server_name: string;
  timestamp: number;
}

// 状态变量
const sharedItems = ref<SharedItem[]>([]);
const selectedItem = ref<SharedItem | null>(null);
const itemContent = ref("");
const sharedDir = ref("");
const serverAddress = ref("");
const connectionStatus = ref("未连接");
const loading = ref(false);
const error = ref("");
const reconnectAttempts = ref(0);
const maxReconnectAttempts = 5;
const wsReady = ref(false);
const textToShare = ref("");
const discoveredServers = ref<ServerInfo[]>([]);
const isDiscovering = ref(false);
const selectedServer = ref<ServerInfo | null>(null);

// WebSocket连接
let ws: WebSocket | null = null;
let reconnectTimeout: number | null = null;

// 关闭WebSocket连接
function closeWebSocketConnection() {
  if (ws) {
    console.log("Closing existing WebSocket connection");

    // 移除所有事件监听器，防止内存泄漏
    ws.onopen = null;
    ws.onclose = null;
    ws.onerror = null;
    ws.onmessage = null;

    // 只有在连接打开或正在连接的状态下才需要关闭
    if (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING) {
      ws.close();
    }

    ws = null;
    wsReady.value = false;
  }

  // 清除所有重连超时
  if (reconnectTimeout !== null) {
    clearTimeout(reconnectTimeout);
    reconnectTimeout = null;
  }
}

// 连接到服务器
async function connectToServer(server?: ServerInfo) {
  // 关闭可能存在的连接
  closeWebSocketConnection();

  let targetServer: string;

  if (server) {
    // 连接到指定的服务器
    targetServer = `${server.server_address}:${server.server_port}`;
    selectedServer.value = server;
  } else {
    // 获取本地服务器地址
    try {
      targetServer = await invoke("get_server_address");
    } catch (e) {
      error.value = `获取服务器地址失败: ${e}`;
      console.error(e);
      return;
    }
  }

  serverAddress.value = targetServer;
  console.log(`Connecting to WebSocket server: ${targetServer}`);
  connectionStatus.value = "正在连接...";

  // 创建WebSocket连接
  ws = new WebSocket(`ws://${targetServer}`);

  ws.onopen = () => {
    console.log("WebSocket connection established");
    connectionStatus.value = "已连接";
    reconnectAttempts.value = 0;
    wsReady.value = true;
    error.value = "";

    // 连接后立即请求共享列表
    requestSharedItems();
  };

  ws.onclose = (event) => {
    console.log(`WebSocket connection closed, code: ${event.code}, reason: ${event.reason}`);
    wsReady.value = false;

    // 非正常关闭且未达到最大重连次数时尝试重连
    if (!event.wasClean && reconnectAttempts.value < maxReconnectAttempts.value) {
      connectionStatus.value = `连接断开，正在重连(${reconnectAttempts.value + 1}/${maxReconnectAttempts.value})...`;
      handleReconnect();
    } else {
      connectionStatus.value = "未连接";
    }
  };

  ws.onerror = (event) => {
    console.error("WebSocket error:", event);
    connectionStatus.value = "连接错误";
    error.value = "WebSocket连接失败";
    wsReady.value = false;
  };

  ws.onmessage = (event) => {
    try {
      const data = JSON.parse(event.data);
      console.log("WebSocket message received:", data);

      // 处理共享列表
      if (data.sharedItems) {
        sharedItems.value = data.sharedItems;
      }

      // 处理新增共享项
      if (data.itemAdded) {
        sharedItems.value = [data.itemAdded, ...sharedItems.value];
      }

      // 处理移除共享项
      if (data.itemRemoved) {
        sharedItems.value = sharedItems.value.filter(item => item.id !== data.itemRemoved);
      }

      // 处理共享项内容
      if (data.content && data.id) {
        itemContent.value = data.content;
        loading.value = false;
      }

      // 处理错误
      if (data.error) {
        error.value = data.error;
        loading.value = false;
      }
    } catch (e) {
      console.error("解析WebSocket消息失败:", e);
      loading.value = false;
    }
  };
}

// 处理重连逻辑
function handleReconnect() {
  reconnectAttempts.value++;

  // 使用指数退避算法增加重连间隔
  const delay = Math.min(1000 * Math.pow(2, reconnectAttempts.value - 1), 10000);

  console.log(`Reconnecting in ${delay}ms (attempt ${reconnectAttempts.value}/${maxReconnectAttempts.value})`);

  reconnectTimeout = window.setTimeout(() => {
    connectToServer(selectedServer.value || undefined);
  }, delay);
}

// 发现网络上的服务
async function discoverServices() {
  isDiscovering.value = true;
  error.value = "";

  try {
    const services = await invoke<ServerInfo[]>("discover_services");
    discoveredServers.value = services;
    console.log("Discovered services:", services);

    if (services.length === 0) {
      console.log("No services found on the network");
    }
  } catch (e) {
    error.value = `发现服务失败: ${e}`;
    console.error("Error discovering services:", e);
  } finally {
    isDiscovering.value = false;
  }
}

// 请求共享列表
function requestSharedItems() {
  if (!wsReady.value) {
    console.warn("WebSocket not ready, cannot request shared items");
    return;
  }

  if (ws && ws.readyState === WebSocket.OPEN) {
    console.log("Requesting shared items list");
    ws.send(JSON.stringify({ action: "getSharedItems" }));
  }
}

// 选择要分享的文件
async function selectFileToShare() {
  try {
    const selectedPath = await openDialog();
    if (selectedPath) {
      console.log(`Selected file to share: ${selectedPath}`);

      if (ws && ws.readyState === WebSocket.OPEN) {
        loading.value = true;
        ws.send(JSON.stringify({
          action: "shareFile",
          path: selectedPath
        }));
      }
    }
  } catch (e) {
    error.value = `选择文件失败: ${e}`;
    console.error("选择文件失败:", e);
  }
}

// 分享文本
function shareText() {
  if (!textToShare.value.trim()) {
    error.value = "请输入要分享的文本";
    return;
  }

  if (ws && ws.readyState === WebSocket.OPEN) {
    console.log("Sharing text content");
    ws.send(JSON.stringify({
      action: "shareText",
      content: textToShare.value
    }));
    textToShare.value = ""; // 清空输入框
  }
}

// 查看共享项内容
function viewSharedItem(item: SharedItem) {
  selectedItem.value = item;

  if (item.content) {
    // 文本类型，已有内容可直接显示
    itemContent.value = item.content;
  } else if (item.path && ws && ws.readyState === WebSocket.OPEN) {
    // 文件类型，需要请求内容
    loading.value = true;
    ws.send(JSON.stringify({
      action: "getItemContent",
      id: item.id
    }));
  }
}

// 删除共享项
function deleteSharedItem(item: SharedItem) {
  if (ws && ws.readyState === WebSocket.OPEN) {
    console.log(`Deleting shared item: ${item.id}`);
    ws.send(JSON.stringify({
      action: "deleteSharedItem",
      id: item.id
    }));

    // 如果当前选中的是被删除的项，则清空选择
    if (selectedItem.value && selectedItem.value.id === item.id) {
      selectedItem.value = null;
      itemContent.value = "";
    }
  }
}

// 监听网络状态变化
function setupNetworkListener() {
  window.addEventListener('online', () => {
    console.log("Network connection restored");
    if (connectionStatus.value !== "已连接" && !wsReady.value) {
      reconnectAttempts.value = 0; // 重置重连计数
      connectToServer(selectedServer.value || undefined);
    }
  });

  window.addEventListener('offline', () => {
    console.log("Network connection lost");
    connectionStatus.value = "网络断开";
  });
}

// 组件挂载时连接WebSocket
onMounted(() => {
  console.log("Component mounted, initializing network connection");
  setupNetworkListener();

  // 先尝试发现服务
  discoverServices().then(() => {
    if (discoveredServers.value.length > 0) {
      // 如果发现了服务，连接到第一个
      connectToServer(discoveredServers.value[0]);
    } else {
      // 否则连接到本地服务
      connectToServer();
    }
  }).catch(() => {
    // 如果发现服务失败，连接到本地服务
    connectToServer();
  });
});

// 组件销毁时关闭WebSocket连接
onUnmounted(() => {
  console.log("Component unmounted, cleaning up WebSocket connection");
  closeWebSocketConnection();
});

// 格式化文件大小函数
function formatFileSize(size: number) {
  if (size < 1024) return `${size} B`;
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(2)} KB`;
  if (size < 1024 * 1024 * 1024) return `${(size / (1024 * 1024)).toFixed(2)} MB`;
  return `${(size / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}
</script>

<template>
  <div class="min-h-screen bg-gray-100 dark:bg-gray-900">
    <!-- 主容器 -->
    <div class="mx-auto py-6 px-4">
      <!-- 侧边抽屉按钮 -->
      <button
        class="fixed top-1/2 left-0 transform -translate-y-1/2 bg-blue-500 text-white p-2 rounded-r focus:outline-none hover:bg-blue-600 z-50">
        📋
      </button>

      <!-- 头部 -->
      <header class="flex justify-between items-center mb-6 pb-3 border-b border-gray-200 dark:border-gray-700">
        <h1 class="text-2xl font-bold text-gray-800 dark:text-white">局域网文件文本传输</h1>
        <div class="flex items-center">
          <span class="bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 px-3 py-1 rounded-md text-sm mr-3">
            {{ serverAddress }}
          </span>
          <span :class="{
            'inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium': true,
            'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300': connectionStatus === '已连接',
            'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-300': connectionStatus === '连接错误',
            'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-300': connectionStatus === '未连接'
          }">
            {{ connectionStatus }}
          </span>
        </div>
      </header>

      <!-- 左右两栏布局 -->
      <div class="flex flex-col md:flex-row gap-6">
        <!-- 左侧栏 - 操作区域 -->
        <div class="md:w-1/3 lg:w-1/4 space-y-4">
          <!-- 错误提示 -->
          <section v-if="error"
            class="bg-red-100 dark:bg-red-900 border-l-4 border-red-500 text-red-700 dark:text-red-300 p-4 rounded-r">
            <div class="flex items-center">
              <span class="mr-2">⚠️</span>
              <p>{{ error }}</p>
            </div>
          </section>

          <!-- 服务发现区域 -->
          <section class="bg-white dark:bg-gray-800 rounded-lg shadow-sm p-4">
            <h2 class="text-lg font-semibold mb-3 text-gray-800 dark:text-white">网络服务</h2>
            <div class="mb-3">
              <button @click="discoverServices" :disabled="isDiscovering"
                class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition text-sm w-full disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center">
                <svg v-if="isDiscovering" class="animate-spin -ml-1 mr-2 h-4 w-4 text-white"
                  xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                  <path class="opacity-75" fill="currentColor"
                    d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z">
                  </path>
                </svg>
                {{ isDiscovering ? '发现中...' : '发现服务' }}
              </button>
            </div>
            <div v-if="discoveredServers.length > 0" class="mt-2 space-y-2">
              <div v-for="server in discoveredServers" :key="`${server.server_address}:${server.server_port}`"
                class="p-2 border border-gray-200 dark:border-gray-700 rounded hover:bg-gray-50 dark:hover:bg-gray-700 cursor-pointer"
                :class="{ 'border-blue-500 bg-blue-50 dark:bg-blue-900': selectedServer && selectedServer.server_address === server.server_address && selectedServer.server_port === server.server_port }"
                @click="connectToServer(server)">
                <div class="font-medium text-sm">{{ server.server_name }}</div>
                <div class="text-xs text-gray-500 dark:text-gray-400">{{ server.server_address }}:{{ server.server_port
                }}</div>
              </div>
            </div>
            <div v-else-if="!isDiscovering" class="text-sm text-gray-500 dark:text-gray-400 text-center italic">
              未发现网络服务
            </div>
          </section>

          <!-- 文件上传区 -->
          <section class="bg-white dark:bg-gray-800 rounded-lg shadow-sm p-4">
            <h2 class="text-lg font-semibold mb-3 text-gray-800 dark:text-white">分享文件</h2>
            <div class="max-w-md mx-auto rounded-lg overflow-hidden md:max-w-xl" @click="selectFileToShare">
              <div class="md:flex">
                <div class="w-full p-3">
                  <div
                    class="relative h-48 rounded-lg border-2 border-blue-500 bg-gray-50 flex justify-center items-center shadow-lg hover:shadow-xl transition-shadow duration-300 ease-in-out">
                    <div class="absolute flex flex-col items-center">
                      <img alt="File Icon" class="mb-3" src="./assets/image.png" />
                      <span class="block text-gray-500 font-semibold">拖拽你的文件到这里</span>
                      <span class="block text-gray-400 font-normal mt-1">或者点击这里上传</span>
                    </div>
                    <!-- <input name="" class="h-full w-full opacity-0 cursor-pointer" type="file"
                      :disabled="connectionStatus !== '已连接'" /> -->
                  </div>
                </div>
              </div>
            </div>
          </section>
          <!-- 添加文本分享区 -->
          <section class="bg-white dark:bg-gray-800 rounded-lg shadow-sm p-4">
            <h2 class="text-lg font-semibold mb-3 text-gray-800 dark:text-white">分享文本</h2>
            <textarea v-model="textToShare"
              class="w-full border dark:border-gray-600 rounded p-2 mb-3 focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-gray-300"
              rows="3" placeholder="输入要分享的文本..."></textarea>
            <button @click="shareText" :disabled="connectionStatus !== '已连接'"
              class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition text-sm w-full disabled:opacity-50 disabled:cursor-not-allowed">
              分享
            </button>
          </section>
        </div>

        <!-- 右侧栏 - 分享列表和文件内容 -->
        <div class="md:w-2/3 lg:w-3/4">
          <!-- 分享列表 -->
          <div class="bg-white dark:bg-gray-800 shadow rounded-lg overflow-hidden">
            <div class="p-4 bg-gray-50 dark:bg-gray-900 border-b border-gray-200 dark:border-gray-700">
              <h2 class="text-lg font-semibold text-gray-800 dark:text-white">分享列表</h2>
            </div>

            <div class="p-4">
              <div class="space-y-4">
                <!-- 当有共享项时 -->
                <div v-if="sharedItems.length > 0">
                  <div v-for="item in sharedItems" :key="item.id"
                    class="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg p-4 shadow-sm hover:shadow transition"
                    :class="{ 'border-blue-500 dark:border-blue-500': selectedItem && selectedItem.id === item.id }">
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
                        <button @click="viewSharedItem(item)"
                          class="px-3 py-1 border border-blue-300 text-blue-500 rounded text-sm hover:bg-blue-50 transition">
                          {{ item.type === 'text' ? '查看' : '预览' }}
                        </button>
                        <button @click="deleteSharedItem(item)"
                          class="px-3 py-1 border border-red-300 text-red-500 rounded text-sm hover:bg-red-50 transition">
                          删除
                        </button>
                      </div>
                    </div>

                    <!-- 共享项内容预览（仅当该项被选中时显示） -->
                    <div v-if="selectedItem && selectedItem.id === item.id"
                      class="mt-4 border-t border-gray-200 dark:border-gray-700 pt-4">
                      <div class="flex justify-between items-center mb-2">
                        <h3 class="text-md font-medium text-gray-700 dark:text-gray-300">{{ item.type === 'text' ?
                          '文本内容' : '文件内容' }}</h3>
                      </div>
                      <div v-if="!loading">
                        <pre
                          class="bg-gray-50 dark:bg-gray-900 p-4 rounded-lg overflow-x-auto whitespace-pre-wrap break-words text-gray-800 dark:text-gray-300">{{ itemContent }}</pre>
                      </div>
                      <div v-else class="flex justify-center items-center h-32 text-gray-500">
                        <svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-blue-500" xmlns="http://www.w3.org/2000/svg"
                          fill="none" viewBox="0 0 24 24">
                          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4">
                          </circle>
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
                <div v-else
                  class="bg-blue-100 dark:bg-blue-900 border-l-4 border-blue-500 text-blue-700 dark:text-blue-300 p-4 rounded">
                  <p>还没有共享的文件或文本，快来分享吧！</p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>