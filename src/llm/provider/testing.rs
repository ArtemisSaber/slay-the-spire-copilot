use std::sync::{Arc, Mutex};

use crate::llm::Effort;

use super::LlmProvider;

type ResponseHandler = dyn Fn(&str, &str, Effort) -> anyhow::Result<String> + Send + Sync + 'static;

#[derive(Clone)]
pub(crate) struct RecordedRequest {
    pub(crate) system_prompt: String,
    pub(crate) prompt: String,
    pub(crate) effort: Effort,
}

pub struct ScriptedProvider {
    requests: Arc<Mutex<Vec<RecordedRequest>>>,
    handler: Arc<ResponseHandler>,
}

impl ScriptedProvider {
    fn new<F>(handler: F) -> Self
    where
        F: Fn(&str, &str, Effort) -> anyhow::Result<String> + Send + Sync + 'static,
    {
        Self {
            requests: Arc::new(Mutex::new(vec![])),
            handler: Arc::new(handler),
        }
    }

    pub(super) fn query(
        &self,
        system_prompt: &str,
        prompt: &str,
        effort: Effort,
    ) -> anyhow::Result<String> {
        self.requests
            .lock()
            .expect("scripted provider request lock should not be poisoned")
            .push(RecordedRequest {
                system_prompt: system_prompt.to_string(),
                prompt: prompt.to_string(),
                effort,
            });
        (self.handler)(system_prompt, prompt, effort)
    }

    fn requests(&self) -> Vec<RecordedRequest> {
        self.requests
            .lock()
            .expect("scripted provider request lock should not be poisoned")
            .clone()
    }
}

impl LlmProvider {
    pub(crate) fn scripted<F>(handler: F) -> Self
    where
        F: Fn(&str, &str, Effort) -> anyhow::Result<String> + Send + Sync + 'static,
    {
        Self::Scripted(ScriptedProvider::new(handler))
    }

    pub(crate) fn recorded_requests(&self) -> Vec<RecordedRequest> {
        match self {
            Self::Scripted(provider) => provider.requests(),
            _ => vec![],
        }
    }
}
