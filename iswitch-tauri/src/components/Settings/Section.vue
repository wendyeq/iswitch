<script setup lang="ts">
interface Props {
  title?: string;
  glassEffect?: boolean;
}

withDefaults(defineProps<Props>(), {
  title: '',
  glassEffect: true,
});
</script>

<template>
  <section :class="{ 'glass-section': glassEffect }">
    <h2 v-if="title" class="section-title">{{ title }}</h2>
    <div class="section-content">
      <slot />
    </div>
  </section>
</template>

<style scoped>
.glass-section {
  background: hsla(0, 0%, 100%, 0.7);
  backdrop-filter: blur(20px) saturate(180%);
  -webkit-backdrop-filter: blur(20px) saturate(180%);
  border-radius: 16px;
  padding: 20px;
  margin-bottom: 24px;
  border: 1px solid hsla(0, 0%, 50%, 0.1);
  box-shadow:
    0 4px 24px rgba(0, 0, 0, 0.06),
    inset 0 1px 0 hsla(0, 0%, 100%, 0.5);
}

html.dark .glass-section {
  background: hsla(0, 0%, 10%, 0.6);
  border-color: hsla(0, 0%, 50%, 0.15);
  box-shadow:
    0 4px 24px rgba(0, 0, 0, 0.3),
    inset 0 1px 0 hsla(0, 0%, 100%, 0.08);
}

.section-title {
  font-size: 13px;
  font-weight: 600;
  color: hsla(0, 0%, 0%, 0.5);
  margin: 0 0 12px 0;
  letter-spacing: 0.02em;
  text-transform: uppercase;
}

html.dark .section-title {
  color: hsla(0, 0%, 100%, 0.5);
}

.section-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}
</style>
