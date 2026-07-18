import { reactive, watch } from 'vue';
import type { PanelVisibility } from '../types';

/**
 * Returns a reactive PanelVisibility that is loaded from localStorage on first
 * call and saved back whenever any panel is toggled. Unknown keys in storage are
 * ignored; keys present in defaults but absent in storage keep their default value.
 */
export function usePersistentPanels(key: string, defaults: PanelVisibility): PanelVisibility {
    let initial = { ...defaults };
    try {
        const stored = localStorage.getItem(key);
        if (stored) {
            initial = { ...defaults, ...JSON.parse(stored) };
        }
    } catch {
        // Malformed storage entry — fall back to defaults silently.
    }

    const panels = reactive<PanelVisibility>(initial);

    watch(panels, (value) => {
        localStorage.setItem(key, JSON.stringify(value));
    }, { deep: true });

    return panels;
}
