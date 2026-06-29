# Ampel remediation sandbox image (ADR-003).
#
# An isolated, polyglot toolchain image in which the worker performs mechanical
# PR consolidation: clone, sequential `git merge --no-ff`, and per-ecosystem
# lockfile regeneration. It runs as a NON-root user (UID 1000), holds NO
# credentials at build time (the PAT is injected at run time via a tmpfs
# env-file), and is intended to run under a restricted egress allowlist
# (see README.md).
#
# Pin to a digest in production via AMPEL_SANDBOX_IMAGE.
FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive \
    LANG=C.UTF-8 \
    # Never prompt for git credentials inside the sandbox.
    GIT_TERMINAL_PROMPT=0 \
    GIT_ASKPASS=/bin/echo

# --- Base OS + git + language runtimes -------------------------------------
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates curl git gnupg \
        build-essential pkg-config \
        python3 python3-pip python3-venv \
        ruby-full \
        golang-go \
    && rm -rf /var/lib/apt/lists/*

# --- Node.js (LTS) + npm + pnpm + yarn -------------------------------------
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y --no-install-recommends nodejs \
    && npm install -g pnpm@9 yarn@1 \
    && rm -rf /var/lib/apt/lists/*

# --- Rust + cargo (rustup, stable) -----------------------------------------
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --no-modify-path --default-toolchain stable --profile minimal \
    && chmod -R a+rwX "$CARGO_HOME" "$RUSTUP_HOME"

# --- Python Poetry + Ruby Bundler ------------------------------------------
RUN pip3 install --no-cache-dir --break-system-packages poetry \
    && gem install --no-document bundler

# --- Non-root user (UID 1000) ----------------------------------------------
RUN useradd --create-home --uid 1000 --shell /bin/bash sandbox
USER 1000
WORKDIR /home/sandbox/work

# The worker mounts an entrypoint + the working tree at run time; this image
# only provides the toolchain. Default to a shell so the container is inert
# unless explicitly driven.
CMD ["/bin/bash"]
