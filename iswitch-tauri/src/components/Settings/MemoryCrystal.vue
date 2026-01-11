<!--
// [INPUT] Prop: status (string) - 'synced' | 'missing' | 'pending'
// [OUTPUT] None
// [PROTOCOL] FractalFlow v1.0
// [POS] src/components/Settings/MemoryCrystal.vue
-->

<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';

const props = defineProps<{
  status: 'synced' | 'missing' | 'pending';
  label?: string; // Overrides status text
  title?: string; // Overrides header title
}>();

const { t } = useI18n();

// Map status to visual classes
const crystalClass = computed(() => {
  switch (props.status) {
    case 'synced':
      return 'crystal--synced';
    case 'pending':
      return 'crystal--pending';
    case 'missing':
      return 'crystal--missing';
    default:
      return 'crystal--missing';
  }
});

const statusText = computed(() => {
  if (props.label) return props.label;
  switch (props.status) {
    case 'synced':
      return t('components.general.import.synced'); // "Synchronized"
    case 'pending':
      return t('components.general.import.cta'); // "Import"
    case 'missing':
      return t('components.general.import.noFile'); // "No Config"
    default:
      return '';
  }
});
</script>

<template>
  <div class="memory-crystal-container">
    <div class="crystal-orb" :class="crystalClass">
      <div class="crystal-core"></div>
      <div class="crystal-aura"></div>
    </div>
    <div class="crystal-info">
      <span class="crystal-title">{{ $t('components.general.import.label') }}</span>
      <span class="crystal-status">{{ statusText }}</span>
    </div>
  </div>
</template>

<style scoped>
.memory-crystal-container {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 12px;
  border-radius: 16px;
  transition: background 0.3s ease;
}

.memory-crystal-container:hover {
  background: rgba(255, 255, 255, 0.05);
}

.crystal-orb {
  position: relative;
  width: 32px;
  height: 32px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* Core - The physical object */
.crystal-core {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: white;
  position: relative;
  z-index: 2;
  box-shadow: inset -2px -2px 6px rgba(0, 0, 0, 0.2);
}

/* Aura - The glow */
.crystal-aura {
  position: absolute;
  width: 100%;
  height: 100%;
  border-radius: 50%;
  filter: blur(8px);
  opacity: 0.6;
  z-index: 1;
  animation: breathe 3s infinite ease-in-out;
}

/* Status: Synced (Prism Blue) */
/* Was Jony Green, now Prism Blue as per correction */
.crystal--synced .crystal-core {
  background: #00f0ff; /* Electric Cyan */
  box-shadow: 0 0 10px #00f0ff;
}
.crystal--synced .crystal-aura {
  background: #00f0ff;
}

/* Status: Missing (Amber) */
.crystal--missing .crystal-core {
  background: #ff9f0a;
  box-shadow: 0 0 5px #ff9f0a;
}
.crystal--missing .crystal-aura {
  background: #ff9f0a;
  opacity: 0.3;
  animation: none;
}

/* Status: Pending (White Pulse) */
.crystal--pending .crystal-core {
  background: #ffffff;
  box-shadow: 0 0 10px white;
}
.crystal--pending .crystal-aura {
  background: white;
  animation: breathe 1.5s infinite ease-in-out;
}

@keyframes breathe {
  0%,
  100% {
    transform: scale(0.8);
    opacity: 0.4;
  }
  50% {
    transform: scale(1.1);
    opacity: 0.7;
  }
}

.crystal-info {
  display: flex;
  flex-direction: column;
}

.crystal-title {
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.8px;
  opacity: 0.5;
  font-weight: 600;
}

.crystal-status {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}
</style>
