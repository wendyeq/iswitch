<!--
  [INPUT]: source: openspec/changes/redesign-vendor-form/design.md ([POS]: Receptacle Design)
  [OUTPUT]: 智能接收槽组件，替代传统表单
  [PROTOCOL]: FractalFlow v1.0
  [POS]: iswitch-tauri/src/components/Main/VendorReceptacle.vue
-->
<template>
  <Transition name="receptacle-backdrop">
    <div v-if="open" class="receptacle-backdrop" @click.self="handleBackdropClick">
      <Transition name="receptacle-entry" @after-enter="onAfterEnter">
        <div
          v-if="open"
          class="receptacle-container"
          :class="{ 'shake-animation': shakeTrigger, 'is-expanded': isExpanded }"
          ref="containerRef"
          @click.stop
        >
          <!-- 核心输入槽 (The Slot) -->
          <div class="input-slot-wrapper">
            <!-- 动态图标 -->
            <div
              class="slot-icon"
              :class="{ 'cursor-pointer': officialSite }"
              @click="handleIconClick"
              :title="officialSite ? t('components.vendorReceptacle.visitDashboard') : ''"
            >
              <img
                v-if="detectedIcon"
                :src="detectedIcon"
                class="provider-favicon"
                alt="Provider Icon"
                @error="handleIconError"
              />
              <div v-else class="default-icon">
                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                  <path
                    d="M12 22C17.5228 22 22 17.5228 22 12C22 6.47715 17.5228 2 12 2C6.47715 2 2 6.47715 2 12C2 17.5228 2 22 12 22Z"
                    stroke="currentColor"
                    stroke-width="2"
                  />
                  <path d="M12 8V16" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
                  <path d="M8 12H16" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
                </svg>
              </div>
            </div>

            <!-- 主输入框 -->
            <input
              ref="inputRef"
              type="text"
              v-model="inputUrl"
              class="slot-input"
              :placeholder="t('components.vendorReceptacle.placeholder')"
              @input="onUrlInput"
              @keydown.enter="handleEnter"
              @keydown.esc="close"
              autocomplete="off"
              autofocus
            />

            <!-- 状态指示灯/按钮 -->
            <div class="slot-action">
              <Transition name="fade">
                <button
                  v-if="canSubmit"
                  class="action-btn submit-btn"
                  @click="submit"
                  :title="t('components.vendorReceptacle.add')"
                >
                  <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
                    <path
                      d="M5 12L10 17L19 8"
                      stroke="currentColor"
                      stroke-width="3"
                      stroke-linecap="round"
                      stroke-linejoin="round"
                    />
                  </svg>
                </button>
                <div v-else-if="isValidating" class="spinner"></div>
              </Transition>
            </div>
          </div>

          <!-- 智能反馈区 (Smart Feedback) -->
          <Transition name="slide-down">
            <div v-if="feedbackMessage" class="smart-feedback" :class="feedbackType">
              {{ feedbackMessage }}
              <span v-if="autoCorrectedUrl" class="correction-hint"> ({{ autoCorrectedUrl }}) </span>
            </div>
          </Transition>

          <!-- 渐进式展开区域 (Progressive Disclosure) -->
          <Transition name="expand">
            <div v-if="isExpanded" class="details-panel">
              <div class="field-row">
                <label>{{ t('components.vendorReceptacle.name') }}</label>
                <input type="text" v-model="form.name" class="glass-input" />
              </div>

              <div class="field-row">
                <label>{{ t('components.vendorReceptacle.apiKey') }}</label>
                <div class="input-with-action">
                  <input
                    type="password"
                    v-model="form.apiKey"
                    class="glass-input"
                    placeholder="sk-..."
                    ref="apiKeyInputRef"
                    @keydown.enter="handleEnter"
                  />
                  <Transition name="fade">
                    <div v-if="form.apiKey.length > 5" class="enter-sub-action" @click="submit" title="Save">
                      <span class="enter-key">↵</span>
                    </div>
                  </Transition>
                </div>
              </div>

              <!-- Advanced Config Toggle -->
              <div class="advanced-toggle" @click="showAdvanced = !showAdvanced">
                <span>{{ t('components.vendorReceptacle.advanced') }}</span>
                <svg :class="{ rotated: showAdvanced }" viewBox="0 0 24 24" width="16" height="16">
                  <path
                    d="M6 9L12 15L18 9"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    fill="none"
                  />
                </svg>
              </div>

              <Transition name="accordion">
                <div v-if="showAdvanced" class="advanced-config">
                  <div class="field-row">
                    <label>{{ t('components.vendorReceptacle.officialSite') }}</label>
                    <input type="text" v-model="form.officialSite" class="glass-input" placeholder="https://..." />
                  </div>
                  <div class="field-row">
                    <label>{{ t('components.vendorReceptacle.mapping') }}</label>
                    <div class="mapping-list">
                      <transition-group name="list">
                        <div v-for="(item, index) in mappingPairs" :key="index" class="mapping-item">
                          <input
                            v-model="item.from"
                            :placeholder="t('components.vendorReceptacle.mappingFrom')"
                            class="glass-input small"
                          />
                          <div class="mapping-arrow">
                            <svg width="12" height="12" viewBox="0 0 24 24" fill="none">
                              <path
                                d="M5 12H19M19 12L12 5M19 12L12 19"
                                stroke="currentColor"
                                stroke-width="2"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                              />
                            </svg>
                          </div>
                          <input
                            v-model="item.to"
                            :placeholder="t('components.vendorReceptacle.mappingTo')"
                            class="glass-input small"
                          />
                          <button
                            @click="removeMapping(index)"
                            class="remove-item-btn"
                            :title="t('components.vendorReceptacle.remove')"
                          >
                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
                              <path
                                d="M18 6L6 18M6 6L18 18"
                                stroke="currentColor"
                                stroke-width="2"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                              />
                            </svg>
                          </button>
                        </div>
                      </transition-group>
                      <button @click="addMapping" class="add-item-btn">
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
                          <path
                            d="M12 5V19M5 12H19"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                          />
                        </svg>
                        {{ t('components.vendorReceptacle.addMapping') }}
                      </button>
                    </div>
                  </div>
                </div>
              </Transition>

              <div v-if="manualExpand" class="collapse-toggle" @click="manualExpand = false">
                <span>{{ t('components.vendorReceptacle.hideDetails') }}</span>
              </div>
            </div>
          </Transition>

          <!-- Expansion Indicator (when collapsed) -->
          <Transition name="fade">
            <div v-if="!isExpanded && inputUrl && !isAutoLocking" class="expansion-hint" @click="manualExpand = true">
              <span>{{ t('components.vendorReceptacle.showDetails') }}</span>
              <svg viewBox="0 0 24 24" width="14" height="14">
                <path
                  d="M6 9L12 15L18 9"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  fill="none"
                />
              </svg>
            </div>
          </Transition>
        </div>
      </Transition>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onUnmounted, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { open as openUrl } from '@tauri-apps/plugin-shell';
import { detectProvider } from '../../utils/providerDetector';
import { getProviderDefaultUrl, getProviderDefaultMapping } from '../../utils/providerDefaults';
import { ProviderType } from '../../types/provider';
import type { DetectionResult } from '../../types/provider';

const props = defineProps<{
  open: boolean;
  initialTab?: string;
  initialData?: any; // For editing mode
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'submit', data: any): void;
}>();

const { t } = useI18n();

// State

const inputRef = ref<HTMLInputElement | null>(null);
const apiKeyInputRef = ref<HTMLInputElement | null>(null);
const inputUrl = ref('');
const isValidating = ref(false);
const shakeTrigger = ref(false);
const showAdvanced = ref(false);
const isAutoLocking = ref(false);
const manualExpand = ref(false);
const mappingPairs = ref<{ from: string; to: string }[]>([]);

const onGlobalKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Escape') {
    close();
  }
};

// Timers
let autoSubmitTimer: number | undefined;

// Form Data
const form = reactive({
  name: '',
  apiKey: '',
  icon: '',
  officialSite: '',
  type: ProviderType.UNKNOWN,
});

// Detection State
const detection = ref<DetectionResult | null>(null);
const detectedIcon = computed(() => detection.value?.icon || form.icon);

watch(
  () => props.open,
  (newVal: boolean) => {
    if (newVal) {
      window.addEventListener('keydown', onGlobalKeydown);
      if (props.initialData) {
        // Edit Mode Initialization
        inputUrl.value = props.initialData.apiUrl || '';
        form.name = props.initialData.name || '';
        form.apiKey = props.initialData.apiKey || '';
        form.icon = props.initialData.icon || '';
        form.officialSite = props.initialData.officialSite || '';

        if (props.initialData.modelMapping) {
          const raw = props.initialData.modelMapping;
          mappingPairs.value = Object.entries(raw).map(([from, to]) => ({ from, to: String(to) }));
        } else {
          mappingPairs.value = [];
        }

        if (inputUrl.value) {
          const result = detectProvider(inputUrl.value, props.initialTab);
          detection.value = result;
          // If editing, prefer stored dashboard URL over detected one
          if (!form.officialSite && result.officialSite) {
            form.officialSite = result.officialSite;
          }
        }
      } else {
        resetState();
        // Force focus for new entry
        setFocus();
      }
    } else {
      window.removeEventListener('keydown', onGlobalKeydown);
    }
  }
);

const isExpanded = computed(() => {
  // Always expand if editing
  if (props.initialData) return true;
  if (manualExpand.value) return true;

  // 自动展开条件：已有输入但置信度低，或者未识别
  if (!inputUrl.value) return false;
  return (
    detection.value?.isAmbiguous ||
    detection.value?.type === ProviderType.UNKNOWN ||
    (inputUrl.value.length > 0 && !detection.value && !isAutoLocking.value)
  );
});

const feedbackMessage = computed(() => {
  if (isAutoLocking.value) return t('components.vendorReceptacle.autoAdding');
  if (detection.value?.isAutoCorrected) return t('components.vendorReceptacle.autoCorrected');
  if (detection.value?.isAmbiguous) return t('components.vendorReceptacle.ambiguous');
  return '';
});

const feedbackType = computed(() => {
  if (isAutoLocking.value) return 'info'; // or success
  if (detection.value?.isAutoCorrected) return 'info';
  if (detection.value?.isAmbiguous) return 'warning';
  return '';
});

const autoCorrectedUrl = computed(() => detection.value?.correctedUrl);

const canSubmit = computed(() => {
  return (
    inputUrl.value.length > 0 &&
    form.apiKey.length > 0 &&
    ((detection.value && !detection.value.isAmbiguous) || form.name.length > 0) // Allows manual override
  );
});

const officialSite = computed(() => form.officialSite || detection.value?.officialSite);

// Focus management
const setFocus = () => {
  // Attempt 1: Vue ref
  inputRef.value?.focus();

  // Attempt 2: DOM query fallback (sometimes refs are slow in transitions)
  setTimeout(() => {
    const el = document.querySelector('.slot-input') as HTMLInputElement;
    if (el) el.focus();
  }, 100);
};

const onAfterEnter = () => {
  setFocus();
};

// Logic
const handleBackdropClick = () => {
  close();
};

const close = () => {
  emit('close');
  // Reset state after transition
  setTimeout(() => {
    resetState();
  }, 300);
};

const resetState = () => {
  inputUrl.value = '';
  form.name = '';
  form.apiKey = '';
  form.icon = '';
  form.officialSite = '';
  form.type = ProviderType.UNKNOWN;
  mappingPairs.value = [];
  detection.value = null;
  isAutoLocking.value = false;
  manualExpand.value = false;
  showAdvanced.value = false;
  window.clearTimeout(autoSubmitTimer);
};

const onUrlInput = () => {
  window.clearTimeout(autoSubmitTimer);
  isAutoLocking.value = false;

  if (!inputUrl.value) {
    detection.value = null;
    return;
  }

  // Debounce detection slightly? No, fast feedback is better for "Aliveness"
  const result = detectProvider(inputUrl.value, props.initialTab);

  detection.value = result;

  if (result.type !== ProviderType.UNKNOWN && !form.name) {
    form.name = result.name;
    form.type = result.type;
    if (!form.officialSite) {
      form.officialSite = result.officialSite || '';
    }

    // Auto-fill Mapping if empty
    if (mappingPairs.value.length === 0) {
      const defaultMapping = getProviderDefaultMapping(result.type);
      if (defaultMapping) {
        mappingPairs.value = Object.entries(defaultMapping).map(([from, to]) => ({ from, to }));
      }
    }

    // Auto-fill logic for Built-in Providers (Magic Keyword Expansion)
    const defaultUrl = getProviderDefaultUrl(result.type);
    if (defaultUrl && !inputUrl.value.startsWith('http') && inputUrl.value.length < 50) {
      // User typed "Deepseek" -> Expand to URL
      inputUrl.value = defaultUrl;
      // Update detection immediately
      const newResult = detectProvider(defaultUrl, props.initialTab);
      detection.value = newResult;
      form.name = newResult.name;
      form.type = newResult.type;

      // Update Mapping again for the new resolved type
      const newMapping = getProviderDefaultMapping(newResult.type);
      if (newMapping && mappingPairs.value.length === 0) {
        mappingPairs.value = Object.entries(newMapping).map(([from, to]) => ({ from, to }));
      }

      // Force expand so user can enter Key
      manualExpand.value = true;

      // Focus on API Key input
      // Using setTimeout to allow transition to start/render
      setTimeout(() => {
        apiKeyInputRef.value?.focus();
      }, 100);

      return; // Stop further processing for this cycle
    }

    // 如果是高置信度，自动填充更多信息

    // Zero-Click Logic: If confidence is high (1.0), schedule auto-submit
    // BUT: Only if not auto-corrected (safety) and if it's a known provider that might not need keys (rare)
    // Actually, for better UX: if it's a high confidence match, don't auto-submit unless apiKey is filled?
    // Let's stick to Design: Only if high confidence AND not corrected.
    if (result.confidence >= 1.0 && !result.isAmbiguous && !result.isAutoCorrected) {
      startAutoSubmit();
    }
  }
};

const startAutoSubmit = () => {
  isAutoLocking.value = true;
  autoSubmitTimer = window.setTimeout(() => {
    if (canSubmit.value) {
      submit();
    }
  }, 1200); // Slightly longer for "Aliveness" feel
};

const addMapping = () => {
  mappingPairs.value.push({ from: '', to: '' });
};

const removeMapping = (index: number) => {
  mappingPairs.value.splice(index, 1);
};

const handleIconClick = async () => {
  if (officialSite.value) {
    try {
      await openUrl(officialSite.value);
    } catch (e) {
      console.error('Failed to open dashboard', e);
    }
  }
};

const handleIconError = () => {
  // If favicon fails, fallback to default or clear it to show SVG
  if (detection.value) {
    detection.value.icon = undefined;
  }
  form.icon = '';
};

const triggerShake = () => {
  shakeTrigger.value = true;
  setTimeout(() => {
    shakeTrigger.value = false;
  }, 500);
};

const handleEnter = () => {
  if (canSubmit.value) {
    submit();
  } else {
    triggerShake();
  }
};

const submit = () => {
  if (!canSubmit.value) {
    triggerShake();
    return;
  }

  // Build mapping from pairs
  const mapping: Record<string, string> = {};
  mappingPairs.value.forEach(p => {
    if (p.from && p.to) {
      mapping[p.from] = p.to;
    }
  });

  const submission = {
    name: form.name || detection.value?.name || 'Custom Provider',
    apiUrl: autoCorrectedUrl.value || inputUrl.value,
    apiKey: form.apiKey,
    icon: detectedIcon.value || 'aicoding', // fallback
    enabled: true,
    modelMapping: mapping,
    officialSite: form.officialSite || detection.value?.officialSite || '', // Map to officialSite
    type: detection.value?.type || ProviderType.UNKNOWN,
  };

  emit('submit', submission);
  close();
};

onUnmounted(() => {
  window.clearTimeout(autoSubmitTimer);
});
</script>

<style scoped>
/* Core Variables */
.receptacle-container {
  --radius-outer: 24px;
  --padding: 20px;
  /* Concentric corner math: Inner = Outer - Padding */
  --radius-inner: calc(var(--radius-outer) - var(--padding));
  --glass-bg: rgba(255, 255, 255, 0.85);
  --glass-border: rgba(255, 255, 255, 0.4);
  --shadow-color: rgba(0, 0, 0, 0.08);
  --accent-color: #007aff;
  --text-color: #1d1d1f;

  position: fixed;
  top: 15%;
  left: 50%;
  transform: translateX(-50%);
  width: 480px;
  max-width: 90vw;
  background: var(--glass-bg);
  backdrop-filter: blur(20px) saturate(180%);
  border: 1px solid var(--glass-border);
  border-radius: var(--radius-outer);
  box-shadow:
    0 4px 6px -1px var(--shadow-color),
    0 12px 32px -4px var(--shadow-color),
    inset 0 1px 0 rgba(255, 255, 255, 0.6);
  padding: var(--padding);
  z-index: 1000;
  display: flex;
  flex-direction: column;
  gap: 16px;

  /* Physical Lighting: Top highlight */
  box-shadow:
    0 1px 0 rgba(255, 255, 255, 0.9) inset,
    0 20px 40px -8px rgba(0, 0, 0, 0.15);
}

.receptacle-backdrop {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: rgba(0, 0, 0, 0.4); /* Darker backdrop */
  z-index: 999;
  backdrop-filter: blur(4px);
}

/* Input Slot */
.input-slot-wrapper {
  display: flex;
  align-items: center;
  background: rgba(0, 0, 0, 0.03);
  border-radius: var(--radius-inner);
  padding: 8px 12px;
  transition: all 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
  border: 1px solid transparent;
}

.input-slot-wrapper:focus-within {
  background: rgba(255, 255, 255, 1);
  box-shadow: 0 0 0 4px rgba(0, 122, 255, 0.15);
  border-color: var(--accent-color);
  transform: scale(1.005);
}

.input-with-action {
  position: relative;
  width: 100%;
}

.enter-sub-action {
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  background: var(--accent-color);
  color: white;
  width: 24px;
  height: 24px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  font-size: 14px;
  font-weight: bold;
  box-shadow: 0 2px 5px rgba(0, 0, 0, 0.1);
  transition: all 0.2s ease;
}

.enter-sub-action:hover {
  transform: translateY(-50%) scale(1.05);
  background: #0062cc;
}

.enter-sub-action:active {
  transform: translateY(-50%) scale(0.95);
}

.slot-input {
  flex: 1;
  background: transparent;
  border: none;
  font-size: 16px;
  color: var(--text-color);
  padding: 8px;
  outline: none;
  font-family: inherit;
}

.slot-icon {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #888;
  margin-right: 8px;
}

.provider-favicon {
  width: 24px;
  height: 24px;
  border-radius: 4px;
}

/* Action Button */
.action-btn {
  background: var(--accent-color);
  color: white;
  border: none;
  border-radius: 50%;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: transform 0.2s;
}

.action-btn:hover {
  transform: scale(1.1);
}

.action-btn:active {
  transform: scale(0.95);
}

/* Smart Feedback */
.smart-feedback {
  font-size: 13px;
  padding: 0 12px;
  margin-top: -8px;
}

.smart-feedback.warning {
  color: #ff9500;
}

.smart-feedback.info {
  color: var(--accent-color);
}

/* Details Panel */
.details-panel {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding-top: 8px;
  border-top: 1px solid rgba(0, 0, 0, 0.05);
}

.field-row label {
  display: block;
  font-size: 12px;
  color: #888;
  margin-bottom: 4px;
}

.glass-input,
.glass-textarea {
  width: 100%;
  background: rgba(0, 0, 0, 0.03);
  border: 1px solid transparent;
  padding: 8px 12px;
  border-radius: 8px;
  color: var(--text-color);
  font-size: 14px;
  outline: none;
  transition: all 0.2s;
}

.glass-input:focus,
.glass-textarea:focus {
  background: rgba(255, 255, 255, 1);
  border-color: var(--accent-color);
  box-shadow: 0 0 0 3px rgba(0, 122, 255, 0.1);
}

@media (prefers-color-scheme: dark) {
  .glass-input,
  .glass-textarea {
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.08);
    color: var(--text-color);
  }

  .glass-input:focus,
  .glass-textarea:focus {
    background: rgba(255, 255, 255, 0.1);
    border-color: var(--accent-color);
    box-shadow: 0 0 0 3px rgba(0, 122, 255, 0.25);
  }

  .field-row label {
    color: #999;
  }
}

/* Mapping List Styles */
.mapping-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 4px;
}

.mapping-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.glass-input.small {
  padding: 6px 10px;
  font-size: 13px;
  flex: 1;
}

.mapping-arrow {
  color: #888;
  opacity: 0.5;
  display: flex;
  align-items: center;
}

.add-item-btn {
  background: none;
  border: 1px dashed rgba(0, 122, 255, 0.3);
  color: var(--accent-color);
  padding: 6px;
  border-radius: 6px;
  font-size: 12px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  margin-top: 4px;
  transition: all 0.2s;
}

.add-item-btn:hover {
  background: rgba(0, 122, 255, 0.05);
  border-color: var(--accent-color);
}

.remove-item-btn {
  background: none;
  border: none;
  color: #ff3b30;
  padding: 4px;
  cursor: pointer;
  opacity: 0.6;
  transition: opacity 0.2s;
}

.remove-item-btn:hover {
  opacity: 1;
}

/* List Transitions */
.list-enter-active,
.list-leave-active {
  transition: all 0.3s ease;
}
.list-enter-from {
  opacity: 0;
  transform: translateX(-10px);
}
.list-leave-to {
  opacity: 0;
  transform: translateX(10px);
}

/* Animations */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.expansion-hint {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  font-size: 11px;
  color: #888;
  cursor: pointer;
  padding: 4px;
  margin-top: -8px;
  opacity: 0.7;
  transition: opacity 0.2s;
}

.expansion-hint:hover {
  opacity: 1;
}

.collapse-toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  color: #888;
  cursor: pointer;
  margin-top: 8px;
  opacity: 0.6;
}

.collapse-toggle:hover {
  opacity: 1;
}

.advanced-toggle {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--accent-color);
  cursor: pointer;
  opacity: 0.8;
  margin-top: 4px;
}

.advanced-toggle:hover {
  opacity: 1;
}

.advanced-toggle svg {
  transition: transform 0.2s;
}

.advanced-toggle svg.rotated {
  transform: rotate(180deg);
}

.advanced-config {
  margin-top: 12px;
}

.receptacle-backdrop-enter-active,
.receptacle-backdrop-leave-active {
  transition: opacity 0.3s ease;
}

.receptacle-backdrop-enter-from,
.receptacle-backdrop-leave-to {
  opacity: 0;
}

.receptacle-entry-enter-active {
  transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1);
}

.receptacle-entry-leave-active {
  transition: all 0.2s ease-in;
}

.receptacle-entry-enter-from {
  opacity: 0;
  transform: translateX(-50%) translateY(20px) scale(0.95);
}

.receptacle-entry-leave-to {
  opacity: 0;
  transform: translateX(-50%) scale(0.95) blur(10px);
}

/* Expand Animation */
.expand-enter-active,
.expand-leave-active {
  transition: all 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
  max-height: 300px;
  opacity: 1;
  overflow: hidden;
}

.expand-enter-from,
.expand-leave-to {
  max-height: 0;
  opacity: 0;
}

/* Shake Animation */
.shake-animation {
  animation: shake 0.5s cubic-bezier(0.36, 0.07, 0.19, 0.97) both;
}

@keyframes shake {
  10%,
  90% {
    transform: translateX(calc(-50% - 1px));
  }
  20%,
  80% {
    transform: translateX(calc(-50% + 2px));
  }
  30%,
  50%,
  70% {
    transform: translateX(calc(-50% - 4px));
  }
  40%,
  60% {
    transform: translateX(calc(-50% + 4px));
  }
}
</style>

<!-- Global Style Patch for Dark Mode Overrides -->
<!-- Using non-scoped style to ensure selectors match correctly against html.dark -->
<style>
html.dark .receptacle-container {
  --glass-bg: rgba(30, 30, 30, 0.85) !important;
  --glass-border: rgba(255, 255, 255, 0.12) !important;
  --shadow-color: rgba(0, 0, 0, 0.5) !important;
  --text-color: #f5f5f7 !important;

  background: #1e1e1e !important;
  background: var(--glass-bg) !important;
  box-shadow:
    0 1px 0 rgba(255, 255, 255, 0.08) inset,
    0 0 0 1px rgba(0, 0, 0, 0.4),
    0 24px 48px -12px rgba(0, 0, 0, 0.6) !important;
}

html.dark .receptacle-backdrop {
  background: rgba(0, 0, 0, 0.6) !important;
  backdrop-filter: blur(4px) !important;
}

html.dark .input-slot-wrapper {
  background: rgba(255, 255, 255, 0.05) !important;
}

html.dark .input-slot-wrapper:focus-within {
  background: rgba(40, 40, 40, 1) !important;
}

html.dark .details-panel {
  border-top: 1px solid rgba(255, 255, 255, 0.05) !important;
}

html.dark .glass-input,
html.dark .glass-textarea {
  background: rgba(255, 255, 255, 0.06) !important;
  border: 1px solid rgba(255, 255, 255, 0.08) !important;
  color: #f5f5f7 !important;
}

html.dark .glass-input:focus,
html.dark .glass-textarea:focus {
  background: rgba(255, 255, 255, 0.1) !important;
  border-color: #007aff !important;
  box-shadow: 0 0 0 3px rgba(0, 122, 255, 0.25) !important;
}

html.dark .field-row label {
  color: #999 !important;
}
</style>
