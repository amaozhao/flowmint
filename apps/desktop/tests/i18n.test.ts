import {
  createTranslator,
  defaultLocale,
  normalizeLocale,
} from "../src/i18n/messages";

function assert(condition: boolean, message: string) {
  if (!condition) {
    throw new Error(message);
  }
}

assert(defaultLocale === "en", "default locale is English");
assert(normalizeLocale("zh") === "zh", "Chinese locale is accepted");
assert(normalizeLocale("en") === "en", "English locale is accepted");
assert(normalizeLocale("fr") === "en", "unknown locale falls back to English");

const tEn = createTranslator("en");
const tZh = createTranslator("zh");

assert(tEn("nav.assets") === "Assets", "English nav label is available");
assert(tZh("nav.assets") === "资产", "Chinese nav label is available");
assert(tEn("nav.syncHistory") === "Sync", "Sync nav label does not promise history");
assert(
  tEn("counts.items", { count: 2 }) === "2 items",
  "English interpolation works",
);
assert(
  tZh("counts.items", { count: 2 }) === "2 项",
  "Chinese interpolation works",
);
