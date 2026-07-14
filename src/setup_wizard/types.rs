pub(crate) const ENV_FILE_NAME: &str = ".env";
pub(crate) const DEFAULT_OPENAI_BASE_URL: &str = "https://api.openai.com/v1";
pub(crate) const DEFAULT_ANTHROPIC_BASE_URL: &str = "https://api.anthropic.com";
pub(crate) const DEFAULT_POLLINATIONS_FREE_BASE_URL: &str = "https://text.pollinations.ai/openai";
pub(crate) const DEFAULT_POLLINATIONS_FREE_MODEL: &str = "openai-fast";
pub(crate) const DEFAULT_MODEL: &str = "gpt-4o-mini";
pub(crate) const DEFAULT_MAX_TOKENS: &str = "50000";
pub(crate) const DEFAULT_MAX_TOKENS_FAST: &str = "300";
pub(crate) const DEFAULT_TEMPERATURE: &str = "0.7";
pub(crate) const PLACEHOLDER_API_KEY: &str = "sk-your-key-here";

pub(crate) const LLM_ENV_KEYS: &[&str] = &[
    "LLM_PROVIDER",
    "LLM_BASE_URL",
    "LLM_API_KEY",
    "LLM_MODEL",
    "LLM_MODEL_FAST",
    "LLM_MODEL_MEDIUM",
    "LLM_MODEL_HEAVY",
    "LLM_MAX_TOKENS",
    "LLM_MAX_TOKENS_FAST",
    "LLM_MAX_TOKENS_MEDIUM",
    "LLM_MAX_TOKENS_HEAVY",
    "LLM_TEMPERATURE",
    "LLM_DISABLE_FAST_THINKING",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EnvAssignment {
    pub(crate) key: &'static str,
    pub(crate) value: String,
}

pub(crate) struct ApiPreset {
    pub(crate) name: &'static str,
    pub(crate) description: &'static str,
    pub(crate) provider: &'static str,
    pub(crate) base_url: &'static str,
    pub(crate) model: &'static str,
    pub(crate) requires_api_key: bool,
    pub(crate) disable_fast_thinking: &'static str,
}

pub(crate) struct ProviderSetup {
    pub(crate) provider: &'static str,
    pub(crate) base_url: String,
    pub(crate) api_key: String,
    pub(crate) fast: String,
    pub(crate) medium: String,
    pub(crate) heavy: String,
    pub(crate) disable_fast_thinking_default: Option<&'static str>,
}

pub(crate) const API_PRESETS: &[ApiPreset] = &[
    ApiPreset {
        name: "Pollinations Free",
        description: "No account or API key, public shared endpoint",
        provider: "pollinations-free",
        base_url: DEFAULT_POLLINATIONS_FREE_BASE_URL,
        model: DEFAULT_POLLINATIONS_FREE_MODEL,
        requires_api_key: false,
        disable_fast_thinking: "",
    },
    ApiPreset {
        name: "OpenAI",
        description: "BYOK, OpenAI API",
        provider: "openai-compatible",
        base_url: DEFAULT_OPENAI_BASE_URL,
        model: DEFAULT_MODEL,
        requires_api_key: true,
        disable_fast_thinking: "",
    },
    ApiPreset {
        name: "Anthropic Claude",
        description: "BYOK, native Claude Messages API",
        provider: "anthropic",
        base_url: DEFAULT_ANTHROPIC_BASE_URL,
        model: "claude-sonnet-4-6",
        requires_api_key: true,
        disable_fast_thinking: "",
    },
    ApiPreset {
        name: "Google Gemini",
        description: "BYOK or free-tier key, OpenAI-compatible Gemini endpoint",
        provider: "openai-compatible",
        base_url: "https://generativelanguage.googleapis.com/v1beta/openai",
        model: "gemini-3.5-flash",
        requires_api_key: true,
        disable_fast_thinking: "",
    },
    ApiPreset {
        name: "DeepSeek",
        description: "BYOK, OpenAI-compatible DeepSeek endpoint",
        provider: "openai-compatible",
        base_url: "https://api.deepseek.com",
        model: "deepseek-v4-flash",
        requires_api_key: true,
        disable_fast_thinking: "true",
    },
    ApiPreset {
        name: "Groq",
        description: "Free plan available, OpenAI-compatible Groq endpoint",
        provider: "openai-compatible",
        base_url: "https://api.groq.com/openai/v1",
        model: "llama-3.3-70b-versatile",
        requires_api_key: true,
        disable_fast_thinking: "",
    },
    ApiPreset {
        name: "OpenRouter Free Router",
        description: "Free routed models, OpenAI-compatible OpenRouter endpoint",
        provider: "openai-compatible",
        base_url: "https://openrouter.ai/api/v1",
        model: "openrouter/free",
        requires_api_key: true,
        disable_fast_thinking: "",
    },
    ApiPreset {
        name: "Mistral",
        description: "BYOK, OpenAI-style chat completions endpoint",
        provider: "openai-compatible",
        base_url: "https://api.mistral.ai/v1",
        model: "mistral-small-latest",
        requires_api_key: true,
        disable_fast_thinking: "",
    },
    ApiPreset {
        name: "Cerebras",
        description: "Free account limits available, OpenAI-compatible endpoint",
        provider: "openai-compatible",
        base_url: "https://api.cerebras.ai/v1",
        model: "gpt-oss-120b",
        requires_api_key: true,
        disable_fast_thinking: "",
    },
];
