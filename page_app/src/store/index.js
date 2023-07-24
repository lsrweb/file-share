import { createStore } from "vuex";

const store = createStore({
  state() {
    return {
      files: [],

      serverStatus: "stop",
      settingForm: {
        url: "",
        uploadPath: "",
        port: 5421
      },

      ipFamily: "ipv4",
      currentNetInterfaceIdx: 0,
      netInterfaceNames: [],
      currentInterfaceName: ""
    };
  },
  mutations: {
    setFiles(state, files) {
      state.files = files;
    },
    setServerStatus(state, status) {
      state.serverStatus = status;
    },
    setSettingForm(state, form) {
      state.settingForm = form;
    },
    setStateKey(state, { key, value }) {
      state[key] = value;
    }
  }
});

export default store;
