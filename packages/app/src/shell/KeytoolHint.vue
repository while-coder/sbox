<script setup lang="ts">
/** keytool 依赖检测提示：缺失时引导用户下载 JDK，存在时显示路径。 */
defineProps<{
  available: boolean | null
  path: string
}>()
</script>

<template>
  <p v-if="available === false" class="warn">
    未检测到 <code>keytool</code>。请先安装 JDK 17+（推荐 <a href="https://adoptium.net/" target="_blank" rel="noopener">Adoptium Temurin</a>），然后重启应用。
  </p>
  <p v-else-if="available === true" class="hint">
    检测到 keytool: <code>{{ path }}</code>
  </p>
</template>

<style scoped>
.warn {
  background: var(--card); border: 1px solid var(--warning, #d97706);
  border-radius: var(--radius); padding: 10px 12px; font-size: 13px;
  color: var(--warning, #d97706); margin: 12px 0;
}
.hint { font-size: 12px; color: var(--fg-muted); margin: 8px 0 16px; }
</style>
