import { ref, InjectionKey, App, reactive } from 'vue';
import { invoke } from "@tauri-apps/api/core";
import type { ServerInfo, SharedItem } from '../types';

// 为WebSocket存储定义注入键
export const WS_STORE_KEY = Symbol('ws-store') as InjectionKey<ReturnType<typeof createWebSocketStore>>;

// 创建全局单例存储
let globalStore: ReturnType<typeof createWebSocketStore> | null = null;

export function createWebSocketStore() {
  let sharedItems = reactive<SharedItem[]>([]);
  const selectedItem = ref<SharedItem | null>(null);
  const itemContent = ref("");
  const serverAddress = ref("");
  const connectionStatus = ref("未连接");
  const loading = ref(false);
  const error = ref("");
  const reconnectAttempts = ref(0);
  const maxReconnectAttempts = ref(5);
  const wsReady = ref(false);
  const discoveredServers = ref<ServerInfo[]>([]);
  const isDiscovering = ref(false);
  const selectedServer = ref<ServerInfo | null>(null);

  let ws: WebSocket | null = null;
  let reconnectTimeout: number | null = null;

  // Ping状态标记
  const pongReceived = ref(false);

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
    console.log("Starting connectToServer...");
    // 关闭可能存在的连接
    closeWebSocketConnection();

    let targetServer: string;

    if (server) {
      // 连接到指定的服务器
      targetServer = `${server.server_address}:${server.server_port}`;
      selectedServer.value = server;
      console.log("Connecting to specified server:", targetServer);
    } else {
      // 使用本地默认地址
      try {
        // Rather than using get_server_address which doesn't exist,
        // use the local IP from get_local_ip with default port
        const localIp = await invoke("get_local_ip");
        targetServer = `${localIp}:8080`;
        console.log("Using local server address:", targetServer);
      } catch (e) {
        error.value = `获取服务器地址失败: ${e}`;
        console.error("Failed to get server address:", e);
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

      // 连接成功后，先发送ping确认服务器响应
      sendPing();

      // 连接后立即请求共享列表
      console.log("Requesting initial shared items list");
      requestSharedItems();
    };

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        console.log("WebSocket message received:", data);

        // 处理pong响应
        if (data.type === 'pong' || data.action === 'pong') {
          console.log("Received pong from server");
          pongReceived.value = true;
          // 清除之前可能设置的错误信息
          if (error.value === "服务器未响应ping请求") {
            error.value = "";
          }
          // 恢复正常连接状态显示
          if (connectionStatus.value === "服务器无响应") {
            connectionStatus.value = "已连接";
          }
          return;
        }

        // 处理推送的已发现服务列表
        if (data.type === 'discoveredServices') {
          console.log("收到推送的服务列表:", data.data);
          discoveredServers.value = data.data || [];
        }

        // 处理共享列表更新
        if (data.type === 'sharedItems' || data.sharedItems) {
          console.log("Updating shared items:", data.sharedItems || data.data);
          sharedItems.splice(0, sharedItems.length, ...(data.sharedItems || data.data || []));
          loading.value = false;
        }

        // 处理新增共享项
        if (data.type === 'itemAdded' || data.itemAdded) {
          const newItem = data.itemAdded || data.data;
          console.log("Adding new shared item:", newItem);
          if (!sharedItems.some(item => item.id === newItem.id)) {
            // 使用 push 方法而不是创建新数组，以保持响应式
            sharedItems.push(newItem);
          }
        }

        // 处理移除共享项
        if (data.type === 'itemRemoved' || data.itemRemoved) {
          const removedId = data.itemRemoved || data.data;
          console.log("Removing shared item:", removedId);
          // 使用 splice 方法从数组中移除项目
          const index = sharedItems.findIndex(item => item.id === removedId);
          if (index !== -1) {
            sharedItems.splice(index, 1);
          }
          
          // 如果当前选中的就是被删除的项，清除选中状态
          if (selectedItem.value && selectedItem.value.id === removedId) {
            selectedItem.value = null;
            itemContent.value = "";
          }
        }

        // 处理共享项内容
        if (data.type === 'itemContent' || (data.content && data.id)) {
          const content = data.content || data.data;
          const id = data.id;
          console.log("Received content for item:", id);
          itemContent.value = content;
          loading.value = false;
        }

        // 处理错误
        if (data.error) {
          console.error("Received error from server:", data.error);
          error.value = data.error;
          loading.value = false;
        }
      } catch (e) {
        console.error("Failed to parse WebSocket message:", e);
        error.value = "消息解析失败";
        loading.value = false;
      }
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
  }

  // 发送ping检查服务器连接状态
  function sendPing() {
    if (ws && ws.readyState === WebSocket.OPEN) {
      console.log("Sending ping to server");
      ws.send(JSON.stringify({ action: "ping", timestamp: Date.now() }));
      
      // 5秒后如果没收到pong响应，认为服务器无响应
      setTimeout(() => {
        if (wsReady.value && !pongReceived.value) {
          console.warn("No pong received from server within timeout");
          connectionStatus.value = "服务器无响应";
          error.value = "服务器未响应ping请求";
        }
      }, 5000);
    }
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
    // 如果已经在发现中，就不要重复发现
    if (isDiscovering.value) {
      return;
    }

    isDiscovering.value = true;
    error.value = "";

    try {
      const services = await invoke<ServerInfo[]>("discover_services");
      discoveredServers.value = services;
      console.log("Discovered services:", services);

      if (services.length === 0) {
        console.log("No services found on the network");
      }

      // 重要：即使没有发现服务也要清除加载状态
      isDiscovering.value = false;
      
      return services;
    } catch (e) {
      error.value = `发现服务失败: ${e}`;
      console.error("Error discovering services:", e);
      return [];
    } finally {
      // 确保在所有情况下都清除加载状态
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

  // 发送WebSocket消息
  function sendMessage(message: any) {
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify(message));
      return true;
    } else if (!ws || ws.readyState === WebSocket.CLOSED || ws.readyState === WebSocket.CLOSING) {
      // 如果WebSocket不存在或已关闭，尝试重新连接
      console.warn('WebSocket is not connected, attempting to reconnect...');
      connectionStatus.value = "正在重新连接...";
      connectToServer(selectedServer.value || undefined).then(() => {
        // 连接成功后，使用setTimeout确保WebSocket开启后再发送消息
        setTimeout(() => {
          if (ws && ws.readyState === WebSocket.OPEN) {
            ws.send(JSON.stringify(message));
          } else {
            error.value = "无法发送消息，连接失败";
          }
        }, 500);
      });
      return false;
    } else if (ws.readyState === WebSocket.CONNECTING) {
      // 如果WebSocket正在连接中，等待连接完成后再发送
      console.warn('WebSocket is connecting, waiting to send message...');
      const checkAndSend = () => {
        if (ws && ws.readyState === WebSocket.OPEN) {
          ws.send(JSON.stringify(message));
        } else if (ws && ws.readyState === WebSocket.CONNECTING) {
          // 仍在连接中，继续等待
          setTimeout(checkAndSend, 100);
        } else {
          error.value = "连接超时，无法发送消息";
        }
      };
      
      // 开始检查
      setTimeout(checkAndSend, 100);
      return false;
    }
    
    return false;
  }

  return {
    // 状态
    sharedItems,
    selectedItem,
    itemContent,
    serverAddress,
    connectionStatus,
    loading,
    error,
    wsReady,
    discoveredServers,
    isDiscovering,
    selectedServer,

    // 方法
    connectToServer,
    closeWebSocketConnection,
    discoverServices,
    requestSharedItems,
    sendMessage,
  };
}

// 创建并获取全局单例存储
export function getWebSocketStore() {
  if (!globalStore) {
    globalStore = createWebSocketStore();
  }
  return globalStore;
}

// 安装到 Vue 应用
export function installWebSocketStore(app: App) {
  const store = getWebSocketStore();
  app.provide(WS_STORE_KEY, store);
  return store;
}

// 使用状态管理 - 已不再需要，各组件应直接使用 inject(WS_STORE_KEY)
// export function useWebSocket() {
//   // 先尝试通过依赖注入获取
//   const store = inject(WS_STORE_KEY, null);
//   if (store) {
//     return store;
//   }
//   
//   // 如果注入失败，则返回全局单例
//   // 这样即使组件没有正确注入，也能访问全局状态
//   return getWebSocketStore();
// }