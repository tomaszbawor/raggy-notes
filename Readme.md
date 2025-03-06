# Raggy notes

Goal of this project is to mimic that I can program in Rust
and I know what I am doing by trying using OpenAI models
to help me design the system and learn about RAG, RUST
and other fancy words that are trendy right now.

> [!WARNING]
> This is not even for personal use, this is just a POC

## Docker Hosting

In order to start ollama you need to run docker compose and then
force ollama to download model.

```bash
docker compose up -d
```

Needed model will be dowloaded automaticly by the script `docker/ollama-entrypoint.sh`
which will pull model provided in the `docker-compose.yaml` for the ollama service.

### Local Links for docker compose services

- [OpenUi](http://localhost:3000)
- [Qdrant UI](http://localhost:6333/dashboard)

## Nix development shell

The file `flake.nix` creates dev environment with all needed tools for building the project.
In order to get into the development env you may run.

```bash
nix develop
```

Then you should be able to build and test it.

```bash
cargo build 
cargo test 
```
