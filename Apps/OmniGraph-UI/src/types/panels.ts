export type PanelId =
    | 'page-list'
    | 'page-strips'
    | 'page-preview'
    | 'section-structure'
    | 'ocr-structure'
    | 'tools'
    | 'structural-tree'
    ;

export type PanelVisibility = Record<PanelId, boolean>;
