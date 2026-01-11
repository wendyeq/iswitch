<!--
[INPUT]:
  source: ../../../../iswitch-tauri/src-tauri/src/lib.rs ([POS]: proxy-error 事件发射)

[OUTPUT]:
  - 端口冲突提示弹窗

[POS]: PortConflict 前端模态框

[PROTOCOL]: FractalFlow v1.0 - 分形自洽
-->
<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import BaseButton from '../common/BaseButton.vue';

const props = defineProps<{
  blockerApp?: string | null;
}>();

const emit = defineEmits<{
  (e: 'dismiss'): void;
}>();

const { t } = useI18n();

const blockerLabel = computed(() => props.blockerApp || t('components.modals.portConflict.unknownApp'));

const handleQuit = async () => {
  try {
    await invoke('quit_app');
  } catch (error) {
    console.error('failed to quit application', error);
  }
};

const handleDismiss = () => {
  emit('dismiss');
};
</script>

<template>
  <div class="port-conflict-overlay" role="dialog" aria-modal="true">
    <div class="port-conflict-dialog">
      <div class="dialog-header">
        <h3>{{ t('components.modals.portConflict.title') }}</h3>
      </div>
      <p class="dialog-message">
        {{
          t('components.modals.portConflict.message', {
            app: blockerLabel,
          })
        }}
      </p>
      <div class="dialog-actions">
        <BaseButton variant="ghost" size="sm" type="button" @click="handleDismiss">
          {{ t('components.modals.portConflict.actionDismiss') }}
        </BaseButton>
        <BaseButton size="sm" type="button" @click="handleQuit">
          {{ t('components.modals.portConflict.actionQuit') }}
        </BaseButton>
      </div>
    </div>
  </div>
</template>

<style scoped>
.port-conflict-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  backdrop-filter: blur(6px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
}

.port-conflict-dialog {
  width: min(420px, calc(100vw - 32px));
  border-radius: 20px;
  padding: 28px;
  background: rgba(255, 255, 255, 0.9);
  box-shadow:
    0 20px 45px rgba(15, 23, 42, 0.25),
    inset 0 0 0 1px rgba(255, 255, 255, 0.35);
  color: #0f172a;
}

.dialog-header h3 {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
}

.dialog-message {
  margin: 16px 0 28px;
  font-size: 14px;
  line-height: 1.6;
  color: rgba(15, 23, 42, 0.8);
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

html.dark .port-conflict-dialog {
  background: rgba(10, 14, 24, 0.92);
  box-shadow:
    0 20px 45px rgba(0, 0, 0, 0.6),
    inset 0 0 0 1px rgba(148, 163, 184, 0.15);
  color: #e2e8f0;
}

html.dark .dialog-message {
  color: rgba(226, 232, 240, 0.85);
}
</style>
