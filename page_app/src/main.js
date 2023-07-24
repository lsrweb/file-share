import { createApp } from 'vue'
import App from './App.vue'
import installElementPlus from './plugins/element'
import store from "@/store";

const app = createApp(App)
installElementPlus(app)

app.use(store)

app.mount('#app')
