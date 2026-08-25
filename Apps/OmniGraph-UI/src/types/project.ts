export interface Author {
    full_name: string;
    abbrev: string | null;
}

export interface ColorSpecification {
    base_color?: string;
    hue_shift?: number;
    lightness_shift?: number;
    saturation_shift?: number;
}

export interface FlowSchema {
    name: string;
    color?: ColorSpecification;
}

export interface LayoutSchema {
    name: string;
    color?: ColorSpecification;
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
