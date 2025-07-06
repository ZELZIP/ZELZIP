import js from "@eslint/js";
import globals from "globals";
import tseslint from "typescript-eslint";
import json from "@eslint/json";
import css from "@eslint/css";
import gitignore from "eslint-config-flat-gitignore";
import eslintPluginAstro from "eslint-plugin-astro";

import { defineConfig } from "eslint/config";

export default defineConfig([
  gitignore(),
  tseslint.configs.recommended,

  // TODO(TRACK: https://github.com/ota-meshi/eslint-plugin-astro/issues/485): Workaround for Astro plugin incompatibility
  ...eslintPluginAstro.configs.recommended.filter((conf) => conf.files),
  {
    extends: [
      ...eslintPluginAstro.configs.recommended.filter((conf) => !conf.files),
    ],
    files: ["**/*.astro"],
    rules: {
      "unicorn/prefer-module": ["off"],
    },
  },

  {
    files: ["**/*.astro"],
    rules: {},
  },

  {
    files: ["**/*.{js,mjs,cjs,ts,mts,cts}"],
    plugins: { js },
    extends: ["js/recommended"],
    rules: {
      "no-unused-vars": "off",
      "@typescript-eslint/no-unused-vars": "error",
    },
  },

  {
    files: ["**/*.{js,mjs,cjs,ts,mts,cts}"],
    languageOptions: { globals: { ...globals.browser, ...globals.node } },
  },

  {
    files: ["**/*.json"],
    plugins: { json },
    language: "json/json",
    extends: ["json/recommended"],
  },

  {
    files: ["**/*.jsonc"],
    plugins: { json },
    language: "json/jsonc",
    extends: ["json/recommended"],
  },

  {
    files: ["**/*.json5"],
    plugins: { json },
    language: "json/json5",
    extends: ["json/recommended"],
  },

  {
    files: ["**/*.css"],
    plugins: { css },
    language: "css/css",
    extends: ["css/recommended"],
  },
]);
