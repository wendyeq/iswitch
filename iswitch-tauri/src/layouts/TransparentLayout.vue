<!--
/**
 * ---
 * [INPUT]: {RouterView}
 *   source: vue-router ([POS]: 路由视图组件)
 * [OUTPUT]: {JSX.Element} - 透明背景布局
 * [POS]: HUD 专用透明布局组件，不显示标题栏和窗口装饰
 * [PROTOCOL]: FractalFlow v1.0
 * ---
 */
-->
<script setup lang="ts">
import { RouterView } from 'vue-router';
</script>

<template>
  <div class="transparent-layout">
    <RouterView />
  </div>
</template>

<style scoped>
.transparent-layout {
  width: 100vw;
  height: 100vh;
  background: transparent;
  overflow: hidden;
}
</style>
