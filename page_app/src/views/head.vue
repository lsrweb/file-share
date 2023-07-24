<script setup>
import QrcodeVue from "qrcode.vue";
import { useStore } from "vuex";
import { computed } from "vue";
import { ElMessage } from "element-plus";
import { copyClipboard } from "@/views/copy";
import {
  currentInterfaceName,
  currentNetInterfaceIdx,
  ipFamily,
  netInterfaceNames
} from "@/views/store";

let api = window.api;
let store = useStore();

let settingForm = computed({
  get: () => {
    return store.state.settingForm;
  },
  set: val => {
    store.commit("setSettingForm", val);
  }
});
const stopServer = () => {
  api.stopServer();
};

let changeIpFamily = function() {
  ipFamily.value = ipFamily.value === "ipv4" ? "ipv6" : "ipv4";
  currentNetInterfaceIdx.value = 0;
  netInterfaceNames.value = api.getNetInterfaceNames(ipFamily.value);
  currentInterfaceName.value = netInterfaceNames.value[0] || "";
  updateIp();
};

let updateIp = () => {
  let ip = api.getIpAddress(currentNetInterfaceIdx.value, ipFamily.value);
  if (ipFamily.value === "ipv6") {
    ip = `[${ip}]`;
  }
  api.updateIp(ip).then(() => {
    settingForm.value = api.getSetting();
    ElMessage.success({
      message: `切换网卡为 "${currentInterfaceName.value}"`,
      type: "success"
    });
  });
};

let handleClipboard = function(data, event) {
  copyClipboard(data, event);
};

let changeNetInterface = function() {
  if (currentNetInterfaceIdx.value === 99) {
    currentNetInterfaceIdx.value = 0;
  } else {
    currentNetInterfaceIdx.value = currentNetInterfaceIdx.value + 1;
  }
  currentInterfaceName.value =
    netInterfaceNames.value[
      currentNetInterfaceIdx.value % netInterfaceNames.value.length
    ];
  updateIp();
};
</script>

<template>
  <el-card class="box-card">
    <template #header>
      <div class="card-header">
        <el-row class="row-bg" justify="space-between">
          <el-col :span="6">正在分享...</el-col>
          <el-col :span="6">
            <el-button type="danger" size="small" @click="stopServer" plain
              >取消分享</el-button
            >
          </el-col>
        </el-row>
      </div>
    </template>
    <el-row class="row-bg">
      <el-col :span="12">分享链接：{{ settingForm.url }}</el-col>
      <el-col :span="4">
        <el-popover placement="left" :width="100" trigger="hover">
          <template #reference>
            <el-button
              type="default"
              size="mini"
              icon="el-icon-document-copy"
              title="复制链接到剪切板"
              @click="handleClipboard(settingForm.url, $event)"
              >复制链接
            </el-button>
          </template>
          <qrcode-vue :value="settingForm.url"></qrcode-vue>
        </el-popover>
      </el-col>
      <el-col :span="4">
        <el-button
          type="default"
          size="mini"
          icon="el-icon-sort"
          title="切换ip协议"
          @click="changeIpFamily()"
          >协议：{{ ipFamily }}
        </el-button>
      </el-col>
      <el-col :span="4">
        <el-button
          v-if="netInterfaceNames.length > 1"
          type="default"
          size="mini"
          icon="el-icon-sort"
          title="切换网卡"
          @click="changeNetInterface()"
          >网卡：{{ currentInterfaceName }}
        </el-button>
      </el-col>
    </el-row>
  </el-card>
</template>

<style scoped></style>
