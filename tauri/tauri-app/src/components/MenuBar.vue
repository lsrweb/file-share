<template>
  <div class="menu-bar">
    <button class="menu-button" @click="minimizeWindow">
      <svg width="12" height="12" viewBox="0 0 12 12" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M1 6H11" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
      </svg>
    </button>
    <button class="menu-button close-button" @click="closeWindow">
      <svg width="12" height="12" viewBox="0 0 12 12" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M1 1L11 11M11 1L1 11" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
      </svg>
    </button>
  </div>
</template>

<script setup lang="ts">
import { hide } from '@tauri-apps/api/app';
import { exit } from '@tauri-apps/plugin-process';


const minimizeWindow = async () => {
  await hide();
};

const closeWindow = async () => {
  try {
    await exit();
  } catch (error) {
    console.error('Failed to exit:', error);
  }
};
</script>

<style scoped>
.menu-bar {
  display: flex;
  justify-content: flex-end;
  align-items: center;
  padding: 8px;
  background-color: transparent;
}

.menu-button {
  display: flex;
  justify-content: center;
  align-items: center;
  width: 32px;
  height: 32px;
  border-radius: 4px;
  background-color: transparent;
  border: none;
  color: var(--text-color);
  cursor: pointer;
  transition: background-color 0.2s;
}

.menu-button:hover {
  background-color: rgba(255, 255, 255, 0.1);
}

.close-button:hover {
  background-color: rgba(255, 0, 0, 0.2);
}
</style>
