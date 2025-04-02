<script setup lang="ts">
import { onMounted, onUnmounted, inject } from "vue";
import { open as openDialog } from "@tauri-apps/plugin-dialog";

// 导入组件
import ServerDiscovery from './components/ServerDiscovery.vue';
import FileUpload from './components/FileUpload.vue';
import TextShare from './components/TextShare.vue';
import SharedList from './components/SharedList.vue';
import HeaderBar from './components/HeaderBar.vue';

// 导入WebSocket状态管理
import { WS_STORE_KEY } from './store/useWebSocket';
import { ServerInfo } from "./types";

// 使用依赖注入获取WebSocket状态
const wsStore = inject(WS_STORE_KEY)!;

// 组件事件处理函数
const handleDiscoverServices = () => {
  wsStore.discoverServices();
};

const handleSelectServer = (server: ServerInfo | undefined) => {
  wsStore.connectToServer(server);
};

const handleSelectFile = async () => {
  try {
    const selectedPath = await openDialog();
    if (selectedPath) {
      console.log(`Selected file to share: ${selectedPath}`);
      wsStore.loading.value = true;
      wsStore.sendMessage({ 
        action: "shareFile", 
        path: selectedPath 
      });
    }
  } catch (e) {
    wsStore.error.value = `选择文件失败: ${e}`;
    console.error("选择文件失败:", e);
  }
};

const handleShareText = (text: string) => {
  wsStore.sendMessage({ 
    action: "shareText", 
    content: text 
  });
};

const handleViewItem = (item) => {
  wsStore.selectedItem.value = item;

  // if (item.content) {
  //   wsStore.itemContent.value = item.content;
  // } else if (item.path) {
  //   wsStore.loading.value = true;
  //   wsStore.sendMessage({
  //     action: "getItemContent",
  //     id: item.id
  //   });
  // }
};

const handleDeleteItem = (item) => {
  wsStore.sendMessage({
    action: "deleteSharedItem",
    id: item.id
  });

  if (wsStore.selectedItem.value && wsStore.selectedItem.value.id === item.id) {
    wsStore.selectedItem.value = null;
    wsStore.itemContent.value = "";
  }
};

// 组件挂载时自动连接
onMounted(() => {
  wsStore.connectToServer();
});

// 组件卸载时关闭连接
onUnmounted(() => {
  wsStore.closeWebSocketConnection();
});
</script>

<template>
  <div class="min-h-screen bg-gray-100 dark:bg-gray-900">
    <div class="mx-auto py-6 px-4">
      <HeaderBar />

      <!-- 错误提示 -->
      <!-- <div v-if="wsStore.error"
        class="bg-red-100 dark:bg-red-900 border-l-4 border-red-500 text-red-700 dark:text-red-300 p-4 rounded-r mb-6">
        <div class="flex items-center">
          <span class="mr-2">⚠️</span>
          <p>{{ wsStore.error }}</p>
        </div>
      </div> -->

      <!-- 左右两栏布局 -->
      <div class="flex flex-col md:flex-row gap-6">
        <!-- 左侧栏 - 操作区域 -->
        <div class="md:w-1/3 lg:w-1/4 space-y-4">
          <ServerDiscovery 
            @discover="handleDiscoverServices"
            @select-server="handleSelectServer"
          />
          
          <FileUpload 
            @select-file="handleSelectFile"
          />
          
          <TextShare 
            @share-text="handleShareText"
          />
        </div>

        <!-- 右侧栏 - 分享列表 -->
        <div class="md:w-2/3 lg:w-3/4">
          <SharedList 
            @view="handleViewItem"
            @delete="handleDeleteItem"
          />
        </div>
      </div>
    </div>
  </div>
</template>