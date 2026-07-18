export type PanelId =
    | 'page-list'
    | 'section-structure'

    | 'page-strips'
    | 'page-preview'

    | 'tools'
    | 'ocr-structure'
    | 'structural-tree'
    ;

export type PanelVisibility = Record<PanelId, boolean>;
