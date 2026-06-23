import { computed, type Ref } from 'vue';
import type {Page} from "../types";


// Parse a Roman numeral string to an integer, or return null.
function parseRoman(s: string): number | null {
    const upper = s.toUpperCase().trim();
    if (!upper || !/^[IVXLCDM]+$/.test(upper)) return null;
    const vals: Record<string, number> = { I:1, V:5, X:10, L:50, C:100, D:500, M:1000 };
    let total = 0, prev = 0;
    for (const ch of [...upper].reverse()) {
        const v = vals[ch];
        if (!v) return null;
        if (v < prev) total -= v; else total += v;
        prev = v;
    }
    return total > 0 ? total : null;
}

// Derive a 1-based ordinal from the page name where possible, so even/odd
// reflects what the user sees rather than the internal storage index.
// Priority: Arabic numeral name → Roman numeral name → 1-based index fallback.
function pageOrdinal(page: Page): number {
    if (page.name) {
        const arabic = parseInt(page.name.trim(), 10);
        if (!isNaN(arabic) && String(arabic) === page.name.trim()) return arabic;
        const roman = parseRoman(page.name.trim());
        if (roman !== null) return roman;
    }
    return page.index + 1; // 1-based: index 0 → ordinal 1 (odd)
}

export function makeIsInFilter(filterMode: Ref<string>) {
    return function isInFilter(page: Page): boolean {
        if (filterMode.value === 'all') return true;

        const ord = pageOrdinal(page);

        if (filterMode.value === 'even') return ord % 2 === 0;

        return ord % 2 !== 0;
    };
}

export function useFilteredPages(
    filterMode: Ref<string>,
    pages: Ref<Page[]>,
) {

    // isInFilter: true when a page passes the current even/odd filter.
    // Uses the page ordinal derived from its name, not the raw storage index.
    function isInFilter(page: Page): boolean {
        if (filterMode.value === 'all')  return true;
        const ord = pageOrdinal(page);
        if (filterMode.value === 'even') return ord % 2 === 0;
        return ord % 2 !== 0;
    }

    const filtered = computed(() => {
        return filterMode.value === 'all'
            ? pages.value
            : pages.value.filter(isInFilter)
    });

    return filtered;
}


