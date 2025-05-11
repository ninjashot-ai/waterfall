#!/usr/bin/env -S just --justfile
# ^ A shebang isn't required, but allows a justfile to be executed
#   like a script, with `./justfile test`, for example.

set shell := ["zsh", "-cu"]
set dotenv-filename := ".env"
set dotenv-load := true

export OPENAI_API_KEY := env("OPENAI_API_KEY")
export OPENAI_BASE_URL := env("OPENAI_BASE_URL")

@demo:
    cargo run --package demo --bin demo
