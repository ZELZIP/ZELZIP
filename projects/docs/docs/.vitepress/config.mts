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
            { text: "Seto database", link: "/seto" },
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

    editLink: {
      pattern:
        "https://github.com/ZELZIP/ZELZIP/edit/main/projects/docs/docs/:path",
      text: "Edit this page on GitHub",
    },

    search: {
      provider: "local",
    },

    socialLinks: [{ icon: "github", link: "https://github.com/ZELZIP/ZELZIP" }],

    footer: {
      message:
        "This project is a fan-made homebrew creation developed independently and is not affiliated with, endorsed by, or associated with Nintendo Co., Ltd or any of its subsidiaries, affiliates, or partners. All trademarks and copyrights referenced are the property of their respective owners.",
      copyright:
        'All text presented here is under the <a href="https://www.mozilla.org/en-US/MPL/2.0/">Mozilla Public License Version 2.0</a> otherwise noted.',
    },
  },
});
