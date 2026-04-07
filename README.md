Built Your Own AI Coding Assistant



A command-line AI coding assistant built in Rust.

It connects to an LLM via OpenRouter, sends user prompts, and autonomously
executes tools (read files, write files, run shell commands) in an agent loop
until the task is complete.

## How It Works

```
User prompt
    │
    ▼
┌──────────────────────────────────┐
│           Agent Loop             │
│                                  │
│  1. Send messages to LLM         │
│  2. LLM responds with either:    │
│     • Tool calls → execute them, │
│       append results, loop back  │
│     • Text → print and exit      │
└──────────────────────────────────┘
```

The program maintains a conversation history (`messages` array) that accumulates
user prompts, assistant responses, and tool results. Each loop iteration sends
the full history to the LLM so it retains context across multiple steps.

## Tools

The assistant exposes three tools to the LLM via the OpenAI-compatible
function-calling API:

| Tool | Description | Parameters |
|------|-------------|------------|
| **Read** | Read and return the contents of a file | `file_path` (string) |
| **Write** | Write content to a file (creates or overwrites) | `file_path` (string), `content` (string) |
| **Bash** | Execute a shell command and return stdout + stderr | `command` (string) |

When the LLM includes `tool_calls` in its response, the program executes each
one, appends the result as a `"role": "tool"` message, and loops back to let
the LLM decide what to do next.

## Prerequisites

- **Rust 1.94+** — install via [rustup](https://rustup.rs/)
- **OpenRouter API key** — sign up at [openrouter.ai](https://openrouter.ai) and create a key

## Setup

1. Clone the repository:

   ```bash
   git clone <your-repo-url>
   cd codecrafters-claude-code-rust
   ```

2. Set your API key:

   ```bash
   export OPENROUTER_API_KEY="your-key-here"
   ```

   Optionally override the base URL (defaults to `https://openrouter.ai/api/v1`):

   ```bash
   export OPENROUTER_BASE_URL="https://your-custom-endpoint.com/v1"
   ```

## Usage

Run the assistant with the `-p` (or `--prompt`) flag:

```bash
./your_program.sh -p "Your prompt here"
```

The first run compiles the project (may take a minute). Subsequent runs are fast.

### Examples

```bash
# Ask a question (no tools needed)
./your_program.sh -p "Explain what a borrow checker is"

# Read a file
./your_program.sh -p "Read README.md and summarize it"

# Write a file
./your_program.sh -p "Create a file called hello.txt containing 'Hello, World!'"

# Run a shell command
./your_program.sh -p "List all Rust source files in this project"

# Multi-step task (agent loop with multiple tool calls)
./your_program.sh -p "Read Cargo.toml, then create a summary.txt listing all dependencies"
```

## Project Structure

```
codecrafters-claude-code-rust/
├── src/
│   └── main.rs              # Application entry point and agent loop
├── Cargo.toml                # Dependencies and project metadata
├── Cargo.lock                # Locked dependency versions
├── your_program.sh           # Local build-and-run script
├── codecrafters.yml          # CodeCrafters config (Rust version, debug flag)
├── .codecrafters/
│   ├── compile.sh            # Remote compilation script
│   └── run.sh                # Remote execution script
└── README.md
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| [tokio](https://crates.io/crates/tokio) | Async runtime for non-blocking I/O |
| [async-openai](https://crates.io/crates/async-openai) | OpenAI-compatible API client (with `byot` for raw JSON requests) |
| [clap](https://crates.io/crates/clap) | CLI argument parsing via derive macros |
| [serde_json](https://crates.io/crates/serde_json) | JSON serialization and deserialization |

## Architecture

### CLI Layer (`clap`)

The program accepts a single required argument `--prompt` / `-p` parsed into an
`Args` struct using clap's derive API.

### API Client (`async-openai`)

An `OpenAIConfig` is configured with the OpenRouter base URL and API key, then
used to create a `Client`. The `create_byot` method sends raw JSON payloads,
allowing full control over the request body including tool definitions.

### Agent Loop

The core loop in `main()`:

1. Sends the full `messages` array to the LLM
2. Appends the assistant's response to `messages`
3. Checks for `tool_calls` in the response:
   - **If present**: executes each tool, appends results as `"role": "tool"` messages, continues the loop
   - **If absent**: prints the assistant's text content and breaks

### Tool Execution

Tools are dispatched by name. Each tool:
- Parses its arguments from the JSON provided by the LLM
- Performs the operation (file I/O or shell execution)
- Returns a result string that gets sent back to the LLM

## Submitting to CodeCrafters

```bash
codecrafters submit
```

Test output will stream to your terminal.

## Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `OPENROUTER_API_KEY` | Yes | — | Your OpenRouter API key |
| `OPENROUTER_BASE_URL` | No | `https://openrouter.ai/api/v1` | API base URL |
