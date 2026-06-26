import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'sbox',
  description: 'sbot 配套桌面工具箱',
  outDir: '../docs',
  // 文档站点位于 Pages 子路径 /sbox/docs/（站点根由 web 工具站占用）
  base: '/sbox/docs/',
  head: [
    ['link', { rel: 'icon', href: '/sbox/docs/logo.svg' }],
  ],
  themeConfig: {
    logo: '/logo.svg',
    socialLinks: [
      { icon: 'github', link: 'https://github.com/while-coder/sbox' },
    ],
    search: {
      provider: 'local',
      options: {
        locales: {
          zh: {
            translations: {
              button: {
                buttonText: '搜索文档',
                buttonAriaLabel: '搜索文档',
              },
              modal: {
                noResultsText: '无法找到相关结果',
                resetButtonTitle: '清除查询条件',
                footer: {
                  selectText: '选择',
                  navigateText: '切换',
                  closeText: '关闭',
                },
              },
            },
          },
        },
      },
    },
  },
  locales: {
    root: {
      label: 'English',
      lang: 'en-US',
      title: 'sbox',
      description: 'Companion desktop toolbox for sbot',
      themeConfig: {
        nav: [
          { text: 'Guide', link: '/guide/getting-started' },
          { text: 'GitHub', link: 'https://github.com/while-coder/sbox' },
        ],
        sidebar: [
          {
            text: 'Introduction',
            items: [
              { text: 'Getting Started', link: '/guide/getting-started' },
            ],
          },
          {
            text: 'Tools',
            items: [
              { text: 'XiaoAI Login', link: '/guide/xiaoai-login' },
              { text: 'Keystore Generator', link: '/guide/keystore-gen' },
              { text: 'Codec', link: '/guide/codec' },
            ],
          },
        ],
        footer: {
          message: 'Released under the MIT License.',
          copyright: 'Copyright © sbox contributors',
        },
      },
    },
    zh: {
      label: '简体中文',
      lang: 'zh-CN',
      title: 'sbox',
      description: 'sbot 配套桌面工具箱',
      themeConfig: {
        nav: [
          { text: '指南', link: '/zh/guide/getting-started' },
          { text: 'GitHub', link: 'https://github.com/while-coder/sbox' },
        ],
        sidebar: [
          {
            text: '介绍',
            items: [
              { text: '快速开始', link: '/zh/guide/getting-started' },
            ],
          },
          {
            text: '工具',
            items: [
              { text: '小爱登录', link: '/zh/guide/xiaoai-login' },
              { text: 'Keystore 生成', link: '/zh/guide/keystore-gen' },
              { text: '编解码', link: '/zh/guide/codec' },
            ],
          },
        ],
        docFooter: {
          prev: '上一页',
          next: '下一页',
        },
        outline: {
          label: '本页目录',
        },
        lastUpdated: {
          text: '最后更新于',
        },
        darkModeSwitchLabel: '主题',
        lightModeSwitchTitle: '切换到浅色模式',
        darkModeSwitchTitle: '切换到深色模式',
        sidebarMenuLabel: '菜单',
        returnToTopLabel: '回到顶部',
        langMenuLabel: '切换语言',
        footer: {
          message: '基于 MIT 许可证发布。',
          copyright: 'Copyright © sbox contributors',
        },
      },
    },
  },
})
