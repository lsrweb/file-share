<template>
  <div>
    <div :style="serverStatus === 'start' ? 'display:none' : ''">
      <div class="btn-box">
        <div class="start-btn" @click="startServer">开启服务</div>
        <div class="start-btn-shadow">
          <span style="--i:1"></span>
          <span style="--i:2"></span>
          <span style="--i:3"></span>
          <span style="--i:4"></span>
          <span style="--i:5"></span>
          <span style="--i:6"></span>
          <span style="--i:7"></span>
          <span style="--i:8"></span>
          <span style="--i:9"></span>
          <span style="--i:10"></span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from "vue";
import { useStore } from "vuex";

let api = window.api;

let store = useStore();

let serverStatus = computed(() => {
  return store.state.serverStatus;
});

let startServer = () => {
  store.commit("setSettingForm", api.getSetting());
  api.startServer();
};
</script>

<style></style>
