<!--
/**
 * ---
 * [INPUT]: {RouterView}
 *     - RouterView: source: vue-router ([POS]: 路由视图组件)
 * [OUTPUT]: {JSX.Element} - 渲染路由对应的页面组件
 * [POS]: Vue 应用根组件，提供路由视图容器
 * [PROTOCOL]:
 * 1. 不包含具体页面逻辑
 * 2. 主题初始化在 main.ts 中通过 ThemeManager.initTheme() 处理
 * ---
 */
-->
<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue';
import { RouterView } from 'vue-router';
import { listen } from '@tauri-apps/api/event';
import PortConflictModal from './components/Modals/PortConflict.vue';

type ProxyErrorPayload = {
  type: string;
  port: number;
  blocker?: {
    name?: string;
    pid?: number;
  };
};

const showPortConflict = ref(false);
const blockerApp = ref<string | null>(null);
let unlistenProxyError: (() => void) | null = null;

const handleProxyError = (payload?: ProxyErrorPayload) => {
  if (!payload || payload.type !== 'PORT_CONFLICT') {
    return;
  }
  blockerApp.value = payload.blocker?.name ?? null;
  showPortConflict.value = true;
};

onMounted(async () => {
  try {
    unlistenProxyError = await listen<ProxyErrorPayload>('proxy-error', event => {
      handleProxyError(event.payload);
    });
  } catch (error) {
    console.error('failed to register proxy-error listener', error);
  }
});

onBeforeUnmount(() => {
  if (unlistenProxyError) {
    unlistenProxyError();
    unlistenProxyError = null;
  }
});

const dismissPortConflict = () => {
  showPortConflict.value = false;
};
</script>

<template>
  <div data-tauri-drag-region class="titlebar">&nbsp;</div>
  <RouterView />
  <PortConflictModal v-if="showPortConflict" :blocker-app="blockerApp" @dismiss="dismissPortConflict" />
</template>
