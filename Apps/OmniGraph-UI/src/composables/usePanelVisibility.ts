import type { InjectionKey, Ref } from 'vue';
import { inject, provide, ref } from 'vue';
import type { PanelVisibility } from '../types';

type PanelVisibilityContext = {
    activePanels: Ref<PanelVisibility | null>;
    setActivePanels: (panels: PanelVisibility | null) => void;
};

const panelVisibilityKey: InjectionKey<PanelVisibilityContext> = Symbol('panelVisibility');

export function providePanelVisibilityContext() {
    const activePanels = ref<PanelVisibility | null>(null);

    function setActivePanels(panels: PanelVisibility | null) {
        activePanels.value = panels;
    }

    const context: PanelVisibilityContext = {
        activePanels,
        setActivePanels,
    };

    provide(panelVisibilityKey, context);

    return context;
}

export function usePanelVisibilityContext() {
    const context = inject(panelVisibilityKey);
    if (!context) {
        throw new Error('Panel visibility context was not provided');
    }
    return context;
}