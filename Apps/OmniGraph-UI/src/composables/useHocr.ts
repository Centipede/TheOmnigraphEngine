import { ref, provide, inject, type Ref, type InjectionKey } from 'vue';
import type { HocrPage, DetectionThresholds } from '../types/hocr';
import { augmentHocrPageWithWordCoords } from '../utils/hocr';

export interface LoadHocrOptions {
  isNoHocrAcceptable?: boolean;
  block_metrics?: boolean;
}

export interface HocrContext {
  hocrPage: Ref<HocrPage | null>;
  machineName: Ref<string | null>;
  stem: Ref<string | null>;
  loading: Ref<boolean>;
  error: Ref<string | null>;
  loadHocr: (machineName: string, stem: string, options?: LoadHocrOptions) => Promise<void>;
  rescanCarea: (machineName: string, stem: string, careaId: string, language?: string) => Promise<void>;
  rescanWord: (machineName: string, stem: string, wordId: string, language?: string) => Promise<void>;
  autoBridgePage: (thresholds: DetectionThresholds) => Promise<void>;
  autoBridgeBlock: (blockId: string, thresholds: DetectionThresholds) => Promise<void>;
  updateHocr: (page: HocrPage | null) => void;
  clearHocr: () => void;
}

const HocrSymbol: InjectionKey<HocrContext> = Symbol('hocr');

export async function fetchHocrPage(machineName: string, stem: string, options: LoadHocrOptions = { isNoHocrAcceptable: true }): Promise<HocrPage | null> {
  const params = new URLSearchParams();
  if (options.block_metrics) params.append('block_metrics', 'true');
  const query = params.toString();
  const url = `/api/projects/${machineName}/pages/${stem}/hocr-json${query ? `?${query}` : ''}`;

  const resp = await fetch(url);
  if (!resp.ok) {
    if (resp.status === 404 && options.isNoHocrAcceptable) {
      return null;
    }
    throw new Error(`Failed to load hOCR: ${resp.statusText}`);
  }
  const page = await resp.json() as HocrPage;
  return augmentHocrPageWithWordCoords(page);
}

export function provideHocrContext() {
  const hocrPage = ref<HocrPage | null>(null);
  const machineName = ref<string | null>(null);
  const stem = ref<string | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function loadHocr(mName: string, sName: string, options: LoadHocrOptions = { isNoHocrAcceptable: true }) {
    if (!mName || !sName) {
      hocrPage.value = null;
      machineName.value = null;
      stem.value = null;
      return;
    }
    hocrPage.value = null;
    loading.value = true;
    error.value = null;
    machineName.value = mName;
    stem.value = sName;
    try {
      hocrPage.value = await fetchHocrPage(mName, sName, options);
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
        let page: HocrPage;
        if (data && typeof data === 'object' && 'page' in data) {
          page = data.page as HocrPage;
        } else {
          page = data as HocrPage;
        }
        hocrPage.value = augmentHocrPageWithWordCoords(page);
      } else {
        error.value = `Rescan failed: ${await resp.text()}`;
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    } finally {
      loading.value = false;
    }
  }

  async function rescanWord(mName: string, sName: string, wordId: string, language = 'eng') {
    loading.value = true;
    error.value = null;
    try {
      const resp = await fetch(`/api/projects/${mName}/pages/${sName}/hocr/words/${wordId}/rescan`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ language })
      });
      if (resp.ok) {
        const data = await resp.json();
        let page: HocrPage;
        if (data && typeof data === 'object' && 'page' in data) {
          page = data.page as HocrPage;
        } else {
          page = data as HocrPage;
        }
        hocrPage.value = augmentHocrPageWithWordCoords(page);
      } else {
        error.value = `Rescan failed: ${await resp.text()}`;
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    } finally {
      loading.value = false;
    }
  }

  async function autoBridgePage(thresholds: DetectionThresholds) {
    if (!machineName.value || !stem.value) return;
    loading.value = true;
    error.value = null;
    try {
      const resp = await fetch(`/api/projects/${machineName.value}/pages/${stem.value}/auto-bridge-page`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ thresholds })
      });
      if (resp.ok) {
        hocrPage.value = await fetchHocrPage(machineName.value, stem.value, { isNoHocrAcceptable: false });
      } else {
        error.value = `Auto-bridge page failed: ${await resp.text()}`;
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    } finally {
      loading.value = false;
    }
  }

  async function autoBridgeBlock(blockId: string, thresholds: DetectionThresholds) {
    if (!machineName.value || !stem.value) return;
    loading.value = true;
    error.value = null;
    try {
      const resp = await fetch(`/api/projects/${machineName.value}/pages/${stem.value}/hocr/blocks/${blockId}/auto-bridge`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ thresholds })
      });
      if (resp.ok) {
        hocrPage.value = await fetchHocrPage(machineName.value, stem.value, { isNoHocrAcceptable: false });
      } else {
        error.value = `Auto-bridge block failed: ${await resp.text()}`;
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
    rescanWord,
    autoBridgePage,
    autoBridgeBlock,
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
