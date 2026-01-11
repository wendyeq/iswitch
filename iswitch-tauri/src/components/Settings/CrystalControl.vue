// [INPUT] settings.rs (Default Values) // [OUTPUT] UI sections for Proxy, Failover, Recovery with default hints //
[PROTOCOL] FractalFlow v1.0 // [POS] src/components/Settings/CrystalControl.vue

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { getVersion } from '@tauri-apps/api/app';
import PhysicSwitch from './PhysicSwitch.vue';
import { fetchAppSettings, saveAppSettings, type AppSettings } from '../../services/appSettings';
import {
  fetchConfigImportStatus,
  importFromCcSwitch,
  fetchCodeSwitchImportStatus,
  importFromCodeSwitch,
  type ConfigImportStatus,
} from '../../services/configImport';
import { showToast } from '../../utils/toast';
import { setupI18n } from '../../utils/i18n';
import type { Locale } from '../../locales';

const router = useRouter();
const { t, locale } = useI18n();

// State
const isVisible = ref(false); // For entry animation
const heatmapEnabled = ref(true);
const autoStartEnabled = ref(false);
const settingsLoading = ref(true);
const importStatus = ref<ConfigImportStatus | null>(null);
const importBusy = ref(false);
const codeSwitchImportStatus = ref<ConfigImportStatus | null>(null);
const codeSwitchImportBusy = ref(false);
const appVersion = ref('');

// Lifecycle
onMounted(async () => {
  // Trigger entry animation
  setTimeout(() => {
    isVisible.value = true;
  }, 50);

  // Load data
  try {
    const data = await fetchAppSettings();
    heatmapEnabled.value = data?.show_heatmap ?? true;
    autoStartEnabled.value = data?.auto_start ?? false;
  } catch (e) {
    console.error('Failed to load settings', e);
  } finally {
    settingsLoading.value = false;
  }

  // Load import status
  try {
    importStatus.value = await fetchConfigImportStatus();
  } catch (e) {
    console.error('Failed to load import status', e);
  }

  // Load code-switch import status
  try {
    codeSwitchImportStatus.value = await fetchCodeSwitchImportStatus();
  } catch (e) {
    console.error('Failed to load code-switch import status', e);
  }

  // Load version
  try {
    appVersion.value = await getVersion();
  } catch (e) {
    appVersion.value = '0.0.0';
  }
});

// Persistence
const persist = async () => {
  if (settingsLoading.value) return;
  try {
    const payload: AppSettings = {
      show_heatmap: heatmapEnabled.value,
      auto_start: autoStartEnabled.value,
    };
    await saveAppSettings(payload);
    window.dispatchEvent(new CustomEvent('app-settings-updated'));
  } catch (e) {
    console.error('Failed to save settings', e);
  }
};

// Sync Action
const handleSync = async () => {
  if (importBusy.value) return;
  importBusy.value = true;
  try {
    const result = await importFromCcSwitch();
    if (result) {
      importStatus.value = result.status;
      showToast(
        t('components.main.importConfig.success', {
          providers: result.imported_providers || 0,
          servers: result.imported_mcp || 0,
        })
      );
    }
  } catch (e) {
    showToast(t('components.main.importConfig.error'), 'error');
  } finally {
    importBusy.value = false;
  }
};

// Code-Switch Sync Action
const handleCodeSwitchSync = async () => {
  if (codeSwitchImportBusy.value) return;
  codeSwitchImportBusy.value = true;
  try {
    const result = await importFromCodeSwitch();
    if (result) {
      codeSwitchImportStatus.value = result.status;
      showToast(
        t('components.main.importConfig.success', {
          providers: result.imported_providers || 0,
          servers: result.imported_mcp || 0,
        })
      );
    }
  } catch (e) {
    showToast(t('components.main.importConfig.error'), 'error');
  } finally {
    codeSwitchImportBusy.value = false;
  }
};

// Lang
const switchLang = async (lang: string) => {
  await setupI18n(lang as Locale);
};

const goBack = () => {
  isVisible.value = false;
  setTimeout(() => {
    router.push('/');
  }, 400); // Wait for exit animation
};
</script>

<template>
  <div class="crystal-overlay">
    <!-- Close Area -->
    <div class="crystal-backdrop" @click="goBack"></div>

    <!-- The Slate -->
    <div class="crystal-slate" :class="{ 'crystal-slate--visible': isVisible }">
      <!-- Header -->
      <div class="slate-header">
        <h2 class="slate-title">{{ t('components.general.title.application') }}</h2>
        <button class="close-btn" @click="goBack">×</button>
      </div>

      <!-- Section: Surface -->
      <div class="slate-section">
        <h3 class="section-label">{{ t('components.settings.surface') }}</h3>
        <div class="control-row">
          <!-- Language Segmented Control -->
          <div class="control-group">
            <span class="control-label">{{ t('components.general.label.language') }}</span>
            <div class="segmented-control" :class="{ 'lang-en': locale === 'en' }">
              <button class="segment-btn" :class="{ active: locale === 'zh' }" @click="switchLang('zh')">中文</button>
              <button class="segment-btn" :class="{ active: locale === 'en' }" @click="switchLang('en')">EN</button>
            </div>
          </div>

          <!-- Heatmap Toggle -->
          <div class="control-group">
            <div class="label-group">
              <span class="control-label">{{ t('components.general.label.heatmap') }}</span>
            </div>
            <PhysicSwitch v-model="heatmapEnabled" @update:modelValue="persist" :disabled="settingsLoading" />
          </div>

          <!-- Auto Start Toggle -->
          <div class="control-group">
            <span class="control-label">{{ t('components.general.label.autoStart') }}</span>
            <PhysicSwitch v-model="autoStartEnabled" @update:modelValue="persist" :disabled="settingsLoading" />
          </div>
        </div>
      </div>
      <!-- Footer: Sync Portal (低调处理 - 非常用功能) -->
      <div class="slate-import-links">
        <button class="import-link" @click="handleSync" :disabled="importBusy">
          {{ t('components.general.import.label') }}
        </button>
        <span class="import-divider">·</span>
        <button class="import-link" @click="handleCodeSwitchSync" :disabled="codeSwitchImportBusy">
          {{ t('components.general.import.codeSwitchLabel') }}
        </button>
      </div>

      <!-- Version & Copyright (Subtle) -->
      <div class="slate-version">v{{ appVersion }}</div>
      <div class="slate-copyright">{{ t('components.general.about.copyright') }}</div>
    </div>
  </div>
</template>

<style scoped>
.crystal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
}

.crystal-backdrop {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.2);
  backdrop-filter: blur(2px);
  z-index: 0;
}

:global(html.dark) .crystal-backdrop {
  background: rgba(0, 0, 0, 0.4);
  backdrop-filter: blur(4px);
}

.crystal-slate {
  position: relative;
  z-index: 10;
  width: 420px;

  /* Theme Variables */
  --slate-bg: var(--capsule-bg);
  --slate-border: var(--capsule-border);
  --slate-shadow: var(--capsule-shadow);
  --text-primary: var(--capsule-text-primary, var(--mac-text));
  --text-secondary: var(--capsule-text-secondary, var(--mac-text-secondary));
  --text-tertiary: color-mix(in srgb, var(--text-secondary) 65%, transparent);
  --input-bg: color-mix(in srgb, var(--mac-text) 4%, transparent);
  --footer-bg: color-mix(in srgb, var(--surface-panel-bg) 80%, transparent);
  --divider-color: var(--border-subtle);
  --segment-bg: color-mix(in srgb, var(--surface-panel-bg) 70%, transparent);
  --segment-active-bg: color-mix(in srgb, var(--capsule-border) 30%, var(--surface-panel-bg));
  --segment-active-text: var(--text-primary);

  background: var(--slate-bg);
  backdrop-filter: blur(40px) saturate(180%);
  border: 1px solid var(--slate-border);
  box-shadow: var(--slate-shadow);
  border-radius: 24px;
  padding: 32px;
  color: var(--text-primary);

  opacity: 0;
  transform: translateY(40px) scale(0.95);
  transition:
    opacity 0.4s ease,
    transform 0.6s cubic-bezier(0.16, 1, 0.3, 1),
    background 0.3s ease,
    color 0.3s ease;

  max-height: 90vh;
  overflow-y: auto;
}

.crystal-slate--visible {
  opacity: 1;
  transform: translateY(0) scale(1);
}

.slate-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 32px;
}

.slate-title {
  font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Display', sans-serif;
  font-size: 24px;
  font-weight: 600;
  letter-spacing: -0.5px;
  color: var(--text-primary);
  margin: 0;
}

.close-btn {
  background: none;
  border: none;
  font-size: 28px;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 0;
  line-height: 1;
  transition: color 0.2s;
}

.close-btn:hover {
  color: var(--text-primary);
}

.slate-section {
  margin-bottom: 28px;
}

.section-label {
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 1px;
  color: var(--text-tertiary);
  margin-bottom: 12px;
  font-weight: 600;
}

.section-divider {
  height: 1px;
  background: var(--divider-color);
  margin: 24px 0;
}

.control-row {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.control-group {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.label-group {
  display: flex;
  flex-direction: column;
}

.control-label {
  font-size: 15px;
  color: var(--text-primary);
  font-weight: 500;
}

.control-sublabel {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-top: 2px;
}

/* Footer Container */
.slate-footer-group {
  margin-top: 40px;
  background: var(--footer-bg);
  border-radius: 20px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.slate-footer-item {
  cursor: pointer;
  transition: background 0.2s;
}

.slate-footer-item:hover {
  background: rgba(255, 255, 255, 0.1);
}

.slate-footer-item:active {
  background: rgba(255, 255, 255, 0.2);
}

.footer-divider {
  height: 1px;
  background: var(--divider-color);
  margin: 0 16px;
}

.slate-version {
  text-align: center;
  font-size: 11px;
  color: var(--text-tertiary);
  margin-top: 16px;
  opacity: 0.5;
  font-family: 'SF Mono', monospace;
}

.slate-copyright {
  text-align: center;
  font-size: 10px;
  color: var(--text-tertiary);
  margin-top: 4px;
  opacity: 0.4;
}

/* Segmented Control - Apple Style */
.segmented-control {
  background: var(--segment-bg);
  padding: 3px;
  border-radius: 9px;
  display: flex;
  height: 32px;
  min-width: 120px;
  position: relative;
  border: 1px solid var(--slate-border);
}

/* 苹果风格滑动指示器 */
.segmented-control::before {
  content: '';
  position: absolute;
  top: 3px;
  left: 3px;
  width: calc(50% - 3px);
  height: calc(100% - 6px);
  background: var(--segment-active-bg, rgba(255, 255, 255, 0.9));
  border-radius: 6px;
  box-shadow:
    0 1px 3px rgba(0, 0, 0, 0.08),
    0 1px 2px rgba(0, 0, 0, 0.04);
  transition: transform 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  z-index: 0;
}

.segmented-control.lang-en::before {
  transform: translateX(100%);
}

.segment-btn {
  flex: 1;
  background: transparent;
  border: none;
  color: var(--text-secondary);
  font-size: 13px;
  font-weight: 500;
  padding: 0 16px;
  border-radius: 6px;
  cursor: pointer;
  transition: color 0.2s;
  position: relative;
  z-index: 1;
  white-space: nowrap;
}

.segment-btn:hover {
  color: var(--text-primary);
}

.segment-btn.active {
  color: var(--text-primary);
  font-weight: 600;
}

/* 导入链接 - 低调设计 */
.slate-import-links {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 8px;
  margin-top: 32px;
  padding-top: 16px;
  border-top: 1px solid var(--divider-color);
}

.import-link {
  background: none;
  border: none;
  color: var(--text-tertiary);
  font-size: 11px;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;
  transition: all 0.2s;
  opacity: 0.6;
}

.import-link:hover {
  opacity: 1;
  color: var(--text-secondary);
  background: rgba(255, 255, 255, 0.05);
}

.import-link:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.import-divider {
  color: var(--text-tertiary);
  opacity: 0.3;
}
</style>
