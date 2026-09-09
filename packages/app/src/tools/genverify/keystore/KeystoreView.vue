<script setup lang="ts">
import { ref } from 'vue'
import { useKeytoolCheck } from '../../../shell/use-keytool'
import KeytoolHint from '../../../shell/KeytoolHint.vue'
import KeystoreGenPanel from './KeystoreGenPanel.vue'
import KeystoreHashPanel from './KeystoreHashPanel.vue'
import KeystoreInfoPanel from './KeystoreInfoPanel.vue'

type Tab = 'info' | 'hash' | 'gen'

const tab = ref<Tab>('info')
const { javaAvailable, javaPath } = useKeytoolCheck()
</script>

<template>
  <div class="ksv">
    <h2>Keystore 工具</h2>
    <p class="lead">
      Android APK 签名 keystore 相关操作：生成新的签名密钥，或读取现有 keystore 的证书指纹。文件与密码仅在本机传给 <code>keytool</code>，不会上传到任何服务器。
    </p>

    <KeytoolHint :available="javaAvailable" :path="javaPath" />

    <div class="tabs">
      <button class="tab" :class="{ active: tab === 'info' }" @click="tab = 'info'">查看指纹</button>
      <button class="tab" :class="{ active: tab === 'hash' }" @click="tab = 'hash'">发布密钥散列</button>
      <button class="tab" :class="{ active: tab === 'gen' }" @click="tab = 'gen'">生成 Keystore</button>
    </div>

    <KeystoreInfoPanel v-if="tab === 'info'" :java-available="javaAvailable" />
    <KeystoreHashPanel v-else-if="tab === 'hash'" :java-available="javaAvailable" />
    <KeystoreGenPanel v-else-if="tab === 'gen'" :java-available="javaAvailable" />
  </div>
</template>

<style scoped>
.ksv { max-width: 720px; margin: 0 auto; }
.lead { color: var(--fg-muted); margin-bottom: 16px; }

.tabs {
  display: flex; gap: 4px; margin-bottom: 16px;
  border-bottom: 1px solid var(--border);
}
.tab {
  padding: 8px 16px; font-size: 13px; cursor: pointer;
  background: none; border: none; color: var(--fg-muted);
  border-bottom: 2px solid transparent; margin-bottom: -1px;
}
.tab:hover { color: var(--fg); }
.tab.active {
  color: var(--primary); font-weight: 500;
  border-bottom-color: var(--primary);
}
</style>
