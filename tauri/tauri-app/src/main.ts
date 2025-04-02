import { createApp } from "vue";
import App from "./App.vue";
import "./core/styles/main.css";
import { installWebSocketStore } from "./store/useWebSocket";
import router from "./router";

const app = createApp(App);

// Install WebSocket store
installWebSocketStore(app);
// Use router
app.use(router);

app.mount("#app");
