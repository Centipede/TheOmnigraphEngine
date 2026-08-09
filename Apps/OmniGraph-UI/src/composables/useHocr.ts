import { ref, provide, inject, type Ref, type InjectionKey } from 'vue';
import type { HocrPage } from '../types/hocr';

export interface HocrContext {
  hocrPage: Ref<HocrPage | null>;
  loading: Ref<boolean>;
  error: Ref<string | null>;
  loadHocr: (machineName: string, stem: string) => Promise<void>;
  updateHocr: (page: HocrPage | null) => void;
  clearHocr: () => void;
}

const HocrSymbol: InjectionKey<HocrContext> = Symbol('hocr');

export function provideHocrContext() {
  const hocrPage = ref<HocrPage | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function loadHocr(machineName: string, stem: string) {
    if (!machineName || !stem) {
      hocrPage.value = null;
      return;
    }
    loading.value = true;
    error.value = null;
    try {
      const resp = await fetch(`/api/projects/${machineName}/pages/${stem}/hocr-json`);
      if (resp.ok) {
        hocrPage.value = await resp.json() as HocrPage;
      } else {
        hocrPage.value = null;
        error.value = `Failed to load hOCR: ${resp.statusText}`;
      }
    } catch (e) {
      hocrPage.value = null;
      error.value = e instanceof Error ? e.message : String(e);
    } finally {
      loading.value = false;
    }
  }

  function updateHocr(page: HocrPage | null) {
    hocrPage.value = page;
  }

  function clearHocr() {
    hocrPage.value = null;
    error.value = null;
  }

  const context: HocrContext = {
    hocrPage,
    loading,
    error,
    loadHocr,
    updateHocr,
    clearHocr
  };

  provide(HocrSymbol, context);
  return context;
}

export function useHocrContext() {
  const context = inject(HocrSymbol);
  if (!context) {
    throw new Error('useHocrContext must be used within a component that calls provideHocrContext');
  }
  return context;
}
