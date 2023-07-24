<template>
  <div class="upload-box">
    <el-upload
      ref="uploadFile"
      accept=""
      drag
      multiple
      :show-file-list="false"
      action=""
      :http-request="addFiles"
    >
      <i class="el-icon-upload"></i>
      <div class="el-upload__text">
        拖拽<b>文件</b>或<b>文件夹</b>到此处或点击<em>选择文件</em>，进行分享~
      </div>
    </el-upload>
  </div>
  <div v-for="file in files" :key="file" class="text item file-item">
    <el-row class="row-bg" justify="space-between">
      <el-col :span="5">
        <el-tooltip effect="light" placement="top">
          <template #content>{{ `由【${file.username}】分享` }}</template>
          <span class="username">{{ file.username }}</span>
        </el-tooltip>
      </el-col>
      <el-col :span="13">
        <el-tooltip v-if="file.type === 'text'" effect="light" placement="top">
          <template #content>{{ file.intro }}</template>
          <span>{{ file.name }}</span>
        </el-tooltip>
        <span v-if="['directory', 'file'].includes(file.type)">{{
          file.name
        }}</span>
      </el-col>
      <el-col :span="6">
        <el-button
          v-if="file.type === 'text'"
          type="default"
          size="mini"
          icon="el-icon-document-copy"
          title="复制文本到剪切板"
          @click="handleClipboard(file.content, $event)"
        ></el-button>
        <el-button
          v-if="['directory', 'file'].includes(file.type)"
          type="default"
          size="mini"
          icon="el-icon-search"
          title="打开文件所在目录"
          @click="openFile(file.name, $event)"
        >
        </el-button>
        <el-button
          type="default"
          size="mini"
          icon="el-icon-delete"
          @click="() => removeFile(file)"
        ></el-button>
      </el-col>
    </el-row>
  </div>
  <el-alert
    v-if="files.length === 0"
    title="无"
    :closable="false"
    type="info"
    center
  >
  </el-alert>
</template>

<script setup>
import { computed } from "vue";
import { ElMessage } from "element-plus";
import { copyClipboard } from "@/views/copy";
import { useStore } from "vuex";

let files = computed({
  get() {
    return store.state.files;
  },
  set(value) {
    store.commit("setFiles", value);
  }
});
let api = window.api;
let store = useStore();

let handleClipboard = function(data, event) {
  copyClipboard(data, event);
};
// eslint-disable-next-line no-unused-vars
let openFile = function(filename, $event) {
  api.openFile(filename, err => {
    ElMessage.error({ message: `文件打开失败 "${err}"`, type: "error" });
  });
};

let removeFile = function(file) {
  let removeFiles = this.files.filter(f => f.name === file.name);
  api.removeFile(removeFiles[0]);
  files.value = api.listFiles();
};

let settingForm = computed(() => {
  return store.state.settingForm;
});

let addFiles = function(params) {
  let file = {
    name: params.file.name,
    path: params.file.path,
    username: settingForm.value.ip
  };
  let { success, message } = api.addFile(file);
  if (success) {
    store.commit("setFiles", api.listFiles());
  } else {
    ElMessage.error(message);
  }
};
</script>

<style scoped></style>
