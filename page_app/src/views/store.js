import { computed } from "vue";
import store from "@/store";

export let ipFamily = computed({
  get: () => {
    return store.state.ipFamily;
  },
  set: val => {
    store.commit("setStateKey", {
      key: "ipFamily",
      value: val
    });
  }
});
export let currentNetInterfaceIdx = computed({
  get: () => {
    return store.state.currentNetInterfaceIdx;
  },
  set: val => {
    store.commit("setStateKey", {
      key: "currentNetInterfaceIdx",
      value: val
    });
  }
});
export let currentInterfaceName = computed({
  get: () => {
    return store.state.currentInterfaceName;
  },
  set: val => {
    store.commit("setStateKey", {
      key: "currentInterfaceName",
      value: val
    });
  }
});
export let netInterfaceNames = computed({
  get: () => {
    return store.state.netInterfaceNames;
  },
  set: val => {
    store.commit("setStateKey", {
      key: "netInterfaceNames",
      value: val
    });
  }
});
