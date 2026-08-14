import { ref, provide, inject, type Ref, type InjectionKey } from 'vue';
import type { HocrPage } from '../types/hocr';

export interface HocrContext {
  hocrPage: Ref<HocrPage | null>;
  machineName: Ref<string | null>;
  stem: Ref<string | null>;
  loading: Ref<boolean>;
  error: Ref<string | null>;
  loadHocr: (machineName: string, stem: string) => Promise<void>;
  rescanCarea: (machineName: string, stem: string, careaId: string, language?: string) => Promise<void>;
  updateHocr: (page: HocrPage | null) => void;
  clearHocr: () => void;
}

const HocrSymbol: InjectionKey<HocrContext> = Symbol('hocr');

export function provideHocrContext() {
  const hocrPage = ref<HocrPage | null>(null);
  const machineName = ref<string | null>(null);
  const stem = ref<string | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function loadHocr(mName: string, sName: string) {
    if (!mName || !sName) {
      hocrPage.value = null;
      machineName.value = null;
      stem.value = null;
      return;
    }
    loading.value = true;
    error.value = null;
    machineName.value = mName;
    stem.value = sName;
    try {
      const resp = await fetch(`/api/projects/${mName}/pages/${sName}/hocr-json`);
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

  async function rescanCarea(mName: string, sName: string, careaId: string, language = 'eng') {
    loading.value = true;
    error.value = null;
    try {
      const resp = await fetch(`/api/projects/${mName}/pages/${sName}/hocr/careas/${careaId}/rescan`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ language })
      });
      if (resp.ok) {
        const data = await resp.json();
        if (data && typeof data === 'object' && 'page' in data) {
          hocrPage.value = data.page as HocrPage;
        } else {
          hocrPage.value = data as HocrPage;
        }
      } else {
        error.value = `Rescan failed: ${await resp.text()}`;
      }
    } catch (e) {
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
    machineName.value = null;
    stem.value = null;
    error.value = null;
  }

  const context: HocrContext = {
    hocrPage,
    machineName,
    stem,
    loading,
    error,
    loadHocr,
    rescanCarea,
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
