import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import "pretendard/dist/web/variable/pretendardvariable-dynamic-subset.css";
import { createApp } from "vue";
import App from "./App.vue";
import "./global.css";
import i18n from "./i18n";
import Setup from "./Setup.vue";

const label = getCurrentWebviewWindow().label;
const component = label === "setup" ? Setup : App;
createApp(component).use(i18n).mount("#app");
