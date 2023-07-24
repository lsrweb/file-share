<template>
  <div class="body">
    <div class="container">
      <div v-if="serverStatus === 'start'">
        <el-space size="large" direction="vertical">
          <Head />
          <el-card class="box-card">
            <template #header>
              <el-dialog v-model="dialogFormVisible" title="分享一段文本">
                <el-input
                  type="textarea"
                  :rows="2"
                  :autosize="{ minRows: 2, maxRows: 4 }"
                  placeholder="请输入内容"
                  v-model="form.text"
                >
                </el-input>
                <template #footer>
                  <el-button type="primary" @click="formSubmit">提交</el-button>
                </template>
              </el-dialog>

              <el-dialog
                v-model="settingFormVisible"
                title="设置"
                style="width: 70%;"
              >
                <el-row class="row-bg">
                  <el-col :span="24">
                    <el-form ref="form" :model="settingForm" label-width="80px">
                      <el-form-item label="上传路径">
                        <el-input v-model="settingForm.uploadPath"></el-input>
                      </el-form-item>
                      <el-form-item label="服务端口">
                        <el-input v-model="settingForm.port"></el-input>
                      </el-form-item>
                      <el-form-item label="密码认证">
                        <el-switch v-model="settingForm.authEnable">
                        </el-switch>
                      </el-form-item>
                      <el-form-item
                        v-if="settingForm.authEnable"
                        label="校验密码"
                      >
                        <el-input
                          v-model="settingForm.password"
                          show-password
                        ></el-input>
                      </el-form-item>
                      <el-form-item label="启用续传">
                        <el-switch v-model="settingForm.tusEnable"> </el-switch>
                      </el-form-item>
                      <el-form-item
                        v-if="settingForm.tusEnable"
                        label="分片大小"
                      >
                        <el-input v-model="settingForm.chunkSize">
                          <template #append><span>M</span></template>
                        </el-input>
                      </el-form-item>
                    </el-form>
                  </el-col>
                </el-row>
                <template #footer>
                  <el-button type="primary" @click="updateSettingsForm"
                    >更新</el-button
                  >
                  <el-button type="primary" @click="closeSettingsForm"
                    >取消</el-button
                  >
                </template>
              </el-dialog>

              <div class="card-header">
                <el-row class="row-bg">
                  <el-col :span="12">分享列表</el-col>
                  <el-col :span="4">
                    <el-button
                      @click="dialogFormVisible = true"
                      type="default"
                      size="mini"
                      icon="el-icon-message"
                      title="分享一段文本"
                      >分享文本
                    </el-button>
                  </el-col>
                  <el-col :span="4">
                    <el-popconfirm
                      @confirm="removeFileAll()"
                      title="确定要清空所有文件吗？"
                    >
                      <template #reference>
                        <el-button
                          type="default"
                          size="mini"
                          icon="el-icon-delete"
                          title="清空列表"
                          >清空列表
                        </el-button>
                      </template>
                    </el-popconfirm>
                  </el-col>
                  <el-col :span="4">
                    <el-button
                      @click="onHandlerSetting()"
                      type="default"
                      size="mini"
                      icon="el-icon-setting"
                      title="设置"
                      >设置
                    </el-button>
                  </el-col>
                </el-row>
              </div>
            </template>
            <List />
          </el-card>
        </el-space>
      </div>
      <start v-if="serverStatus === 'stop'"></start>
    </div>
  </div>
</template>

<script>
import { ElMessage } from "element-plus";
import Start from "@/views/start.vue";
import Head from "@/views/head.vue";
import List from "@/views/list.vue";
import store from "@/store";
import { currentInterfaceName, netInterfaceNames } from "@/views/store";

let api = window.api;

export default {
  name: "App",
  data: () => {
    return {
      qrcode: "",
      ipFamily: "ipv4",
      netInterfaceNames: [],
      currentNetInterfaceIdx: 0,
      currentInterfaceName: "",
      form: {
        text: ""
      },
      settingFormVisible: false,

      dialogFormVisible: false,
      timer: null
    };
  },
  computed: {
    serverStatus: {
      get() {
        return this.$store.state.serverStatus;
      },
      set(value) {
        this.$store.commit("setServerStatus", value);
      }
    },
    settingForm: {
      get() {
        return this.$store.state.settingForm;
      },
      set(value) {
        this.$store.commit("setSettingForm", value);
      }
    },
    files: {
      get() {
        return this.$store.state.files;
      },
      set(value) {
        this.$store.commit("setFiles", value);
      }
    }
  },
  methods: {
    onHandlerSetting: function() {
      console.log("--onHandlerSetting--");
      this.settingForm = api.getSetting();
      this.settingFormVisible = true;
    },
    updateSettingsForm: function() {
      console.log(this.settingForm);
      api
        .updateSetting(this.settingForm)
        .then(() => {
          ElMessage.success("更新成功");
          this.settingForm = api.getSetting();
          this.settingFormVisible = false;
        })
        .catch(() => {
          ElMessage.error("更新失败");
          this.settingForm = api.getSetting();
          this.settingFormVisible = false;
        });
    },
    closeSettingsForm: function() {
      this.settingForm = api.getSetting();
      this.settingFormVisible = false;
    },
    formSubmit: function() {
      let text = this.form.text;
      api.addText(text, this.settingForm.ip);
      this.files = api.listFiles();
      this.form.text = "";
      this.dialogFormVisible = false;
    },

    removeFileAll: function() {
      this.files.forEach(f => {
        api.removeFile(f);
      });
      this.files = api.listFiles();
      ElMessage.success({ message: "已清空列表", type: "success" });
    },

    updatePage() {
      store.commit("setServerStatus", api.getServerStatus());
      netInterfaceNames.value = api.getNetInterfaceNames(this.ipFamily);
      currentInterfaceName.value = netInterfaceNames.value[0] || "";
      store.commit("setFiles", api.listFiles());
      store.commit("setSettingForm", api.getSetting());
      console.log("---settingForm--", this.settingForm);
    }
  },
  mounted: function() {
    try {
      // document.getElementsByClassName('el-upload__input')[0].webkitdirectory = true
      // 注册事件监听
      api.registryEventListener("server.statusChange", event => {
        console.log("---服务状态变更---", event);
        this.serverStatus = event.data.status;
      });
      api.registryEventListener("fileDb.listChange", event => {
        console.log("---文件列表变更---", event);
        let files = api.listFiles();
        store.commit("setFiles", files);
      });

      this.updatePage();
    } catch (e) {
      console.log(e);
    }
  },
  beforeUnmount() {
    clearInterval(this.timer);
    this.timer = null;
  },
  components: { List, Head, Start }
};
</script>

<style>
body {
  margin: 0;
}

.body {
  font-family: "Helvetica Neue", Helvetica, "PingFang SC", "Hiragino Sans GB",
    "Microsoft YaHei", "微软雅黑", Arial, sans-serif;
  background-image: linear-gradient(to top, #fbc2eb 0%, #a6c1ee 100%);
  background-blend-mode: screen, overlay, hard-light, color-burn, color-dodge,
    normal;
  background-attachment: fixed;
  background-repeat: no-repeat;
  background-position: 0 100%;
  min-height: 600px;
  position: absolute;
  width: 100%;
  height: 100%;
}

.container {
  max-width: 700px;
  margin: 0 auto;
  padding-top: 18px;
}

.upload-box {
  display: flex;
  justify-content: center;
  align-items: center;
}

.el-upload-dragger .el-icon-upload {
  font-size: 46px !important;
  margin: 0 !important;
}

.el-upload-dragger {
  height: 90px !important;
  width: 500px !important;
  margin-bottom: 16px;
}

.file-item {
  margin-bottom: 10px;
}

.username {
  font-size: 12px;
  margin-left: 2px;
  color: #909399;
}

.box-card {
  width: 700px;
}

.row-bg {
  align-items: center;
}

.btn-box {
  width: 150px;
  height: 150px;
}

.el-popover.el-popper {
  min-width: 100px !important;
}

.start-btn {
  text-align: center;
  line-height: 100px;
  width: 100px;
  height: 100px;
  border-radius: 50%;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.12), 0 0 6px rgba(0, 0, 0, 0.04);
  z-index: 99;
  position: absolute;
  top: 150px;
  left: 50%;
  margin-top: -50px;
  margin-left: -50px;
  background-color: #409eff;
  color: #ffffff;
}

.start-btn:hover {
  cursor: pointer;
  background-color: #66b1ff;
}

.start-btn-shadow {
  width: 100px;
  height: 100px;
  display: flex;
  justify-content: center;
  align-items: center;
  position: absolute;
  top: 150px;
  left: 50%;
  margin-top: -50px;
  margin-left: -50px;
  z-index: 1;
}

.start-btn-shadow span {
  z-index: 1;
  position: absolute;
  box-sizing: border-box;
  border: 2px solid #ffffff;
  border-radius: 50%;
  animation: animate 2s linear infinite;
  animation-delay: calc(0.5s * var(--i));
}

.start-btn-shadow:nth-child(2) span {
  border: none;
  background-color: #f5f5f5;
  opacity: 0.8;
}

@keyframes animate {
  0% {
    width: 50px;
    height: 50px;
  }

  50% {
    opacity: 0.5;
  }

  100% {
    width: 200px;
    height: 200px;
    opacity: 0;
  }
}
</style>
