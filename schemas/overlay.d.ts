// Auto-generated from src/advice.rs OverlayOutput. Keep in sync with the copilot.

export interface AdviceFields {
  recommendation: string;
  reason: string;
  risk: string;
  commentary: string;
}

export interface OverlayOutput {
  schema_version: number;
  status: "loading" | "ok" | "error";
  overlay_visibility: boolean;
  advice: AdviceFields;
  autoplay?: {
    mode: "off" | "auto";
    status: "idle" | "planning" | "executing" | "error";
  };
  screen_type: string | null;
  scenario: string;
  in_combat: boolean;
  state_hash: string;
  floor: number | null;
  character: string | null;
  timestamp_ms: number;
}
