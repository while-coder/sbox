<script setup lang="ts">
import SboxApp from '@sbox/ui/app'
import { UpdaterDialog } from '@while-coder/tauri-updater-vue'
</script>

<template>
  <SboxApp><template #updater><UpdaterDialog /></template></SboxApp>
</template>
