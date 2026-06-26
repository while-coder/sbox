<script setup lang="ts">
import { useRoute, useRouter } from 'vue-router'
import { computed } from 'vue'

const route = useRoute()
const router = useRouter()
const isHome = computed(() => route.path === '/')
</script>

<template>
  <div class="app">
    <header class="app-bar">
      <button v-if="!isHome" class="back" @click="router.push('/')">← 返回</button>
      <h1 class="title">sbox</h1>
      <a class="repo" href="https://github.com/" target="_blank" rel="noreferrer" title="源码">GitHub</a>
    </header>
    <main class="app-main">
      <router-view />
    </main>
  </div>
</template>

<style scoped>
.app { display: flex; flex-direction: column; height: 100%; }
.app-bar {
  display: flex; align-items: center; gap: 12px;
  padding: 12px 20px;
  border-bottom: 1px solid var(--border);
  background: var(--card);
}
.title { font-size: 16px; font-weight: 600; margin: 0; }
.back {
  background: none; border: none; color: var(--primary); cursor: pointer;
  font-size: 14px; padding: 4px 8px;
}
.back:hover { color: var(--primary-hover); }
.repo {
  margin-left: auto; color: var(--fg-muted); text-decoration: none; font-size: 13px;
}
.repo:hover { color: var(--primary); }
.app-main { flex: 1; overflow: auto; padding: 20px; }
</style>
