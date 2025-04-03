<script setup lang="ts">
import { onMounted, onUnmounted, inject, ref } from "vue";
import { open as openDialog } from "@tauri-apps/plugin-dialog";

// Import components
import ServerDiscovery from '../components/ServerDiscovery.vue';
import FileUpload from '../components/FileUpload.vue';
import TextShare from '../components/TextShare.vue';
import SharedList from '../components/SharedList.vue';
import HeaderBar from '../components/HeaderBar.vue';
import TabGroup from '../components/TabGroup.vue';
import TabPanel from '../components/TabPanel.vue';

// Import WebSocket state management
import { WS_STORE_KEY } from '../store/useWebSocket';
import { ServerInfo, SharedItem } from "../types";
import Drawer from "../components/Drawer.vue";

// Use dependency injection to get WebSocket state
const wsStore = inject(WS_STORE_KEY)!;

// Define tabs for sharing options
const sharingTabs = [
  { id: 'file', title: '分享文件' },
  { id: 'text', title: '分享文本' },
];

// Component event handlers
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
      wsStore.loading.value = true;
      wsStore.sendMessage({
        action: "shareFile",
        path: selectedPath
      });
    }
  } catch (e) {
    wsStore.error.value = `Failed to select file: ${e}`;
  }
};

const handleShareText = (text: string) => {
  wsStore.sendMessage({
    action: "shareText",
    content: text
  });
};

const handleViewItem = (item: SharedItem) => {
  wsStore.selectedItem.value = item;
};

const handleDeleteItem = (item: SharedItem) => {
  wsStore.sendMessage({
    action: "deleteSharedItem",
    id: item.id
  });

  if (wsStore.selectedItem.value && wsStore.selectedItem.value.id === item.id) {
    wsStore.selectedItem.value = null;
    wsStore.itemContent.value = "";
  }
};

// Auto-connect when component mounts
onMounted(() => {
  wsStore.connectToServer();
});

// Close connection when component unmounts
onUnmounted(() => {
  wsStore.closeWebSocketConnection();
});

const showLeftDrawer = ref(false);
const toggleLeftDrawer = () => {
  showLeftDrawer.value = !showLeftDrawer.value;
};

</script>

<template>
  <div class="min-h-screen bg-gray-100 dark:bg-gray-900">
    <div class="mx-auto py-1 px-4">
      <HeaderBar />

      <Drawer v-model="showLeftDrawer" direction="left" title="Left Drawer" @close="showLeftDrawer = false" />

      <!-- Two-column layout -->
      <div class="flex flex-col md:flex-row gap-6">
        <!-- Left column - Operation area -->
        <div class="md:w-1/3 lg:w-1/4 space-y-4">
          <ServerDiscovery @discover="handleDiscoverServices" @select-server="handleSelectServer" />

          <!-- Tabbed interface for sharing options -->
          <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm overflow-hidden">
            <TabGroup :tabs="sharingTabs" defaultTab="text">
              <!-- Text sharing tab -->
              <TabPanel id="text">
                <TextShare @share-text="handleShareText" />
              </TabPanel>
              <!-- File sharing tab -->
              <TabPanel id="file">
                <FileUpload @select-file="handleSelectFile" />
              </TabPanel>
            </TabGroup>
          </div>
        </div>

        <!-- Right column - Shared list -->
        <div class="md:w-2/3 lg:w-3/4">
          <SharedList @view="handleViewItem" @delete="handleDeleteItem" />
        </div>
      </div>
    </div>
  </div>
</template>