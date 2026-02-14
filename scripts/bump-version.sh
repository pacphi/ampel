#!/usr/bin/env bash
set -euo pipefail

# Bump the version consistently across all version-bearing files:
#   - Cargo.toml (workspace.package.version)
#   - crates/ampel-i18n-builder/Cargo.toml (package.version)
#   - frontend/package.json (version)
#
# Usage:
#   ./scripts/bump-version.sh <new-version>
#   ./scripts/bump-version.sh 0.4.0
#   ./scripts/bump-version.sh patch   # auto-bump patch (0.3.0 -> 0.3.1)
#   ./scripts/bump-version.sh minor   # auto-bump minor (0.3.0 -> 0.4.0)
#   ./scripts/bump-version.sh major   # auto-bump major (0.3.0 -> 1.0.0)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

CARGO_TOML="$ROOT_DIR/Cargo.toml"
I18N_CARGO_TOML="$ROOT_DIR/crates/ampel-i18n-builder/Cargo.toml"
PACKAGE_JSON="$ROOT_DIR/frontend/package.json"

if [[ $# -ne 1 ]]; then
    echo "Usage: $0 <new-version|patch|minor|major>"
    echo ""
    echo "Examples:"
    echo "  $0 0.4.0    # set explicit version"
    echo "  $0 patch    # bump patch: 0.3.0 -> 0.3.1"
    echo "  $0 minor    # bump minor: 0.3.0 -> 0.4.0"
    echo "  $0 major    # bump major: 0.3.0 -> 1.0.0"
    exit 1
fi

# Read current version from root Cargo.toml
current_version=$(grep -A5 '^\[workspace\.package\]' "$CARGO_TOML" | grep '^version' | head -1 | sed 's/.*"\(.*\)".*/\1/')

if [[ -z "$current_version" ]]; then
    echo "Error: could not read current version from $CARGO_TOML"
    exit 1
fi

arg="$1"

# Handle symbolic bump types
case "$arg" in
    patch|minor|major)
        IFS='.' read -r major minor patch <<< "$current_version"
        case "$arg" in
            patch) patch=$((patch + 1)) ;;
            minor) minor=$((minor + 1)); patch=0 ;;
            major) major=$((major + 1)); minor=0; patch=0 ;;
        esac
        new_version="${major}.${minor}.${patch}"
        ;;
    *)
        # Validate explicit version format
        if ! echo "$arg" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$'; then
            echo "Error: invalid version format '$arg'. Expected semver like 0.4.0 or 1.0.0-beta.1"
            exit 1
        fi
        new_version="$arg"
        ;;
esac

if [[ "$current_version" == "$new_version" ]]; then
    echo "Version is already $current_version — nothing to do."
    exit 0
fi

echo "Bumping version: $current_version -> $new_version"
echo ""

# 1. Root Cargo.toml — workspace.package.version
echo "  Updating $CARGO_TOML"
sed -i '' "s/^version = \"$current_version\"/version = \"$new_version\"/" "$CARGO_TOML"

# 2. crates/ampel-i18n-builder/Cargo.toml — package.version
echo "  Updating $I18N_CARGO_TOML"
sed -i '' "s/^version = \"$current_version\"/version = \"$new_version\"/" "$I18N_CARGO_TOML"

# 3. frontend/package.json — version field
echo "  Updating $PACKAGE_JSON"
sed -i '' "s/\"version\": \"$current_version\"/\"version\": \"$new_version\"/" "$PACKAGE_JSON"

echo ""
echo "Done. Files updated to $new_version:"
echo "  - Cargo.toml (workspace)"
echo "  - crates/ampel-i18n-builder/Cargo.toml"
echo "  - frontend/package.json"
echo ""
echo "Next steps:"
echo "  git add Cargo.toml crates/ampel-i18n-builder/Cargo.toml frontend/package.json"
echo "  git commit -m \"chore: bump version to $new_version\""
