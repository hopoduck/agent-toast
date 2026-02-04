import { createI18n } from "vue-i18n";
import ko from "./locales/ko.json";
import en from "./locales/en.json";

const i18n = createI18n({
  legacy: false,
  locale: "ko",
  fallbackLocale: "ko",
  messages: { ko, en },
});

export default i18n;
