<script setup lang="ts">
import { ref, watch } from 'vue';

const props = defineProps({
  // Whether the drawer is open
  modelValue: {
    type: Boolean,
    default: false
  },
  // Direction from which the drawer appears
  direction: {
    type: String,
    default: 'right',
    validator: (value: string) => ['left', 'right'].includes(value)
  },
  // Width of the drawer (in pixels or as CSS value)
  width: {
    type: String,
    default: '300px'
  },
  // Title of the drawer
  title: {
    type: String,
    default: ''
  },
  // Whether to show close button
  showClose: {
    type: Boolean,
    default: true
  },
  // Whether clicking the backdrop closes the drawer
  closeOnBackdropClick: {
    type: Boolean,
    default: true
  },
  // Z-index of the drawer
  zIndex: {
    type: Number,
    default: 50
  }
});

const emit = defineEmits(['update:modelValue', 'close']);

const isVisible = ref(props.modelValue);

// Watch for changes in modelValue prop to update local state
watch(() => props.modelValue, (newValue) => {
  isVisible.value = newValue;
});

// Close the drawer
const closeDrawer = () => {
  isVisible.value = false;
  emit('update:modelValue', false);
  emit('close');
};

// Handle backdrop click
const handleBackdropClick = (event: MouseEvent) => {
  if (props.closeOnBackdropClick && event.target === event.currentTarget) {
    closeDrawer();
  }
};

// Calculate position and transform styles based on direction
const getDrawerStyle = () => {
  const style = {
    width: props.width,
    zIndex: props.zIndex + 1,
  };
  
  if (props.direction === 'left') {
    return {
      ...style,
      left: 0,
      right: 'auto',
    };
  } else {
    return {
      ...style, 
      left: 'auto',
      right: 0,
    };
  }
};

// Calculate transform class based on direction
const getTransformClass = () => {
  return props.direction === 'left' 
    ? 'translate-x-[-100%]' 
    : 'translate-x-[100%]';
};
</script>

<template>
  <div v-if="isVisible" class="relative" aria-modal="true" role="dialog">
    <!-- Backdrop -->
    <div
      @click="handleBackdropClick"
      class="fixed inset-0 bg-black bg-opacity-50 backdrop-blur-sm transition-opacity duration-300 ease-in-out"
      :style="{ zIndex: zIndex }"
    ></div>
    
    <!-- Drawer -->
    <div
      class="fixed top-0 bottom-0 bg-white dark:bg-gray-800 shadow-xl transition-transform duration-300 ease-in-out h-full"
      :class="[isVisible ? 'transform-none' : getTransformClass()]"
      :style="getDrawerStyle()"
    >
      <!-- Header -->
      <div class="flex justify-between items-center p-4 border-b border-gray-200 dark:border-gray-700">
        <h2 class="font-semibold text-lg text-gray-800 dark:text-white">{{ title }}</h2>
        <button
          v-if="showClose"
          @click="closeDrawer"
          class="p-2 rounded-full text-gray-500 hover:text-gray-800 dark:text-gray-400 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-700 focus:outline-none"
          aria-label="Close"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
          </svg>
        </button>
      </div>
      
      <!-- Content -->
      <div class="p-4 overflow-y-auto h-[calc(100%-64px)]">
        <slot></slot>
      </div>
    </div>
  </div>
</template>