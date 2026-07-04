export interface Author {
    full_name: string;
    abbrev: string | null;
}

export interface Project {
    name: string;
    machine_name: string;
    abbrev: string | null;
    description: string | null;
    authors: Author[];
    published: string | null;
}

export interface ProjectCreateForm {
    name: string;
    machine_name: string;
}
