import Clipboard from "clipboard";
import { ElMessage } from "element-plus";

export const copyClipboard = (text, event) => {
  const clipboard = new Clipboard(event.target, {
    text: () => text
  });
  clipboard.on("success", () => {
    ElMessage.success({ message: "复制成功", type: "success" });
  });
  clipboard.onClick(event);
};
