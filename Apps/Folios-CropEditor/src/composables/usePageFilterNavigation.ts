import { nextTick, type Ref } from 'vue';
import type { Page } from '../types';

export function findClosestVisiblePageIndex(
    anchorIndex: number,
    pages: Page[],
    visiblePages: Page[],
): number | null {
    if (!visiblePages.length) return null;

    if (visiblePages.some(page => page.index === anchorIndex)) return anchorIndex;

    const anchorPos = pages.findIndex(page => page.index === anchorIndex);
    if (anchorPos < 0) return visiblePages[0].index;

    let bestIndex = visiblePages[0].index;
    let bestDistance = Number.POSITIVE_INFINITY;

    for (const page of visiblePages) {
        const pos = pages.findIndex(candidate => candidate.index === page.index);
        if (pos < 0) continue;

        const distance = Math.abs(pos - anchorPos);
        if (distance < bestDistance) {
            bestIndex = page.index;
            bestDistance = distance;
        }
    }

    return bestIndex;
}

export function usePageFilterNavigation(options: {
    filterMode: Ref<string>;
    pages: Ref<Page[]>;
    visiblePages: Ref<Page[]>;
    selectionAnchor: Ref<number | null>;
    currentPageIndex: Ref<number | null>;
    setAnchor: (pageIndex: number) => void;
}) {
    async function onFilterChange(event: Event) {
        const group = event.currentTarget as HTMLInputElement;
        const newFilterMode = group.value;

        const anchor = options.selectionAnchor.value ?? options.currentPageIndex.value;
        options.filterMode.value = newFilterMode;
        await nextTick();

        const visible = options.visiblePages.value;
        if (!visible.length) {
            options.selectionAnchor.value = null;
            options.currentPageIndex.value = null;
            return;
        }

        const nextAnchor = anchor === null
            ? visible[0].index
            : findClosestVisiblePageIndex(anchor, options.pages.value, visible);

        if (nextAnchor !== null && nextAnchor !== options.selectionAnchor.value) {
            options.setAnchor(nextAnchor);
        }
    }

    return {
        onFilterChange,
    };
}