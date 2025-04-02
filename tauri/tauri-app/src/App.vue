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

// 状态变量
const files = ref<FileInfo[]>([]);
const selectedFile = ref<FileInfo | null>(null);
const fileContent = ref("");
const sharedDir = ref("");
const serverAddress = ref("");
const connectionStatus = ref("未连接");
const loading = ref(false);
const error = ref("");
const reconnectAttempts = ref(0);
const maxReconnectAttempts = 5;
const wsReady = ref(false);

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

// 连接到WebSocket服务器
async function connectToWebSocket() {
  // 先关闭可能存在的连接
  closeWebSocketConnection();
  
  // 获取服务器地址
  try {
    if (!serverAddress.value) {
      serverAddress.value = await invoke("get_server_address");
    }
    
    console.log(`Connecting to WebSocket server: ${serverAddress.value}`);
    connectionStatus.value = "正在连接...";
    
    // 创建WebSocket连接
    ws = new WebSocket(`ws://${serverAddress.value}`);
    
    ws.onopen = () => {
      console.log("WebSocket connection established");
      connectionStatus.value = "已连接";
      reconnectAttempts.value = 0;
      wsReady.value = true;
      error.value = "";
      
      // 连接后立即请求文件列表
      requestFileList();
    };
    
    ws.onclose = (event) => {
      console.log(`WebSocket connection closed, code: ${event.code}, reason: ${event.reason}`);
      wsReady.value = false;
      
      // 非正常关闭且未达到最大重连次数时尝试重连
      if (!event.wasClean && reconnectAttempts.value < maxReconnectAttempts) {
        connectionStatus.value = `连接断开，正在重连(${reconnectAttempts.value + 1}/${maxReconnectAttempts})...`;
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
        
        // 处理文件列表
        if (data.files) {
          files.value = data.files;
        }
        
        // 处理文件内容
        if (data.content && data.path) {
          fileContent.value = data.content;
          // 找到对应的文件
          selectedFile.value = files.value.find(f => f.path === data.path) || null;
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
  } catch (e) {
    error.value = `获取服务器地址失败: ${e}`;
    console.error("获取服务器地址失败:", e);
    connectionStatus.value = "未连接";
  }
}

// 处理重连逻辑
function handleReconnect() {
  reconnectAttempts.value++;
  
  // 使用指数退避算法增加重连间隔
  const delay = Math.min(1000 * Math.pow(2, reconnectAttempts.value - 1), 10000);
  
  console.log(`Reconnecting in ${delay}ms (attempt ${reconnectAttempts.value}/${maxReconnectAttempts})`);
  
  reconnectTimeout = window.setTimeout(() => {
    connectToWebSocket();
  }, delay);
}

// 请求文件列表
function requestFileList() {
  if (!wsReady.value) {
    console.warn("WebSocket not ready, cannot request file list");
    return;
  }
  
  if (ws && ws.readyState === WebSocket.OPEN) {
    console.log("Requesting file list");
    ws.send(JSON.stringify({ action: "getFileList" }));
  }
}

// 请求文件内容
function requestFileContent(file: FileInfo) {
  if (!wsReady.value) {
    console.warn("WebSocket not ready, cannot request file content");
    return;
  }
  
  if (ws && ws.readyState === WebSocket.OPEN && !file.is_dir) {
    console.log(`Requesting content for file: ${file.path}`);
    selectedFile.value = file;
    loading.value = true;
    ws.send(JSON.stringify({ action: "getFileContent", path: file.path }));
  }
}

// 选择共享目录
async function selectSharedDir() {
  try {
    const selectedPath = await openDialog();
    if (selectedPath) {
      console.log(`Setting shared directory: ${selectedPath}`);
      const result = await invoke("set_shared_dir", { path: selectedPath });
      sharedDir.value = selectedPath;
      requestFileList();
    }
  } catch (e) {
    error.value = `选择目录失败: ${e}`;
    console.error("选择目录失败:", e);
  }
}

// 监听网络状态变化
function setupNetworkListener() {
  window.addEventListener('online', () => {
    console.log("Network connection restored");
    if (connectionStatus.value !== "已连接" && !wsReady.value) {
      reconnectAttempts.value = 0; // 重置重连计数
      connectToWebSocket();
    }
  });
  
  window.addEventListener('offline', () => {
    console.log("Network connection lost");
    connectionStatus.value = "网络断开";
  });
}

// 组件挂载时连接WebSocket
onMounted(() => {
  console.log("Component mounted, initializing WebSocket connection");
  setupNetworkListener();
  connectToWebSocket();
});

// 组件销毁时关闭WebSocket连接
onUnmounted(() => {
  console.log("Component unmounted, cleaning up WebSocket connection");
  closeWebSocketConnection();
});
</script>

<template>
  <div class="min-h-screen bg-gray-100 dark:bg-gray-900">
    <!-- 主容器 -->
    <div class="container mx-auto py-6 px-4">
      <!-- 侧边抽屉按钮 -->
      <button
        class="fixed top-1/2 left-0 transform -translate-y-1/2 bg-blue-500 text-white p-2 rounded-r focus:outline-none hover:bg-blue-600 z-50"
      >
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
          <!-- 选择共享目录 -->
          <section class="bg-white dark:bg-gray-800 rounded-lg shadow-sm p-4">
            <div class="flex flex-wrap items-center gap-2">
              <button
                @click="selectSharedDir"
                :disabled="connectionStatus !== '已连接'"
                class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 transition duration-200 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                选择共享目录
              </button>
              <span v-if="sharedDir" class="text-sm text-gray-600 dark:text-gray-400 mt-2 break-all">
                当前共享: {{ sharedDir }}
              </span>
            </div>
          </section>

          <!-- 错误提示 -->
          <section v-if="error" class="bg-red-100 dark:bg-red-900 border-l-4 border-red-500 text-red-700 dark:text-red-300 p-4 rounded-r">
            <div class="flex items-center">
              <span class="mr-2">⚠️</span>
              <p>{{ error }}</p>
            </div>
          </section>

          <!-- 添加上传区 (在后续版本实现) -->
          <section class="bg-white dark:bg-gray-800 rounded-lg shadow-sm p-4">
            <h2 class="text-lg font-semibold mb-3 text-gray-800 dark:text-white">上传文件</h2>
            <div class="mb-3">
              <input 
                type="file" 
                class="w-full text-sm text-gray-600 dark:text-gray-400 file:mr-3 file:py-2 file:px-4 file:rounded file:border-0 file:text-sm file:bg-blue-50 file:text-blue-700 dark:file:bg-blue-900 dark:file:text-blue-200 hover:file:bg-blue-100 dark:hover:file:bg-blue-800 cursor-pointer"
              />
            </div>
            <button 
              class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition text-sm w-full"
            >
              上传
            </button>
          </section>

          <!-- 添加文本分享区 (在后续版本实现) -->
          <section class="bg-white dark:bg-gray-800 rounded-lg shadow-sm p-4">
            <h2 class="text-lg font-semibold mb-3 text-gray-800 dark:text-white">分享文本</h2>
            <textarea
              class="w-full border dark:border-gray-600 rounded p-2 mb-3 focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-gray-300"
              rows="3"
              placeholder="输入要分享的文本..."
            ></textarea>
            <button 
              class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition text-sm w-full"
            >
              分享
            </button>
          </section>

          <!-- 二维码 (在后续版本实现) -->
          <section class="bg-white dark:bg-gray-800 rounded-lg shadow-sm p-4 text-center">
            <div class="w-32 h-32 mx-auto bg-gray-200 dark:bg-gray-700 flex items-center justify-center mb-2">
              <span class="text-gray-500 dark:text-gray-400">二维码</span>
            </div>
            <p class="text-sm text-gray-600 dark:text-gray-400">扫描二维码访问此文件共享服务</p>
          </section>
        </div>

        <!-- 右侧栏 - 分享列表 -->
        <div class="md:w-2/3 lg:w-3/4">
          <div class="bg-white dark:bg-gray-800 shadow rounded-lg overflow-hidden">
            <div class="p-4 bg-gray-50 dark:bg-gray-900 border-b border-gray-200 dark:border-gray-700">
              <h2 class="text-lg font-semibold text-gray-800 dark:text-white">分享列表</h2>
            </div>
            
            <div class="p-4">
              <!-- 分享列表 -->
              <div class="space-y-4">
                <!-- 当有文件时 -->
                <div v-if="files.length > 0">
                  <div v-for="file in files" :key="file.path" class="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg p-4 shadow-sm hover:shadow transition">
                    <div class="flex flex-col sm:flex-row sm:justify-between sm:items-center">
                      <div class="mb-2 sm:mb-0">
                        <div class="flex items-center">
                          <span class="mr-2 text-xl">{{ file.is_dir ? '📁' : '📄' }}</span>
                          <span class="font-semibold text-gray-800 dark:text-white">{{ file.name }}</span>
                        </div>
                      </div>
                      <div class="flex">
                        <button
                          v-if="!file.is_dir"
                          @click="requestFileContent(file)"
                          class="px-3 py-1 border border-blue-300 text-blue-500 rounded mr-2 text-sm hover:bg-blue-50 transition"
                        >
                          预览
                        </button>
                      </div>
                    </div>
                  </div>
                </div>
                
                <!-- 当没有文件时 -->
                <div v-else class="bg-blue-100 dark:bg-blue-900 border-l-4 border-blue-500 text-blue-700 dark:text-blue-300 p-4 rounded">
                  <p>还没有共享的文件，请先选择共享目录！</p>
                </div>
              </div>
            </div>
          </div>

          <!-- 文件内容预览 -->
          <div v-if="selectedFile" class="mt-6 bg-white dark:bg-gray-800 shadow rounded-lg overflow-hidden">
            <div class="p-4 bg-gray-50 dark:bg-gray-900 border-b border-gray-200 dark:border-gray-700">
              <h2 class="text-lg font-semibold text-gray-800 dark:text-white">
                文件内容: {{ selectedFile.name }}
              </h2>
            </div>
            
            <div class="p-4">
              <pre v-if="!loading" class="bg-gray-50 dark:bg-gray-900 p-4 rounded-lg overflow-x-auto whitespace-pre-wrap break-words text-gray-800 dark:text-gray-300">{{ fileContent }}</pre>
              <div v-else class="flex justify-center items-center h-32 text-gray-500">
                <svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-blue-500" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
                加载中...
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style>
@tailwind base;
@tailwind components;
@tailwind utilities;
</style>
