import { defineConfig } from "vitepress";

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "ZELZIP",
  description: "Shared documentation for all the ZELZIP projects",
  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    nav: [
      { text: "Home", link: "/" },
      { text: "Visit the main page", link: "https://zelzip.dev" },
    ],

    sidebar: {
      "/": [
        {
          text: "Projects",
          items: [
            { text: "NiiEBLA library", link: "/niiebla/niiebla" },
            { text: "Wiki", link: "/wiki/wiki" },
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

      "/wiki/": [
        {
          text: "ZELZIP Wiki",
          items: [
            { text: "About this wiki", link: "/wiki/wiki" },
            {
              text: "Parental Control Master Key Generation Algorithms",
              link: "/wiki/parental-control-master-key-generation-algorithms",
            },
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
