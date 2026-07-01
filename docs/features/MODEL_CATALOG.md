# Model Catalog Feature

## Overview

The Model Catalog is the curated, catalog-driven source of *selectable* models in
the Remediation **Model** tab. It gives the UI a single, air-gap-aware list of
providers and their models, plus thin proxies for local [Ollama](https://ollama.com)
discovery and remote model pulls. Key properties:

- One curated list of remediation-capable models grouped by provider
- Capability metadata (context window, tool use, code edit, egress, cost) per model
- Air-gap filtering: external providers are hidden for air-gapped organizations
- Live Ollama tag discovery and background model pulls against a local server
- No new environment variable — the catalog is embedded in the binary at build time

## Source of Truth

The editable source of truth is [`config/models.yaml`](../../config/models.yaml).
It is parsed by `crates/ampel-core/src/remediation/model_catalog.rs` and embedded
into the binary at build time via `include_str!` (`DEFAULT_CATALOG_YAML`). There is
**no** runtime file read and **no** new environment variable — editing the catalog
means editing this file and rebuilding.

### Adding a Model

Drop a new entry under the appropriate provider's `models:` list. **Every field is
optional** — entries are deserialized with serde defaults, so a partial (or
future-extended) entry never breaks the catalog, and unknown provider keys are
skipped (warned), not fatal.

```yaml
providers:
  ollama:
    description: "Local Ollama server (OpenAI-compatible) — stays on the host/network."
    egress: local_only
    models:
      - id: qwen2.5-coder
        name: "Qwen2.5 Coder 7B"
        family: qwen
        context_size: 32768
        tool_calling: true
        code_edit: true
        quality: good
        ollama_tag: "qwen2.5-coder:7b"
```

**Provider block fields:**

- `description` — human-readable provider summary
- `egress` — `external` or `local_only`; narrows the provider's egress class
- `models` — the list of catalog models for the provider

**Model fields (all optional / defaulted):**

- `id` — stable model identifier used as the catalog `modelId`
- `name` — display name
- `family` — model family (e.g. `claude`, `qwen`, `deepseek`)
- `context_size` — max context window in tokens
- `tool_calling` — whether the model supports native tool use
- `code_edit` — whether the model participates in the edit loop
- `quality` — `excellent` | `good` | `fair`
- `cost_per_1k_input` / `cost_per_1k_output` — quoted strings parsed as exact
  `Decimal` values (never `f64`, per ADR-008)
- `ollama_tag` — the Ollama pull tag (Ollama models only)
- `model_path` — on-disk model path (ONNX local providers only)

Capability fields (modality, egress, output contract, model kind, cost) are
**derived** from the provider kind when omitted — see `CatalogModel::caps` — and
may be narrowed per-provider via `egress:`.

## API Endpoints

All model-catalog endpoints are authenticated and located under
`/api/model-catalog`.

### Get Catalog

Return the embedded catalog grouped by provider, each model resolved to its
capabilities.

**Endpoint:** `GET /api/model-catalog`

**Authentication:** Required

**Query Parameters:**

- `organizationId` (optional) — when supplied, owned by the caller, and
  `air_gapped`, external-egress providers (Claude/Gemini) are omitted entirely
  (ADR-014). A missing or unowned org applies no filter.

**Response:**

```json
{
  "data": {
    "providers": [
      {
        "kind": "ollama",
        "description": "Local Ollama server (OpenAI-compatible) — stays on the host/network.",
        "egress": "local_only",
        "models": [
          {
            "id": "qwen2.5-coder",
            "name": "Qwen2.5 Coder 7B",
            "family": "qwen",
            "quality": "good",
            "ollamaTag": "qwen2.5-coder:7b",
            "contextWindow": 32768,
            "toolUse": true,
            "codeEdit": true,
            "egress": "local_only",
            "outputContract": "unified_diff",
            "cost": { "kind": "free" }
          }
        ]
      }
    ]
  }
}
```

### List Ollama Tags

Proxy a local Ollama server's `/api/tags` to discover models already pulled onto
the host.

**Endpoint:** `GET /api/model-catalog/ollama/tags`

**Authentication:** Required

**Query Parameters:**

- `accountId` (required) — an authorized Ollama model-provider account whose
  `endpointUrl` (default `http://localhost:11434`) is used as the target

The account endpoint is passed through the **same** SSRF guard as
`validate_model_account` before any request leaves the process: local-only egress
is exempt (reaching `localhost` is its purpose), while external egress is fully
SSRF-checked and rejected with `422` for internal/metadata targets. Upstream Ollama
error detail is never returned — failures are logged server-side and reduced to a
generic `502`.

### Pull Ollama Model

Start a background pull of a model onto the local Ollama server and return a job id
immediately.

**Endpoint:** `POST /api/model-catalog/ollama/pull`

**Authentication:** Required

**Request Body:**

```json
{
  "accountId": "…",
  "model": "qwen2.5-coder:7b"
}
```

**Response:**

```json
{
  "data": {
    "jobId": "…",
    "status": "queued"
  }
}
```

### Get Pull Status

Poll the status of a running or completed pull job. Jobs are owner-scoped; a job
owned by another user is treated as absent (`404`) so existence is never leaked.

**Endpoint:** `GET /api/model-catalog/ollama/pull/{id}/status`

**Authentication:** Required

**Response:**

```json
{
  "data": {
    "jobId": "…",
    "status": "downloading",
    "detail": "pulling manifest"
  }
}
```

Job status advances through `queued → downloading → ready`, or to `error` on
failure. The pull-job registry is in-memory (never persisted) and bounded: once it
exceeds `MAX_PULL_JOBS`, the oldest terminal (`ready`/`error`) jobs are evicted
first, deterministically by insertion sequence.

## Implementation Details

### Air-gap Filtering

`filter_catalog` performs a pure, DB-free projection of the catalog into the
response DTO. When the resolved organization is `air_gapped`, any provider or model
whose resolved egress is `external` (Claude/Gemini) is omitted entirely (ADR-014),
so an air-gapped organization only ever sees local-only providers.

### Shared SSRF Guard

The Ollama tags and pull endpoints reuse `handlers::security::assert_endpoint_safe`
after `load_authorized_account`, exactly like the validate path. Local-only
providers are intentionally exempt inside that guard; external egress is
SSRF-checked before any tags/pull request is issued.

### Embedded Catalog

The catalog is compiled into the binary via `include_str!("config/models.yaml")`,
so there is no runtime dependency on the file's presence and **no new environment
variable**. `load_catalog` falls back to the embedded default when an on-disk path
is missing or unreadable.

## Troubleshooting

### Ollama pull/discovery fails with "could not reach the Ollama server"

The tags and pull proxies run **inside the API container**. From there, `localhost`
resolves to the container itself — not your host — so an account whose `endpointUrl`
is `http://localhost:11434` cannot reach an Ollama server running on your machine.

Set the account's **Endpoint URL** to reach the host instead:

- **Docker (macOS / Windows / Linux):** `http://host.docker.internal:11434`
  (the compose file maps `host.docker.internal` to the host gateway, so this works
  on Linux too).
- **Ollama in its own container on the same compose network:** use the service name,
  e.g. `http://ollama:11434`.
- **API running natively (not in Docker):** `http://localhost:11434` is correct.

The pull job's `detail` now names the failure (e.g. the unreachable endpoint), and
the SSRF guard still exempts local-only providers, so these host targets are allowed.

## Related Files

- `config/models.yaml` — the editable catalog source of truth
- `crates/ampel-core/src/remediation/model_catalog.rs` — parse + capability derivation
- `crates/ampel-api/src/handlers/model_catalog.rs` — catalog + Ollama proxy handlers
- `crates/ampel-api/tests/test_model_catalog.rs` — integration tests
