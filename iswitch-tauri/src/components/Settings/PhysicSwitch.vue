<!--
// [INPUT] Prop: modelValue (boolean) - The toggle state
// [OUTPUT] Event: update:modelValue - Emitted on toggle
// [PROTOCOL] FractalFlow v1.0
// [POS] src/components/Settings/PhysicSwitch.vue
-->

<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  modelValue: boolean;
  label?: string;
  disabled?: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
}>();

const toggle = () => {
  if (props.disabled) return;
  emit('update:modelValue', !props.modelValue);
};

const switchClasses = computed(() => ({
  'physic-switch--active': props.modelValue,
  'physic-switch--inactive': !props.modelValue,
  'physic-switch--disabled': props.disabled,
}));
</script>

<template>
  <div class="physic-switch-wrapper" :class="{ disabled: disabled }" @click="toggle">
    <div class="physic-switch" :class="switchClasses">
      <div class="physic-knob"></div>
      <div class="physic-bloom"></div>
    </div>
    <span v-if="label" class="physic-label">{{ label }}</span>
  </div>
</template>

<style scoped>
.physic-switch-wrapper {
  display: flex;
  align-items: center;
  gap: 12px;
  cursor: pointer;
  user-select: none;
}

.physic-switch-wrapper.disabled {
  cursor: not-allowed;
  opacity: 0.6;
}

.physic-switch {
  position: relative;
  width: 50px;
  height: 28px;
  background: rgba(120, 120, 128, 0.16);
  border-radius: 30px;
  transition: background-color 0.4s cubic-bezier(0.4, 0, 0.2, 1);
  overflow: visible;
  /* To allow glow leak */
}

/* Light Mode Defaults */
.physic-switch--inactive {
  background: rgba(120, 120, 128, 0.2);
  border: 1px solid rgba(0, 0, 0, 0.05);
}

.physic-switch--active {
  /* Azure Blue - Matching LevitatingCapsule toggle */
  background: var(--capsule-accent-azure, #3b82f6);
  border: 1px solid rgba(59, 130, 246, 0.5);
  box-shadow: 0 0 15px rgba(59, 130, 246, 0.4);
}

.physic-knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 24px;
  height: 24px;
  background: white;
  border-radius: 50%;
  box-shadow:
    0 3px 8px rgba(0, 0, 0, 0.15),
    0 3px 1px rgba(0, 0, 0, 0.06);
  transition: transform 0.5s cubic-bezier(0.17, 0.89, 0.32, 1.15);
  /* Springy */
  z-index: 2;
}

.physic-switch--active .physic-knob {
  transform: translateX(22px);
}

/* Internal Bloom/Glow */
.physic-bloom {
  position: absolute;
  top: 50%;
  left: 50%;
  width: 100%;
  height: 100%;
  transform: translate(-50%, -50%) scale(0.8);
  border-radius: 30px;
  opacity: 0;
  background: inherit;
  filter: blur(8px);
  transition: opacity 0.4s ease;
  z-index: 0;
}

.physic-switch--active .physic-bloom {
  opacity: 0.6;
}

/* Dark Mode Overrides */
:global(html.dark) .physic-switch--inactive {
  background: rgba(255, 255, 255, 0.15);
  border: 1px solid rgba(255, 255, 255, 0.1);
}

:global(html.dark) .physic-switch--active {
  background: var(--capsule-accent-azure, #3b82f6);
  /* Azure Blue 500 */
  box-shadow: 0 0 20px rgba(59, 130, 246, 0.5);
}

.physic-label {
  font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Text', 'Segoe UI', Roboto, sans-serif;
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
  opacity: 0.9;
}
</style>
