export interface Author {
    full_name: string;
    abbrev: string | null;
}

export interface FlowSchema {
    name: string;
}

export interface LayoutSchema {
    name: string;
}

export interface Project {
    name: string;
    machine_name: string;
    abbrev: string | null;
    description: string | null;
    authors: Author[];
    published: string | null;
    ocr_language: string | null;
    flows: FlowSchema[];
    layouts: LayoutSchema[];
}

export interface ProjectCreateForm {
    name: string;
    machine_name: string;
}
