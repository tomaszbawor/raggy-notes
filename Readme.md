# Raggy Notes

A Rust-based TUI application for semantic search and note management using RAG (Retrieval-Augmented Generation).

## Features

- 📝 Manage and search your markdown notes with semantic search
- 🧠 Interact with AI models to ask questions about your notes
- 🔍 Find relevant information using natural language queries
- 🖥️ Terminal-based user interface for fast, keyboard-driven workflows

## Architecture

Raggy Notes uses:

- **Ollama**: Local LLM for text generation and embeddings
- **Qdrant**: Vector database for storing and searching embeddings
- **Ratatui**: Terminal UI framework for the interface
- **Tokio**: Async runtime for efficient concurrent operations

## Getting Started

### Prerequisites

- Rust toolchain (1.74+)
- Docker and Docker Compose (for Ollama and Qdrant)

### Quick Start

1. **Start the infrastructure services**:

```bash
docker compose up -d
```

2. **Build the application**:

```bash
cargo build --release
```

3. **Initialize the configuration**:

```bash
./target/release/raggy-notes init --scan-path /path/to/your/markdown/notes
```

4. **Index your notes**:

```bash
./target/release/raggy-notes index
```

5. **Start the application**:

```bash
./target/release/raggy-notes
```

## Usage

### Navigation

- `Tab`: Switch between tabs (Chat, Search, Settings)
- `Ctrl+Q` or `Ctrl+C`: Quit
- `Enter`: Send message/execute search
- `Up/Down`: Navigate search results

### Tabs

- **Chat**: Interact with the AI model
- **Search**: Search your notes semantically
- **Settings**: Configure application settings

## Docker Support

You can run the entire application stack with Docker Compose:

```bash
docker compose up -d
docker exec -it raggy-notes /app/raggy-notes init --scan-path /app/notes
docker exec -it raggy-notes /app/raggy-notes index
docker exec -it raggy-notes /app/raggy-notes
```

## Development

### Environment Setup

The project includes a Nix flake for development:

```bash
nix develop
```

### Project Structure

- `src/`
  - `config/`: Application configuration
  - `llama/`: Ollama client integration
  - `rag/`: RAG implementation (files, vector DB)
  - `tui/`: Terminal UI components
  - `error.rs`: Error handling
  - `main.rs`: Application entry point

## License

This project is licensed under the MIT License - see the LICENSE file for details.
