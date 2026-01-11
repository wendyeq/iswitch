<script setup lang="ts">
import { ref, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { open } from '@tauri-apps/plugin-dialog';
import {
  fetchConfigImportStatus,
  fetchConfigImportStatusForFile,
  importFromCcSwitch,
  importFromCustomFile,
  fetchCodeSwitchImportStatus,
  importFromCodeSwitch,
  type ConfigImportResult,
  type ConfigImportStatus,
} from '../../services/configImport';
import { showToast } from '../../utils/toast';
import BaseButton from '../common/BaseButton.vue';

const { t } = useI18n();

const ccSwitchStatus = ref<ConfigImportStatus | null>(null);
const codeSwitchStatus = ref<ConfigImportStatus | null>(null);
const customStatus = ref<ConfigImportStatus | null>(null);
const isBusy = ref(false);
const isDragging = ref(false);

type ImportSource = 'cc-switch' | 'code-switch' | 'custom';

const activeSource = ref<ImportSource | null>(null);
const activeStatus = computed(() => {
  if (activeSource.value === 'custom') return customStatus.value;
  if (activeSource.value === 'code-switch') return codeSwitchStatus.value;
  return ccSwitchStatus.value;
});

const loadStatuses = async () => {
  try {
    ccSwitchStatus.value = await fetchConfigImportStatus();
    codeSwitchStatus.value = await fetchCodeSwitchImportStatus();
  } catch (error) {
    console.error('failed to load import statuses', error);
  }
};

const hasPending = (status: ConfigImportStatus | null) => {
  return Boolean(status?.pending_providers || status?.pending_mcp);
};

const isSynced = (status: ConfigImportStatus | null) => {
  return status?.config_exists && !hasPending(status);
};

const crystalState = computed(() => {
  const status = activeStatus.value;
  if (!status) return 'missing';
  if (isSynced(status)) return 'synced';
  if (hasPending(status)) return 'pending';
  return 'missing';
});

const crystalClass = computed(() => ({
  'sync-crystal': true,
  'sync-crystal--dragging': isDragging.value,
  'sync-crystal--synced': crystalState.value === 'synced',
  'sync-crystal--pending': crystalState.value === 'pending',
  'sync-crystal--missing': crystalState.value === 'missing',
  'sync-crystal--busy': isBusy.value,
}));

const statusText = computed(() => {
  if (isBusy.value) return t('components.settings.sync.syncing');
  if (crystalState.value === 'synced') return t('components.settings.sync.synced');
  if (crystalState.value === 'pending') {
    const status = activeStatus.value;
    return t('components.general.import.detail', {
      providers: status?.pending_provider_count ?? 0,
      servers: status?.pending_mcp_count ?? 0,
    });
  }
  return t('components.settings.sync.noConfig');
});

const handleImport = async (source: ImportSource) => {
  if (isBusy.value) return;

  const status =
    source === 'custom' ? customStatus.value : source === 'code-switch' ? codeSwitchStatus.value : ccSwitchStatus.value;

  if (!hasPending(status)) return;

  isBusy.value = true;
  activeSource.value = source;

  try {
    let result: ConfigImportResult | null = null;

    if (source === 'custom' && customStatus.value?.config_path) {
      result = await importFromCustomFile(customStatus.value.config_path);
    } else if (source === 'code-switch') {
      result = await importFromCodeSwitch();
    } else {
      result = await importFromCcSwitch();
    }

    if (result) {
      const importedProviders = result.imported_providers ?? 0;
      const importedServers = result.imported_mcp ?? 0;

      if (importedProviders > 0 || importedServers > 0) {
        showToast(
          t('components.main.importConfig.success', {
            providers: importedProviders,
            servers: importedServers,
          })
        );
      } else if (result.status?.config_exists) {
        showToast(t('components.main.importConfig.empty'));
      }

      // Update status
      if (source === 'custom') {
        customStatus.value = result.status;
      } else if (source === 'code-switch') {
        codeSwitchStatus.value = result.status;
      } else {
        ccSwitchStatus.value = result.status;
      }
    }
  } catch (error) {
    console.error('failed to import', error);
    showToast(t('components.main.importConfig.error'), 'error');
  } finally {
    isBusy.value = false;
  }
};

const handleUpload = async () => {
  if (isBusy.value) return;

  try {
    const selection = await open({
      title: t('components.general.import.uploadTitle'),
      multiple: false,
      directory: false,
      filters: [{ name: 'JSON', extensions: ['json'] }],
    });

    const path = Array.isArray(selection) ? selection[0] : selection;
    if (!path) return;

    const status = await fetchConfigImportStatusForFile(path);
    customStatus.value = status;
    activeSource.value = 'custom';
  } catch (error) {
    console.error('failed to load custom config', error);
    showToast(t('components.general.import.loadError'), 'error');
  }
};

const handleDrop = async (e: DragEvent) => {
  e.preventDefault();
  isDragging.value = false;

  if (isBusy.value) return;

  const file = e.dataTransfer?.files[0];
  if (!file) return;

  // In Tauri, we'd need to handle this differently
  // For now, we'll skip drag-drop or use Tauri's file drop API
  showToast(t('components.settings.sync.dragNotSupported'), 'error');
};

const handleDragOver = (e: DragEvent) => {
  e.preventDefault();
  if (!isBusy.value) {
    isDragging.value = true;
  }
};

const handleDragLeave = () => {
  isDragging.value = false;
};

defineExpose({
  loadStatuses,
});

// Load initial statuses
loadStatuses();
</script>

<template>
  <div :class="crystalClass" @drop="handleDrop" @dragover="handleDragOver" @dragleave="handleDragLeave">
    <div class="sync-crystal__core">
      <div class="sync-crystal__beacon" />
      <div class="sync-crystal__rings">
        <span class="ring" />
        <span class="ring" />
        <span class="ring" />
      </div>
    </div>

    <div class="sync-crystal__info">
      <p class="sync-crystal__status">{{ statusText }}</p>
      <p v-if="activeStatus?.config_path" class="sync-crystal__path">
        {{ t('components.general.import.path', { path: activeStatus.config_path }) }}
      </p>
    </div>

    <div class="sync-crystal__actions">
      <BaseButton
        v-if="ccSwitchStatus && hasPending(ccSwitchStatus)"
        size="sm"
        variant="primary"
        :disabled="isBusy"
        @click="handleImport('cc-switch')"
      >
        {{ t('components.general.import.cta') }}
      </BaseButton>

      <BaseButton
        v-if="codeSwitchStatus && hasPending(codeSwitchStatus)"
        size="sm"
        variant="primary"
        :disabled="isBusy"
        @click="handleImport('code-switch')"
      >
        {{ t('components.general.import.cta') }}
      </BaseButton>

      <BaseButton
        v-if="customStatus && hasPending(customStatus)"
        size="sm"
        variant="primary"
        :disabled="isBusy"
        @click="handleImport('custom')"
      >
        {{ t('components.general.import.confirm') }}
      </BaseButton>

      <BaseButton size="sm" variant="outline" :disabled="isBusy" @click="handleUpload">
        {{ t('components.general.import.upload') }}
      </BaseButton>

      <BaseButton
        v-if="customStatus"
        size="sm"
        variant="ghost"
        :disabled="isBusy"
        @click="
          customStatus = null;
          activeSource = null;
        "
      >
        {{ t('components.general.import.clear') }}
      </BaseButton>
    </div>
  </div>
</template>

<style scoped>
.sync-crystal {
  position: relative;
  display: flex;
  align-items: center;
  gap: 20px;
  padding: 24px;
  background: hsla(0, 0%, 100%, 0.5);
  border-radius: 20px;
  border: 2px dashed hsla(0, 0%, 50%, 0.15);
  transition: all 0.3s ease;
  overflow: hidden;
}

.sync-crystal--dragging {
  border-color: hsla(210, 100%, 52%, 0.5);
  background: hsla(210, 100%, 52%, 0.05);
  transform: scale(1.02);
}

.sync-crystal--synced {
  border-style: solid;
  border-color: hsla(142, 76%, 36%, 0.3);
  background: hsla(142, 76%, 36%, 0.05);
}

.sync-crystal--pending {
  border-color: hsla(38, 92%, 50%, 0.4);
  background: hsla(38, 92%, 50%, 0.05);
}

html.dark .sync-crystal {
  background: hsla(0, 0%, 100%, 0.03);
  border-color: hsla(0, 0%, 50%, 0.2);
}

html.dark .sync-crystal--synced {
  background: hsla(142, 76%, 36%, 0.1);
  border-color: hsla(142, 76%, 36%, 0.4);
}

html.dark .sync-crystal--pending {
  background: hsla(38, 92%, 50%, 0.1);
  border-color: hsla(38, 92%, 50%, 0.5);
}

.sync-crystal__core {
  position: relative;
  width: 48px;
  height: 48px;
  flex-shrink: 0;
}

.sync-crystal__beacon {
  position: absolute;
  inset: 12px;
  border-radius: 50%;
  background: hsla(0, 0%, 50%, 0.2);
  transition: all 0.3s ease;
}

.sync-crystal--synced .sync-crystal__beacon {
  background: hsla(142, 76%, 36%, 1);
  box-shadow: 0 0 20px hsla(142, 76%, 36%, 0.6);
  animation: pulse-green 2s ease-in-out infinite;
}

.sync-crystal--pending .sync-crystal__beacon {
  background: hsla(38, 92%, 50%, 1);
  box-shadow: 0 0 16px hsla(38, 92%, 50%, 0.5);
  animation: pulse-amber 1.5s ease-in-out infinite;
}

.sync-crystal__rings {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: none;
}

.ring {
  position: absolute;
  border-radius: 50%;
  border: 1px solid hsla(0, 0%, 50%, 0.1);
  opacity: 0;
}

.sync-crystal--synced .ring {
  border-color: hsla(142, 76%, 36%, 0.3);
  animation: ring-expand 2s ease-out infinite;
}

.sync-crystal--synced .ring:nth-child(2) {
  animation-delay: 0.6s;
}

.sync-crystal--synced .ring:nth-child(3) {
  animation-delay: 1.2s;
}

@keyframes pulse-green {
  0%,
  100% {
    opacity: 1;
    transform: scale(1);
  }
  50% {
    opacity: 0.8;
    transform: scale(1.1);
  }
}

@keyframes pulse-amber {
  0%,
  100% {
    opacity: 1;
    transform: scale(1);
  }
  50% {
    opacity: 0.7;
    transform: scale(1.05);
  }
}

@keyframes ring-expand {
  0% {
    inset: 12px;
    opacity: 0.6;
  }
  100% {
    inset: -4px;
    opacity: 0;
  }
}

.sync-crystal__info {
  flex: 1;
  min-width: 0;
}

.sync-crystal__status {
  margin: 0 0 4px 0;
  font-size: 14px;
  font-weight: 500;
  color: hsla(0, 0%, 0%, 0.85);
}

.sync-crystal__path {
  margin: 0;
  font-size: 12px;
  color: hsla(0, 0%, 0%, 0.5);
  font-family: 'SF Mono', ui-monospace, SFMono-Regular, Menlo, monospace;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

html.dark .sync-crystal__status {
  color: hsla(0, 0%, 100%, 0.9);
}

html.dark .sync-crystal__path {
  color: hsla(0, 0%, 100%, 0.5);
}

.sync-crystal__actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  justify-content: flex-end;
}
</style>
