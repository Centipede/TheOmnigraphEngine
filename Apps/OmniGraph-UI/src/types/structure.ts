export interface SectionHeadline {
    page: string;
    block_id: string;
}

export type SubsectionType = 'sections' | 'flows';

export type SectionKind =
    | 'part'
    | 'chapter'
    | 'section'
    | 'subsection'
    | 'subsubsection'
    | 'subsubsubsection'
    | 'subsubsubsubsection';

export interface Section {
    path_id: string;
    kind: SectionKind;
    title: string;
    is_linked: boolean;
    is_orphaned: boolean;
    is_suggested: boolean;
    headline: SectionHeadline | null;
    subsection_type: SubsectionType;
    subsections: Section[];
}

export interface Headline {
    page: string;
    block_id: string;
    is_linked: boolean;
}

export interface StructureDb {
    sections: Section[];
    headlines: Headline[];
}
