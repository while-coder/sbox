<template>
  <div class="home-container">
    <div class="home-card">
      <img src="/logo.svg" alt="sbox" class="home-logo" />
      <h1 class="home-title">sbox</h1>
      <p class="home-tagline">{{ t.tagline }}</p>
      <p class="home-desc">{{ t.desc }}</p>
      <div class="home-actions">
        <a :href="t.getStartedLink" class="home-btn home-btn-primary">{{ t.getStarted }}</a>
        <a href="https://github.com/while-coder/sbox" target="_blank" class="home-btn home-btn-secondary">GitHub</a>
      </div>
      <ul class="home-highlights">
        <li v-for="(item, i) in t.highlights" :key="i" v-html="item"></li>
      </ul>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useData } from 'vitepress'

const { lang } = useData()

const messages = {
  'en-US': {
    tagline: 'Companion desktop toolbox for sbot',
    desc: 'A small Tauri + Vue 3 desktop app bundling the chores you hit while setting up sbot — log in to XiaoAI for channel credentials, generate an Android signing keystore, and convert / hash text. Run it locally, copy the output, paste into admin.',
    getStarted: 'Get Started →',
    getStartedLink: '/sbox/guide/getting-started',
    highlights: [
      '<strong>XiaoAI Login</strong> — sign in to your Mi account and export the credentials sbot <code>channel.xiaoai</code> needs',
      '<strong>Keystore Generator</strong> — create a PKCS12 Android signing keystore and the 4 GitHub Actions secrets',
      '<strong>Codec</strong> — Base64 · URL · Hex · HTML · Unicode · JSON, MD5 & SHA hashes, UUID, timestamps, byte sizes',
      '<strong>Cross-platform</strong> — Windows · macOS · Linux, built with Tauri 2',
    ],
  },
  'zh-CN': {
    tagline: 'sbot 配套桌面工具箱',
    desc: '基于 Tauri + Vue 3 的小桌面应用，集合了配置 sbot 时常用的杂活 —— 登录小爱拿渠道凭据、生成 Android 签名 keystore、做编解码与哈希。本地运行，复制输出，粘贴到 admin 即可。',
    getStarted: '快速开始 →',
    getStartedLink: '/sbox/zh/guide/getting-started',
    highlights: [
      '<strong>小爱登录</strong> — 登录小米账号，导出 sbot <code>channel.xiaoai</code> 所需的凭据',
      '<strong>Keystore 生成</strong> — 生成 PKCS12 Android 签名 keystore 与 GitHub Actions 所需的 4 个 secret',
      '<strong>编解码工具</strong> — Base64 · URL · Hex · HTML · Unicode · JSON，MD5 与 SHA 哈希，UUID、时间戳与字节大小',
      '<strong>跨平台</strong> — Windows · macOS · Linux，基于 Tauri 2 构建',
    ],
  },
}

const t = computed(() => messages[lang.value as keyof typeof messages] || messages['en-US'])
</script>
