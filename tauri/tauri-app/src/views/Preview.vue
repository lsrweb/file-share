<script setup lang="ts">
import { ref, onMounted, watch, onUnmounted } from 'vue';
import { useRoute } from 'vue-router';

const route = useRoute();
const fileId = ref(route.params.id as string);
const file = ref<any>(null);
const fileContent = ref<string>('');
const previewType = ref<string>('');
const isLoading = ref(true);
const error = ref('');
const scale = ref(1.0);
const pageNum = ref(1);
const totalPages = ref(0);

// 格式化文件大小
function formatFileSize(size: number) {
  if (size < 1024) return `${size} B`;
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(2)} KB`;
  if (size < 1024 * 1024 * 1024) return `${(size / (1024 * 1024)).toFixed(2)} MB`;
  return `${(size / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

// 加载文件信息
async function loadFileInfo() {
  if (!fileId.value) return;
  
  isLoading.value = true;
  error.value = '';
  
  try {
    // 调用后端API获取文件信息
    const apiEndpoint = `/api/files/${fileId.value}`;
    const response = await fetch(apiEndpoint);

    if (!response.ok) {
      throw new Error(`Server returned an error: ${response.status}`);
    }

    const data = await response.json();

    if (!data.success) {
      throw new Error(data.message || 'Failed to fetch file details');
    }

    file.value = data.data;
    determinePreviewType();

    if (previewType.value === 'text' && file.value.type !== 'file') {
      fileContent.value = file.value.content || '';
    }
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load file details';
    console.error('Error loading file details:', e);
  } finally {
    isLoading.value = false;
  }
}

// 根据文件类型确定预览类型
function determinePreviewType() {
  if (!file.value) {
    previewType.value = '';
    return;
  }
  
  // 文本类型
  if (file.value.type !== 'file') {
    previewType.value = 'text';
    return;
  }
  
  // 文件类型，根据mimeType判断
  const mimeType = file.value.mimeType || '';
  
  if (mimeType.startsWith('image/')) {
    previewType.value = 'image';
  } else if (mimeType.startsWith('video/')) {
    previewType.value = 'video';
  } else if (mimeType.startsWith('audio/')) {
    previewType.value = 'audio';
  } else if (mimeType === 'application/pdf') {
    previewType.value = 'pdf';
    // 初始化PDF预览
    initPdfPreview();
  } else if (
    mimeType.includes('spreadsheet') || 
    mimeType.includes('document') || 
    mimeType.includes('presentation') ||
    mimeType.includes('msword') || 
    mimeType.includes('officedocument')
  ) {
    previewType.value = 'office';
  } else if (
    mimeType.includes('text/') || 
    mimeType === 'application/json' ||
    mimeType === 'application/xml' ||
    mimeType === 'application/javascript'
  ) {
    previewType.value = 'text';
    loadTextContent();
  } else {
    previewType.value = 'other';
  }
}

// 加载文本内容
async function loadTextContent() {
  if (!file.value || !file.value.id) return;
  
  try {
    const response = await fetch(`/api/file/content/${file.value.id}`);
    
    if (!response.ok) {
      throw new Error(`服务器返回错误: ${response.status}`);
    }
    
    const data = await response.json();
    
    if (!data.success) {
      throw new Error(data.message || '获取文件内容失败');
    }
    
    fileContent.value = data.data || '';
  } catch (e: unknown) {
    console.error('加载文本内容失败:', e);
    fileContent.value = '无法加载文件内容: ' + (e instanceof Error ? e.message : '未知错误');
  }
}

// 初始化PDF预览
function initPdfPreview() {
  // PDF.js 初始化代码
  // 这里通常需要引入PDF.js库并进行相关配置
  // 由于Tauri应用的限制，可能需要特殊处理
  console.log('初始化PDF预览');
}

// 返回上一页
function goBack() {
  window.history.back();
}

// 上一页（PDF）
function prevPage() {
  if (pageNum.value <= 1) return;
  pageNum.value--;
}

// 下一页（PDF）
function nextPage() {
  if (pageNum.value >= totalPages.value) return;
  pageNum.value++;
}

// 放大（PDF）
function zoomIn() {
  scale.value += 0.25;
}

// 缩小（PDF）
function zoomOut() {
  if (scale.value <= 0.5) return;
  scale.value -= 0.25;
}

// 监听路由变化
watch(() => route.params.id, (newId) => {
  if (newId && newId !== fileId.value) {
    fileId.value = newId as string;
    loadFileInfo();
  }
});

// 组件挂载时加载文件信息
onMounted(() => {
  loadFileInfo();
  
  // 适配预览容器大小
  window.addEventListener('resize', resizePreviewContainer);
  resizePreviewContainer();
});

// 组件卸载时移除事件监听
onUnmounted(() => {
  window.removeEventListener('resize', resizePreviewContainer);
});

// 适配预览容器大小
function resizePreviewContainer() {
  const header = document.querySelector('.preview-header');
  const container = document.getElementById('preview-container');
  if (header && container) {
    container.style.height = `calc(100vh - ${header.clientHeight}px)`;
  }
}
</script>

<template>
  <div class="preview-page">
    <!-- 预览头部 -->
    <div class="preview-header">
      <div class="file-info">
        <button class="btn-back" @click="goBack">返回</button>
        <h4 v-if="file">{{ file.name }}</h4>
        <span v-if="file && file.size" class="file-size">{{ formatFileSize(file.size) }}</span>
      </div>
      <a v-if="file && file.id" :href="`/download/${file.id}`" class="btn-download" download>下载</a>
    </div>

    <!-- 加载状态 -->
    <div v-if="isLoading" class="preview-loading">
      <div class="loading-spinner"></div>
      <p>加载中...</p>
    </div>

    <!-- 错误信息 -->
    <div v-else-if="error" class="preview-error">
      <h5>加载失败</h5>
      <p>{{ error }}</p>
      <button @click="loadFileInfo" class="btn-retry">重试</button>
    </div>

    <!-- 预览容器 -->
    <div v-else class="preview-container" id="preview-container">
      <!-- 图片预览 -->
      <div v-if="previewType === 'image'" class="image-preview">
        <img :src="`/download/${fileId}`" :alt="file?.name" class="preview-image">
      </div>

      <!-- 视频预览 -->
      <div v-else-if="previewType === 'video'" class="video-preview">
        <video controls autoplay class="preview-video">
          <source :src="`/download/${fileId}`" :type="file?.mimeType">
          您的浏览器不支持视频预览
        </video>
      </div>

      <!-- 音频预览 -->
      <div v-else-if="previewType === 'audio'" class="audio-preview">
        <audio controls class="preview-audio">
          <source :src="`/download/${fileId}`" :type="file?.mimeType">
          您的浏览器不支持音频预览
        </audio>
      </div>

      <!-- PDF预览 -->
      <div v-else-if="previewType === 'pdf'" class="pdf-preview">
        <div id="pdf-viewer"></div>
        <div class="pdf-controls">
          <button @click="prevPage" class="pdf-control-btn">上一页</button>
          <span class="pdf-page-info">第 {{ pageNum }} 页，共 {{ totalPages }} 页</span>
          <button @click="nextPage" class="pdf-control-btn">下一页</button>
          <button @click="zoomIn" class="pdf-control-btn">放大</button>
          <button @click="zoomOut" class="pdf-control-btn">缩小</button>
        </div>
      </div>

      <!-- 文本预览 -->
      <div v-else-if="previewType === 'text'" class="text-preview">
        <pre class="preview-text">{{ fileContent }}</pre>
      </div>

      <!-- Office文档预览 -->
      <div v-else-if="previewType === 'office'" class="office-preview">
        <object class="browser-preview" :data="`/download/${fileId}`" :type="file?.mimeType">
          <div class="unsupported-file">
            <h5>浏览器无法直接预览此类型的Office文档</h5>
            <p>文件类型: {{ file?.mimeType || '未知' }}</p>
            <div class="download-hint">
              <a :href="`/download/${fileId}`" class="btn-download-local" download>
                下载到本地查看
              </a>
            </div>
            <div class="office-hint">
              <p><strong>提示：</strong>Office类型的文档通常需要安装相应的软件才能查看。</p>
            </div>
          </div>
        </object>
      </div>

      <!-- 其他类型文件预览 -->
      <div v-else class="other-preview">
        <object class="browser-preview" :data="`/download/${fileId}`" :type="file?.mimeType">
          <div class="unsupported-file">
            <h5>无法预览此类型的文件</h5>
            <p>文件类型: {{ file?.mimeType || '未知' }}</p>
            <a :href="`/download/${fileId}`" class="btn-download-local" download>下载文件</a>
          </div>
        </object>
      </div>
    </div>
  </div>
</template>

<style scoped>
.preview-page {
  width: 100%;
  height: 100vh;
  display: flex;
  flex-direction: column;
  background-color: #f5f5f5;
}

.preview-header {
  padding: 15px 20px;
  background-color: #fff;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  display: flex;
  justify-content: space-between;
  align-items: center;
  z-index: 10;
}

.file-info {
  display: flex;
  align-items: center;
}

.btn-back {
  margin-right: 15px;
  padding: 6px 12px;
  background-color: transparent;
  border: 1px solid #3b82f6;
  color: #3b82f6;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
}

.btn-back:hover {
  background-color: rgba(59, 130, 246, 0.1);
}

.file-size {
  margin-left: 10px;
  padding: 3px 8px;
  background-color: #e5e7eb;
  border-radius: 12px;
  font-size: 12px;
  color: #4b5563;
}

.btn-download {
  padding: 6px 12px;
  background-color: #10b981;
  color: white;
  border-radius: 4px;
  text-decoration: none;
  font-size: 14px;
}

.btn-download:hover {
  background-color: #059669;
}

.preview-container {
  flex: 1;
  display: flex;
  justify-content: center;
  align-items: center;
  overflow: auto;
}

.preview-loading,
.preview-error {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  color: #4b5563;
}

.loading-spinner {
  border: 3px solid rgba(0, 0, 0, 0.1);
  border-top: 3px solid #3b82f6;
  border-radius: 50%;
  width: 30px;
  height: 30px;
  animation: spin 1s linear infinite;
  margin-bottom: 15px;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.btn-retry {
  margin-top: 15px;
  padding: 6px 12px;
  background-color: #3b82f6;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

.btn-retry:hover {
  background-color: #2563eb;
}

/* 图片预览 */
.image-preview {
  width: 100%;
  height: 100%;
  display: flex;
  justify-content: center;
  align-items: center;
}

.preview-image {
  max-width: 90%;
  max-height: 90%;
  object-fit: contain;
}

/* 视频预览 */
.video-preview,
.audio-preview {
  width: 100%;
  height: 100%;
  display: flex;
  justify-content: center;
  align-items: center;
}

.preview-video,
.preview-audio {
  max-width: 90%;
}

/* PDF预览 */
.pdf-preview {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
}

#pdf-viewer {
  flex: 1;
  overflow: auto;
}

.pdf-controls {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 10px 0;
  background-color: #fff;
  border-top: 1px solid #e5e7eb;
}

.pdf-control-btn {
  margin: 0 5px;
  padding: 5px 10px;
  background-color: #f3f4f6;
  border: 1px solid #d1d5db;
  border-radius: 4px;
  font-size: 12px;
  cursor: pointer;
}

.pdf-page-info {
  margin: 0 10px;
  font-size: 14px;
  color: #4b5563;
}

/* 文本预览 */
.text-preview {
  width: 90%;
  height: 90%;
  display: flex;
  justify-content: center;
  align-items: center;
}

.preview-text {
  width: 100%;
  height: 100%;
  padding: 20px;
  background-color: white;
  overflow: auto;
  border-radius: 5px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  white-space: pre-wrap;
  font-family: monospace;
}

/* Office和其他文件预览 */
.office-preview,
.other-preview {
  width: 90%;
  height: 90%;
  background-color: white;
  border-radius: 5px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  overflow: hidden;
}

.browser-preview {
  width: 100%;
  height: 100%;
  border: none;
}

.unsupported-file {
  text-align: center;
  padding: 30px;
  background-color: white;
  border-radius: 5px;
}

.btn-download-local {
  display: inline-block;
  margin-top: 15px;
  padding: 8px 15px;
  background-color: #3b82f6;
  color: white;
  text-decoration: none;
  border-radius: 4px;
}

.btn-download-local:hover {
  background-color: #2563eb;
}

.office-hint {
  margin-top: 20px;
  padding: 10px;
  background-color: #f0f9ff;
  border-left: 4px solid #38bdf8;
  text-align: left;
}
</style>