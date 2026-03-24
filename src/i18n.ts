import { createI18n } from "vue-i18n";
import ko from "./locales/ko.json";
import en from "./locales/en.json";

const browserLang = navigator.language.startsWith("ko") ? "ko" : "en";

const i18n = createI18n({
  legacy: false,
  locale: browserLang,
  fallbackLocale: "ko",
  messages: { ko, en },
});

export default i18n;
