<!--
  [INPUT]: source: openspec/specs/provider-capsules/spec.md ([POS]: Notification Design)
  [OUTPUT]: Inline Notification Component
  [PROTOCOL]: FractalFlow v1.0
  [POS]: iswitch-tauri/src/components/common/InlineNotification.vue
-->
<template>
  <Teleport to="body">
    <Transition name="toast-fade">
      <div v-if="visible" class="inline-notification">
        <div class="notification-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path
              d="M12 2v6m0 0l-3-3m3 3l3-3M4.93 4.93L7.76 7.76M2 12h6m0 0l-3-3m3 3l-3 3M4.93 19.07l2.83-2.83M12 22v-6m0 0l3 3m-3-3l-3 3M19.07 19.07l-2.83-2.83M22 12h-6m0 0l3 3m-3-3l3-3M19.07 4.93l-2.83 2.83"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        </div>
        <span class="notification-message">{{ message }}</span>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
interface Props {
  visible: boolean;
  message: string;
}

defineProps<Props>();
</script>

<style scoped>
.inline-notification {
  position: fixed;
  top: 100px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 2000;

  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 16px;

  border-radius: 999px;
  background: rgba(40, 40, 45, 0.9);
  /* Lighter dark background for contrast against black pages */
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border: 1px solid rgba(255, 255, 255, 0.2);
  /* Stronger border */
  box-shadow:
    0 8px 24px rgba(0, 0, 0, 0.5),
    inset 0 1px 0 rgba(255, 255, 255, 0.1);

  color: #fff;
  font-size: 0.9rem;
  font-weight: 500;
  white-space: nowrap;
  pointer-events: none;
  transition: all 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
}

/* Light mode adjustment */
:global(html.light) .inline-notification {
  background: rgba(255, 255, 255, 0.9);
  color: #000;
  border-color: rgba(0, 0, 0, 0.15);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
}

.notification-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  color: #3b82f6;
  /* Blue accent */
  width: 18px;
  height: 18px;
}

.notification-icon svg {
  width: 100%;
  height: 100%;
  animation: spin-slow 3s linear infinite;
}

@keyframes spin-slow {
  from {
    transform: rotate(0deg);
  }

  to {
    transform: rotate(360deg);
  }
}

.toast-fade-enter-active,
.toast-fade-leave-active {
  transition: all 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.toast-fade-enter-from,
.toast-fade-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(-10px) scale(0.95);
}
</style>
