# Slay the Spire AI Copilot

A local CLI copilot for Slay the Spire that reads game state via CommunicationMod, calls an LLM for advice, and writes suggestions to a local file.

## Requirements

- Rust 1.85+ (edition 2024)
- [Slay the Spire](https://store.steampowered.com/app/646570/Slay_the_Spire/) with [ModTheSpire](https://github.com/kiooeht/ModTheSpire) and [CommunicationMod](https://github.com/ForgottenArbiter/CommunicationMod)

## Quick Start

```bash
# Build
cargo build --release

# Copy .env.example and configure (optional, mock provider works without)
cp .env.example .env

# Test with the mock provider
echo '{"in_game":true,"game_state":{"screen_type":"NONE","class":"IRONCLAD","floor":1,"current_hp":68,"max_hp":75,"gold":99}}' | cargo run --release
```

Check `output/advice.txt` for the AI suggestion.

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `LLM_PROVIDER` | `mock` | `mock` or `openai-compatible` |
| `LLM_BASE_URL` | — | API base URL (required for `openai-compatible`) |
| `LLM_API_KEY` | — | API key (required for `openai-compatible`) |
| `LLM_MODEL` | `gpt-4o-mini` | Model name |

### Mock Provider

Returns a canned Chinese response. No API key or network needed.

### OpenAI-Compatible Provider

Works with OpenAI, Ollama, vLLM, or any `/chat/completions` endpoint.

```env
LLM_PROVIDER=openai-compatible
LLM_BASE_URL=https://api.openai.com/v1
LLM_API_KEY=sk-your-key
LLM_MODEL=gpt-4o-mini
```

## CommunicationMod Setup

1. Install [ModTheSpire](https://github.com/kiooeht/ModTheSpire) and [CommunicationMod](https://github.com/ForgottenArbiter/CommunicationMod)
2. Place `CommunicationMod.jar` in your ModTheSpire mods directory
3. Launch ModTheSpire with CommunicationMod enabled
4. Edit your SpireConfig file to point to the release binary:

```
# In your SpireConfig file:
command=/path/to/slay-the-spire-copilot/target/release/slay-the-spire-copilot
```

5. Start a Slay the Spire run. The copilot will receive game state, generate advice, and write it to `output/advice.txt`.

## Protocol

The binary communicates with CommunicationMod via stdin/stdout:

- **stdout** → `ready\n` (on startup) → `WAIT 30\n` (after each state)
- **stdin** → line-delimited JSON game state from CommunicationMod

All logs go to `logs/sts-ai.log`. stdout is reserved exclusively for CommunicationMod protocol commands.

## Project Structure

```
src/
  main.rs       — main loop, stdin/stdout orchestration
  config.rs     — environment variable loading
  protocol.rs   — protocol messages (ready, WAIT 30)
  state.rs      — normalized game state + stable hashing
  prompt.rs     — Chinese LLM prompt builder
  llm.rs        — LLM provider abstraction (mock + openai-compatible)
  advice.rs     — advice cache + file output
  logging.rs    — tracing file logger
tests/
  fixtures/      — sample CommunicationMod JSON states
```

## Output Format

AI advice is written to `output/advice.txt` in Chinese format:

```
推荐：（recommended action）
理由：（reasoning）
风险：（risks）
吐槽：（commentary）
```

## License

MIT
