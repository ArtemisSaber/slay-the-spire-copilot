use super::LlmProvider;

impl std::fmt::Debug for LlmProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mock => formatter.debug_struct("Mock").finish(),
            #[cfg(test)]
            Self::Scripted(_) => formatter.debug_struct("Scripted").finish_non_exhaustive(),
            Self::OpenAiCompatible {
                base_url,
                api_key: _,
                temperature,
                client,
                fast,
                medium,
                heavy,
            } => formatter
                .debug_struct("OpenAiCompatible")
                .field("base_url", base_url)
                .field("api_key", &"<REDACTED>")
                .field("temperature", temperature)
                .field("client", client)
                .field("fast", fast)
                .field("medium", medium)
                .field("heavy", heavy)
                .finish(),
            Self::PollinationsFree {
                base_url,
                temperature,
                client,
                fast,
                medium,
                heavy,
            } => formatter
                .debug_struct("PollinationsFree")
                .field("base_url", base_url)
                .field("temperature", temperature)
                .field("client", client)
                .field("fast", fast)
                .field("medium", medium)
                .field("heavy", heavy)
                .finish(),
            Self::Anthropic {
                base_url,
                api_key: _,
                client,
                fast,
                medium,
                heavy,
            } => formatter
                .debug_struct("Anthropic")
                .field("base_url", base_url)
                .field("api_key", &"<REDACTED>")
                .field("client", client)
                .field("fast", fast)
                .field("medium", medium)
                .field("heavy", heavy)
                .finish(),
        }
    }
}
