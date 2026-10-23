<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { Box, Github } from 'lucide-vue-next'
import { native } from '../platform/native'
const appVersion = ref('')
const aboutError = ref('')
onMounted(async () => { appVersion.value = await native.getAppVersion() })
async function openGitHub() {
  aboutError.value = ''
  try { await native.openExternalUrl('https://github.com/while-coder/sbox') }
  catch (error) { aboutError.value = `无法打开 GitHub：${String(error)}` }
}
</script>
<template>
<section class="about">
      <header class="about-hero">
        <div class="about-mark" aria-hidden="true"><Box :size="30" stroke-width="1.8" /></div>
        <div class="about-copy">
          <h2>sbox</h2>
          <p>本地桌面工具箱</p>
        </div>
        <span class="version-badge">v{{ appVersion || '…' }}</span>
        <p class="about-description">集中提供编码、校验、图像处理、截图和系统辅助工具，让日常操作少一些重复步骤。</p>
      </header>

      <div class="about-grid">
        <slot name="update" />

        <section class="about-panel" aria-labelledby="github-heading">
          <div class="about-panel-icon github-icon" aria-hidden="true"><Github :size="19" stroke-width="1.8" /></div>
          <div class="about-panel-copy">
            <h3 id="github-heading">GitHub 与使用说明</h3>
            <p>查看源码、使用说明、更新记录或提交问题反馈。</p>
          </div>
          <button type="button" class="btn btn-outline about-action" @click="openGitHub">查看详情</button>
        </section>
      </div>
      <p v-if="aboutError" class="about-error" role="alert">{{ aboutError }}</p>
        </section>
</template>
<style scoped>
.about { margin-top: 4px; }
.about-hero {
  display: grid; grid-template-columns: auto minmax(0, 1fr) auto; gap: 12px; align-items: center;
  padding: 24px; border: 1px solid var(--border); border-radius: calc(var(--radius) + 2px);
  background: linear-gradient(135deg, color-mix(in srgb, var(--primary) 12%, var(--card)), var(--card) 56%);
}
.about-mark {
  display: grid; place-items: center;
  width: 54px; height: 54px; border-radius: 14px; background: var(--primary); color: #fff;
  box-shadow: 0 8px 20px color-mix(in srgb, var(--primary) 28%, transparent);
}
.about-copy { min-width: 0; }
.about-copy h2 { margin: 0; font-size: 22px; line-height: 1.2; }
.about-copy p { margin: 4px 0 0; color: var(--fg-muted); font-size: 13px; }
.version-badge { padding: 4px 8px; border: 1px solid var(--border); border-radius: 99px; background: var(--card); color: var(--fg-muted); font: 12px ui-monospace, monospace; }
.about-description { grid-column: 1 / -1; margin: 5px 0 0; color: var(--fg-muted); font-size: 13px; }
.about-grid { display: grid; gap: 12px; margin-top: 14px; }
:deep(.about-panel) { display: grid; grid-template-columns: auto minmax(0, 1fr) auto; gap: 12px; align-items: center; padding: 16px; border: 1px solid var(--border); border-radius: var(--radius); background: var(--card); }
:deep(.about-panel-icon) { display: grid; width: 38px; height: 38px; place-items: center; border-radius: 9px; color: var(--primary); background: color-mix(in srgb, var(--primary) 12%, var(--card)); }
.github-icon { color: var(--fg); background: var(--bg); }
:deep(.about-panel-copy) { min-width: 0; }
:deep(.about-panel-copy h3) { margin: 0; font-size: 14px; font-weight: 600; }
:deep(.about-panel-copy p) { margin: 4px 0 0; color: var(--fg-muted); font-size: 12px; }
:deep(.about-action) { flex: 0 0 auto; min-width: 96px; }
.row-desc.update-error { color: var(--danger); }
:deep(.about-panel-copy .update-error) { color: var(--danger); }
.about-error { margin: 0 0 16px; color: var(--danger); font-size: 12px; }

@media (max-width: 560px) {
  .about-hero { grid-template-columns: auto minmax(0, 1fr); padding: 20px; }
  .version-badge { justify-self: start; }
  :deep(.about-panel) { grid-template-columns: auto minmax(0, 1fr); }
  :deep(.about-action) { grid-column: 1 / -1; width: 100%; }
}
</style>
