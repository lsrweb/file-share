import { createApp } from "vue";
import App from "./App.vue";
import "./core/styles/main.css";
import { installWebSocketStore } from "./store/useWebSocket";

const app = createApp(App);

// 安装 WebSocket store 到 Vue 应用
installWebSocketStore(app);

app.mount("#app");
