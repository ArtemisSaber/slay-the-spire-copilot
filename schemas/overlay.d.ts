// Auto-generated from src/advice.rs OverlayOutput. Keep in sync with the copilot.

export interface AdviceFields {
  recommendation: string;
  reason: string;
  risk: string;
  commentary: string;
}

export interface LearningRunStatus {
  accepted: boolean;
  run_kind: "legitimate" | "debug_flow" | "synthetic" | "restored" | "incomplete" | "unknown";
  reasons: LearningIneligibilityReason[];
}

export type LearningIneligibilityReason =
  | "debug_ascension"
  | "invalid_ascension"
  | "missing_terminal_outcome"
  | "missing_character"
  | "missing_seed"
  | "missing_objective"
  | "missing_app_version"
  | "missing_model_profile"
  | "missing_prompt_schema"
  | "missing_locale"
  | "synthetic_input"
  | "stdin_test"
  | "state_restore"
  | "undo_used"
  | "inconsistent_telemetry"
  | "already_committed";

export interface LearningStatus {
  mode: "off" | "collect" | "shadow" | "on";
  case_count: number;
  lesson_count: number;
  last_run: LearningRunStatus | null;
}

export interface OverlayOutput {
  schema_version: number;
  status: "loading" | "ok" | "error";
  overlay_visibility: boolean;
  advice: AdviceFields;
  autoplay?: {
    mode: "off" | "auto" | "paused";
    status: "startup" | "idle" | "planning" | "executing" | "error" | "stopped";
  };
  learning?: LearningStatus;
  screen_type: string | null;
  scenario: string;
  in_combat: boolean;
  state_hash: string;
  floor: number | null;
  character: string | null;
  timestamp_ms: number;
}
