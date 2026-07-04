export interface OcrServer {
    host: string;
    port: number;
}

export type OcrCommandFormat = 'native' | 'docker';
export type OcrServerStatus = 'unconfigured' | 'online' | 'offline';

export interface OcrSettingsUpdate {
    server_1: OcrServer | null;
    server_2: OcrServer | null;
    command_format: OcrCommandFormat;
}

export interface SettingsForm {
    openai_api_key: string;
    perplexity_api_key: string;
    ocr: OcrSettingsUpdate;
}

export interface SettingsStatus {
    openai_api_key_set: boolean;
    perplexity_api_key_set: boolean;
    ocr_server_1: OcrServer | null;
    ocr_server_2: OcrServer | null;
    ocr_command_format: OcrCommandFormat;
    ocr_server_1_status: OcrServerStatus;
    ocr_server_2_status: OcrServerStatus;
}
