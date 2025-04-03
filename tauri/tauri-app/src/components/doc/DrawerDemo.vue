<script setup lang="ts">
import { ref } from 'vue';
import Drawer from './Drawer.vue';

// Left drawer state
const showLeftDrawer = ref(false);
const toggleLeftDrawer = () => {
  showLeftDrawer.value = !showLeftDrawer.value;
};

// Right drawer state
const showRightDrawer = ref(false);
const toggleRightDrawer = () => {
  showRightDrawer.value = !showRightDrawer.value;
};

// Custom width drawer state
const showWideDrawer = ref(false);
const toggleWideDrawer = () => {
  showWideDrawer.value = !showWideDrawer.value;
};

// Drawer without close button state
const showNoCloseDrawer = ref(false);
const toggleNoCloseDrawer = () => {
  showNoCloseDrawer.value = !showNoCloseDrawer.value;
};
</script>

<template>
  <div class="p-4">
    <h2 class="text-xl font-bold mb-4">Drawer Component Demo</h2>
    
    <div class="flex flex-wrap gap-4 mb-8">
      <button 
        @click="toggleLeftDrawer" 
        class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition"
      >
        Open Left Drawer
      </button>
      
      <button 
        @click="toggleRightDrawer" 
        class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition"
      >
        Open Right Drawer
      </button>
      
      <button 
        @click="toggleWideDrawer" 
        class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition"
      >
        Open Wide Drawer
      </button>
      
      <button 
        @click="toggleNoCloseDrawer" 
        class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition"
      >
        Open Drawer (No Close Button)
      </button>
    </div>
    
    <!-- Left Drawer -->
    <Drawer
      v-model="showLeftDrawer"
      direction="left"
      title="Left Drawer"
      @close="showLeftDrawer = false"
    >
      <div class="space-y-4">
        <p>This drawer opens from the left side.</p>
        <p>Click the backdrop or close button to dismiss.</p>
      </div>
    </Drawer>
    
    <!-- Right Drawer -->
    <Drawer
      v-model="showRightDrawer"
      direction="right"
      title="Right Drawer"
      @close="showRightDrawer = false"
    >
      <div class="space-y-4">
        <p>This drawer opens from the right side.</p>
        <p>Click the backdrop or close button to dismiss.</p>
      </div>
    </Drawer>
    
    <!-- Wide Drawer -->
    <Drawer
      v-model="showWideDrawer"
      direction="right"
      width="500px"
      title="Wide Drawer"
      @close="showWideDrawer = false"
    >
      <div class="space-y-4">
        <p>This drawer has a custom width of 500px.</p>
        <p>Click the backdrop or close button to dismiss.</p>
      </div>
    </Drawer>
    
    <!-- Drawer without close button -->
    <Drawer
      v-model="showNoCloseDrawer"
      direction="left"
      :showClose="false"
      title="No Close Button"
      @close="showNoCloseDrawer = false"
    >
      <div class="space-y-4">
        <p>This drawer doesn't show the close button.</p>
        <p>Click the backdrop to dismiss or use the button below.</p>
        <button 
          @click="showNoCloseDrawer = false" 
          class="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700 transition"
        >
          Close Drawer
        </button>
      </div>
    </Drawer>
  </div>
</template>