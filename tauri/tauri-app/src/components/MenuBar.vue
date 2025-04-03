<template>
  <div class="flex justify-end items-center p-2 bg-transparent " >
    <button class="flex justify-center items-center w-8 h-6 rounded bg-transparent border-none text-gray-700 dark:text-gray-300 cursor-pointer transition-colors hover:bg-gray-200 dark:hover:bg-gray-700" @click="minimizeWindow">
      <svg width="12" height="12" viewBox="0 0 12 12" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M1 6H11" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
      </svg>
    </button>
    <button class="flex justify-center items-center w-8 h-6 rounded bg-transparent border-none text-gray-700 dark:text-gray-300 cursor-pointer transition-colors hover:bg-red-100 dark:hover:bg-red-900" @click="closeWindow">
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
    await exit(0);
  } catch (error) {
    console.error('Failed to exit:', error);
  }
};
</script>
