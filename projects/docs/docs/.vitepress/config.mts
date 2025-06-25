import { defineConfig } from "vitepress";

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "ZEL.ZIP",
  description: "Shared documentation for all the ZEL.ZIP projects",
  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    nav: [
      { text: "Home", link: "/" },
      { text: "Visit the main page", link: "https://zel.zip" },
    ],

    sidebar: {
      "/": [
        {
          text: "Projects",
          items: [
            { text: "NiiEBLA library", link: "/niiebla/niiebla" },
            { text: "ReNUS library", link: "/renus" },
            { text: "ViiENTO CLI", link: "/viiento" },
          ],
        },
      ],

      "/niiebla/": [
        {
          text: "The NiiEBLA library",
          items: [
            { text: "Getting Started", link: "/niiebla/niiebla" },
            { text: "WAD/TAD files", link: "/niiebla/wad" },
            { text: "Title IDs", link: "/niiebla/title_ids" },
          ],
        },
      ],
    },

    socialLinks: [
      { icon: "github", link: "https://github.com/vuejs/vitepress" },
    ],
  },
});
