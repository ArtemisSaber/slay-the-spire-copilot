use super::{GameRuntime, state::ProcessedState};
use crate::llm::Effort;

/// Truncation length for prompt previews in debug logs.
const PROMPT_LOG_PREVIEW_CHARS: usize = 200;

impl GameRuntime {
    pub(super) async fn generate_advice(&mut self, state: &ProcessedState<'_>) {
        if state.screen_type == "SHOP_ROOM" || state.screen_type == "SHOP_SCREEN" {
            self.map_gate.on_shop();
        }
        if state.screen_type == "MAP" && state.normalized.map_first_node_chosen == Some(false) {
            self.map_gate.on_act_entry();
        }

        let map_should_generate = state.screen_type == "MAP"
            && self
                .map_gate
                .should_generate(state.raw, &state.normalized.map_nodes);
        if !crate::gate::should_generate_advice(state.screen_type, state.raw)
            && !self.combat_turn_gate.is_player_turn_start(state.raw)
            && !map_should_generate
        {
            tracing::debug!("skipping screen type: {}", state.screen_type);
            return;
        }

        tracing::info!(
            "state screen={} room={} floor={} hp={}/{} block={} energy={} hand={} mons={} deck={} incoming={} danger={:?}",
            state.screen_type,
            state.room_type,
            state
                .normalized
                .floor
                .map_or("?".to_string(), |value| value.to_string()),
            state
                .normalized
                .current_hp
                .map_or("?".to_string(), |value| value.to_string()),
            state
                .normalized
                .max_hp
                .map_or("?".to_string(), |value| value.to_string()),
            state
                .normalized
                .block
                .map_or("?".to_string(), |value| value.to_string()),
            state
                .normalized
                .energy
                .map_or("?".to_string(), |value| value.to_string()),
            state.normalized.hand.len(),
            state.normalized.monsters.len(),
            state.normalized.deck_names.len(),
            state.normalized.incoming_damage,
            state.normalized.danger.level,
        );

        let effort = Effort::from_screen_type(state.screen_type, state.normalized.is_in_combat());
        let prompt =
            crate::prompt::build_prompt(state.normalized, &self.locale, self.map_gate.shop_visited);
        tracing::debug!(
            "prompt ({} chars): {}",
            prompt.len(),
            &prompt[..prompt.len().min(PROMPT_LOG_PREVIEW_CHARS)],
        );

        self.cache.write_overlay_loading(state.metadata);
        let advice = self
            .cache
            .get_or_compute(
                state.hash,
                &prompt,
                effort,
                state.scenario,
                &self.provider,
                &self.locale,
            )
            .await;

        let fields = crate::advice::parse_advice_response(&advice, &self.locale);
        let status = if advice.contains(&self.locale.fallback.llm_error) {
            "error"
        } else {
            "ok"
        };
        self.cache
            .write_overlay_ready(status, &fields, state.metadata);

        self.journal
            .log_advice(state.hash, effort, state.scenario, &prompt, &advice);
        tracing::info!(
            "wrote advice ({} chars hash={})",
            advice.len(),
            &state.hash[..16]
        );
        self.cache.write_advice(&advice);

        if state.screen_type == "MAP"
            && let (Some(current_x), Some(current_y)) = (
                state.normalized.map_current_x,
                state.normalized.map_current_y,
            )
        {
            self.map_gate
                .record(&state.normalized.map_nodes, current_x, current_y);
        }
    }
}
