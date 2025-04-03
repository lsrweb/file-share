<script setup lang="ts">
import { inject } from 'vue';
import { WS_STORE_KEY } from '../store/useWebSocket';

const wsStore = inject(WS_STORE_KEY)!;

const emit = defineEmits<{
  'select-file': [];
}>();
</script>

<template>
  <div class="p-4">
    <div class="max-w-md mx-auto rounded-lg overflow-hidden md:max-w-xl" @click="emit('select-file')">
      <div class="md:flex">
        <div class="w-full">
          <div :class="{
              'relative h-48 rounded-lg border-2 bg-gray-50 flex justify-center items-center shadow-lg transition-shadow duration-300 ease-in-out': true,
              'border-blue-500 hover:shadow-xl cursor-pointer': wsStore.wsReady,
              'border-gray-300 opacity-50 cursor-not-allowed': !wsStore.wsReady
            }">
            <div class="absolute flex flex-col items-center">
              <img alt="File Icon" class="mb-3" src="../assets/image.png" />
              <span class="block text-gray-500 font-semibold">拖拽你的文件到这里</span>
              <span class="block text-gray-400 font-normal mt-1">或者点击这里上传</span>
              <span v-if="!wsStore.wsReady" class="text-red-500 text-sm mt-2">请先连接服务器</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>