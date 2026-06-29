#!/usr/bin/env bash
# Build + tag the Ampel remediation sandbox image (ADR-003).
#
# Usage:
#   ./build.sh [TAG]
#
# Env:
#   AMPEL_SANDBOX_IMAGE   Image name (default: ghcr.io/ampel/remediation-sandbox)
#   AMPEL_SANDBOX_RUNTIME Container runtime to build with (podman|docker; auto-detect otherwise)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
IMAGE="${AMPEL_SANDBOX_IMAGE:-ghcr.io/ampel/remediation-sandbox}"
TAG="${1:-latest}"

# Resolve runtime: explicit env wins, else prefer podman, else docker.
if [[ -n "${AMPEL_SANDBOX_RUNTIME:-}" ]]; then
  RUNTIME="${AMPEL_SANDBOX_RUNTIME}"
elif command -v podman >/dev/null 2>&1; then
  RUNTIME="podman"
elif command -v docker >/dev/null 2>&1; then
  RUNTIME="docker"
else
  echo "error: no container runtime found (need podman or docker)" >&2
  exit 1
fi

echo "Building ${IMAGE}:${TAG} with ${RUNTIME}..."
"${RUNTIME}" build -t "${IMAGE}:${TAG}" "${SCRIPT_DIR}"

echo "Built ${IMAGE}:${TAG}"
echo "Pin a digest for production use:"
echo "  ${RUNTIME} inspect --format '{{index .RepoDigests 0}}' ${IMAGE}:${TAG}"
