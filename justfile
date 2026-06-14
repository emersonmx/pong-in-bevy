set quiet

setup:
    #!/usr/bin/env bash
    set -euo pipefail

    if ! command -v bevy &> /dev/null; then
        echo "bevy not found, installing..."
        cli_version="cli-v0.1.0-alpha.2"
        cargo install \
            --git https://github.com/TheBevyFlock/bevy_cli \
            --tag "$cli_version" \
            --locked \
            bevy_cli
    fi

    if ! command -v bevy_lint &> /dev/null; then
        echo "bevy_lint not found, installing..."
        toolchain="nightly-2026-01-22"
        lint_version="lint-v0.6.0"
        rustup toolchain install "$toolchain" \
            --component rustc-dev \
            --component llvm-tools
        rustup run "$toolchain" cargo install \
            --git https://github.com/TheBevyFlock/bevy_cli.git \
            --tag "$lint_version" \
            --locked \
            bevy_lint
    fi

    if ! command -v lefthook &> /dev/null; then
        echo "lefthook not found, installing..."
        go install github.com/evilmartians/lefthook/v2@latest
    fi
    lefthook install

    if ! command -v bacon &> /dev/null; then
        echo "bacon not found, installing..."
        cargo install --locked bacon
    fi

build *ARGS:
    bevy build {{ ARGS }}

run *ARGS:
    bevy run {{ ARGS }}

watch *ARGS:
    bacon {{ ARGS }}

format:
    cargo fmt

lint:
    cargo clippy --locked --workspace --all-targets --all-features
    bevy_lint --locked --workspace --all-targets --all-features

lint-fix:
    cargo clippy --fix --allow-dirty --locked --workspace --all-targets --all-features
    bevy_lint --fix --allow-dirty --locked --workspace --all-targets --all-features

ci:
    cargo fmt --check
    cargo clippy --locked --workspace --all-targets --profile ci --all-features
    bevy_lint --locked --workspace --all-targets --profile ci --all-features

test *ARGS:
    cargo test {{ ARGS }}

clean *ARGS:
    cargo clean {{ ARGS }}
