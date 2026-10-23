<script setup lang="ts">
import { ref } from 'vue'
import { useKeytoolCheck } from '../../../shell/use-keytool'
import KeytoolHint from '../../../shell/KeytoolHint.vue'
import KeystoreGenPanel from './KeystoreGenPanel.vue'
import KeystoreDetailsPanel from './KeystoreDetailsPanel.vue'
import STabs from '@qingfeng346/ui-kit/components/tabs/STabs.vue'
import STabPane from '@qingfeng346/ui-kit/components/tabs/STabPane.vue'

type Tab = 'details' | 'gen'

const tab = ref<Tab>('details')
const { javaAvailable, javaPath } = useKeytoolCheck()
</script>

<template>
  <div class="ksv">
    <h2>Keystore 工具</h2>
    <p class="lead">
      Android 签名相关操作：查看 keystore / 证书 / APK 的证书详情与指纹，或生成新的签名密钥。文件与密码仅在本机处理，不会上传到任何服务器。
    </p>

    <KeytoolHint :available="javaAvailable" :path="javaPath" />

    <STabs v-model="tab" class="ksv-tabs">
      <STabPane name="details" tab="文件详情" displayDirective="destroy">
        <KeystoreDetailsPanel :java-available="javaAvailable" />
      </STabPane>
      <STabPane name="gen" tab="生成 Keystore" displayDirective="destroy">
        <KeystoreGenPanel :java-available="javaAvailable" />
      </STabPane>
    </STabs>
  </div>
</template>

<style scoped>
.ksv { max-width: 720px; margin: 0 auto; }
.lead { color: var(--fg-muted); margin-bottom: 16px; }

.ksv-tabs { margin-bottom: 16px; }
</style>
