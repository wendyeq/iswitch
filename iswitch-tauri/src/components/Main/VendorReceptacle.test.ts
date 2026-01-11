/**
 * [INPUT]: source: iswitch-tauri/src/components/Main/VendorReceptacle.vue
 * [OUTPUT]: VendorReceptacle Component Test
 * [PROTOCOL]: FractalFlow v1.0
 * [POS]: iswitch-tauri/src/components/Main/VendorReceptacle.test.ts
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import { nextTick } from 'vue';
import VendorReceptacle from './VendorReceptacle.vue';
import { ProviderType } from '../../types/provider';

// Mock i18n
vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string) => key,
  }),
}));

// Mock Tauri Shell
const mocks = vi.hoisted(() => ({
  open: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-shell', () => ({
  open: mocks.open,
}));

// Mock providerDetector to control test conditions
// Actually, using real one is fine for comprehensive test, but mocking allows testing "Loading" or "Ambiguous" states easily.
// Let's use real one for now, as it's simple logic.
// If we needed to mock, we would do:
// vi.mock('../../utils/providerDetector', () => ({ detectProvider: vi.fn(...) }))

describe('VendorReceptacle', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
    vi.useRealTimers();
  });

  it('renders correctly when open', () => {
    const wrapper = mount(VendorReceptacle, {
      props: { open: true },
    });
    expect(wrapper.find('.receptacle-container').exists()).toBe(true);
    expect(wrapper.find('input.slot-input').exists()).toBe(true);
  });

  it('does not render when closed', () => {
    const wrapper = mount(VendorReceptacle, {
      props: { open: false },
    });
    // Transition stub might leave element for a bit, but v-if="open" on container
    // However, the transition wraps the backdrop.
    expect(wrapper.find('.receptacle-container').exists()).toBe(false);
  });

  it('detects provider on input', async () => {
    const wrapper = mount(VendorReceptacle, {
      props: { open: true },
    });

    const input = wrapper.find('input.slot-input');
    // Using a keyword like 'Deepseek' triggers auto-expansion and URL fill
    await input.setValue('Deepseek');

    // Wait for keyword expansion logic and focus shift
    await flushPromises();
    await vi.dynamicImportSettled(); // Ensure transitions/focus are processed

    // The URL should have been expanded to the default Deepseek URL
    expect((input.element as HTMLInputElement).value).toContain('api.deepseek.com');

    // Now we enter the API key
    const apiKeyInput = wrapper.find('input[type="password"]');
    await apiKeyInput.setValue('sk-test-key-123');

    // Trigger submit
    await input.trigger('keydown.enter');

    expect(wrapper.emitted('submit')).toBeTruthy();
    const emitData = wrapper.emitted('submit')![0][0] as any;
    expect(emitData.type).toBe(ProviderType.DEEPSEEK);
    expect(emitData.apiKey).toBe('sk-test-key-123');
  });

  it('shows advanced options toggle', async () => {
    const wrapper = mount(VendorReceptacle, {
      props: { open: true },
    });
    // Force expand by typing something ambiguous
    const input = wrapper.find('input.slot-input');
    await input.setValue('https://unknown-service.com');
    await flushPromises();

    // Assuming ambiguous input triggers expansion logic in computed prop
    // "detection.value?.type === ProviderType.UNKNOWN" -> isExpanded true

    expect(wrapper.find('.details-panel').exists()).toBe(true);

    // Toggle advanced
    const advancedToggle = wrapper.find('.advanced-toggle');
    expect(advancedToggle.exists()).toBe(true);

    await advancedToggle.trigger('click');
    expect(wrapper.find('.advanced-config').exists()).toBe(true);
  });

  it('emits close on backdrop click', async () => {
    const wrapper = mount(VendorReceptacle, {
      props: { open: true },
    });

    await wrapper.find('.receptacle-backdrop').trigger('click.self'); // modifiers handled by vue-test-utils?
    // .self modifier simulation usually requires checking event target manually if not using explicit trigger options
    // But wrapper.trigger('click.self') isn't standard in vtu v2?
    // Actually standard VTU trigger respects modifiers if supported or just fires event.
    // The component has @click.self="handleBackdropClick"

    // We can simulate validation by firing click on backdrop directly
    await wrapper.find('.receptacle-backdrop').trigger('click');

    expect(wrapper.emitted('close')).toBeTruthy();
  });

  it('handles smart link click', async () => {
    const wrapper = mount(VendorReceptacle, { props: { open: true } });
    const input = wrapper.find('input.slot-input');

    // OpenAI URL has a dashboard link
    await input.setValue('https://api.openai.com/v1');
    await flushPromises();

    // Find icon
    const iconBtn = wrapper.find('.slot-icon');
    await iconBtn.trigger('click');

    expect(mocks.open).toHaveBeenCalled();
  });

  it('supports advanced mapping workflow and submit emits composed mapping', async () => {
    const wrapper = mount(VendorReceptacle, { props: { open: true } });
    const vm = wrapper.vm as unknown as Record<string, any>;

    vm.manualExpand = true;
    vm.showAdvanced = true;
    await nextTick();

    const addBtn = wrapper.find('.add-item-btn');
    await addBtn.trigger('click');
    await nextTick();

    let mappingInputs = wrapper.findAll('.mapping-item input');
    expect(mappingInputs).toHaveLength(2);
    await mappingInputs[0].setValue('claude-*');
    await mappingInputs[1].setValue('glm-4.7');

    const removeBtn = wrapper.find('.mapping-item .remove-item-btn');
    await removeBtn.trigger('click');
    await nextTick();
    expect(wrapper.findAll('.mapping-item')).toHaveLength(0);

    vm.addMapping();
    await nextTick();
    mappingInputs = wrapper.findAll('.mapping-item input');
    await mappingInputs[0].setValue('claude-*');
    await mappingInputs[1].setValue('glm-4.7');

    vm.form.name = 'Manual Provider';
    vm.form.apiKey = 'sk-abc';
    vm.inputUrl = 'https://api.manual.dev';
    await nextTick();

    vm.submit();
    const submission = wrapper.emitted('submit')?.[0]?.[0] as any;
    expect(submission.modelMapping).toEqual({ 'claude-*': 'glm-4.7' });
    expect(submission.apiUrl).toBe('https://api.manual.dev');
  });

  it('prefills edit mode form and mapping when initialData 提供', async () => {
    const wrapper = mount(VendorReceptacle, {
      props: {
        open: false,
        initialData: {
          apiUrl: 'https://api.edit.dev/v1',
          name: 'Editable',
          apiKey: 'sk-edit',
          modelMapping: { 'gpt-*': 'glm' },
        },
      },
    });
    await wrapper.setProps({ open: true });
    await flushPromises();
    const vm = wrapper.vm as unknown as Record<string, any>;

    expect(vm.form.name).toBe('Editable');
    expect(vm.form.apiKey).toBe('sk-edit');
    expect(vm.mappingPairs.length).toBe(1);
    expect(vm.isExpanded).toBe(true);
  });

  it('展示歧义/自动修正提示并渲染纠正链接', async () => {
    const wrapper = mount(VendorReceptacle, { props: { open: true } });
    const vm = wrapper.vm as unknown as Record<string, any>;

    vm.detection = {
      isAmbiguous: true,
      name: 'Unknown',
      type: ProviderType.UNKNOWN,
    };
    await nextTick();
    const warning = wrapper.find('.smart-feedback');
    expect(warning.exists()).toBe(true);
    expect(warning.classes()).toContain('warning');

    vm.detection = {
      isAmbiguous: false,
      isAutoCorrected: true,
      correctedUrl: 'https://resolved.example',
      name: 'Resolved',
      type: ProviderType.OPENAI,
    };
    await nextTick();
    const info = wrapper.find('.smart-feedback');
    expect(info.text()).toContain('components.vendorReceptacle.autoCorrected');
    expect(wrapper.html()).toContain('https://resolved.example');
  });

  it('startAutoSubmit 锁定输入并在可提交时触发 submit', async () => {
    vi.useFakeTimers();
    const wrapper = mount(VendorReceptacle, { props: { open: true } });
    const vm = wrapper.vm as unknown as Record<string, any>;
    vm.inputUrl = 'https://api.openai.com/v1';
    vm.form.apiKey = 'sk-123456789';
    vm.detection = {
      type: ProviderType.OPENAI,
      name: 'OpenAI',
      isAmbiguous: false,
      isAutoCorrected: false,
      confidence: 1,
    };

    vm.startAutoSubmit();
    expect(vm.isAutoLocking).toBe(true);

    await vi.advanceTimersByTimeAsync(1250);
    expect(wrapper.emitted('submit')).toBeTruthy();
  });

  it('handleIconError 清理检测图标并保留默认 SVG', () => {
    const wrapper = mount(VendorReceptacle, { props: { open: true } });
    const vm = wrapper.vm as unknown as Record<string, any>;
    vm.detection = { icon: 'data:image/png;base64,x', type: ProviderType.OPENAI };
    vm.form.icon = 'custom-icon';

    vm.handleIconError();

    expect(vm.form.icon).toBe('');
    expect(vm.detection.icon).toBeUndefined();
  });

  it('handleEnter 在不可提交时触发 shake', async () => {
    vi.useFakeTimers();
    const wrapper = mount(VendorReceptacle, { props: { open: true } });
    const vm = wrapper.vm as unknown as Record<string, any>;

    vm.inputUrl = '';
    vm.form.apiKey = '';
    vm.handleEnter();
    expect(vm.shakeTrigger).toBe(true);

    await vi.advanceTimersByTimeAsync(500);
    expect(vm.shakeTrigger).toBe(false);
  });

  it('open 切换为 false 时会移除 keydown 监听', async () => {
    const addSpy = vi.spyOn(window, 'addEventListener');
    const removeSpy = vi.spyOn(window, 'removeEventListener');
    const wrapper = mount(VendorReceptacle, { props: { open: false } });
    await wrapper.setProps({ open: true });
    expect(addSpy).toHaveBeenCalledWith('keydown', expect.any(Function));
    await wrapper.setProps({ open: false });
    expect(removeSpy).toHaveBeenCalledWith('keydown', expect.any(Function));
    addSpy.mockRestore();
    removeSpy.mockRestore();
  });

  it('expansion hint 控制 manualExpand，collapse toggle 可折叠', async () => {
    const wrapper = mount(VendorReceptacle, { props: { open: true } });
    const vm = wrapper.vm as unknown as Record<string, any>;
    vm.inputUrl = 'https://api.openai.com/v1';
    vm.detection = { type: ProviderType.OPENAI, isAmbiguous: false };
    vm.manualExpand = false;
    vm.isAutoLocking = false;
    await nextTick();

    const hint = wrapper.find('.expansion-hint');
    expect(hint.exists()).toBe(true);
    await hint.trigger('click');
    await nextTick();
    expect(vm.manualExpand).toBe(true);

    const collapse = wrapper.find('.collapse-toggle');
    expect(collapse.exists()).toBe(true);
    await collapse.trigger('click');
    expect(vm.manualExpand).toBe(false);
  });

  it('resetState 会清空输入、映射与检测状态', () => {
    const wrapper = mount(VendorReceptacle, { props: { open: true } });
    const vm = wrapper.vm as unknown as Record<string, any>;
    vm.inputUrl = 'https://api.manual.dev';
    vm.form.name = 'Manual';
    vm.form.apiKey = 'sk-xxx';
    vm.mappingPairs = [{ from: 'a', to: 'b' }];
    vm.detection = { type: ProviderType.OPENAI };
    vm.manualExpand = true;
    vm.isAutoLocking = true;

    vm.resetState();
    expect(vm.inputUrl).toBe('');
    expect(vm.form.name).toBe('');
    expect(vm.form.apiKey).toBe('');
    expect(vm.mappingPairs).toHaveLength(0);
    expect(vm.detection).toBeNull();
    expect(vm.manualExpand).toBe(false);
    expect(vm.isAutoLocking).toBe(false);
  });

  it('onUrlInput 在清空输入时会重置 detection', () => {
    const wrapper = mount(VendorReceptacle, { props: { open: true } });
    const vm = wrapper.vm as unknown as Record<string, any>;
    vm.inputUrl = '';
    vm.detection = { type: ProviderType.OPENAI };

    vm.onUrlInput();
    expect(vm.detection).toBeNull();
  });
});
