export interface SettingsForm {
    openai_api_key: string;
    perplexity_api_key: string;
}

export interface SettingsStatus {
    openai_api_key_set: boolean;
    perplexity_api_key_set: boolean;
}
