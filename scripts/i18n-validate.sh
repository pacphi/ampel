#!/bin/bash
# I18n Validation Utility
# Validates translation files for both backend and frontend
# Usage: ./scripts/i18n-validate.sh [--backend|--frontend|--all] [--fix]

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
COVERAGE_THRESHOLD=95
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Modes
VALIDATE_BACKEND=false
VALIDATE_FRONTEND=false
FIX_MODE=false

# Parse arguments
for arg in "$@"; do
  case $arg in
    --backend)
      VALIDATE_BACKEND=true
      shift
      ;;
    --frontend)
      VALIDATE_FRONTEND=true
      shift
      ;;
    --all)
      VALIDATE_BACKEND=true
      VALIDATE_FRONTEND=true
      shift
      ;;
    --fix)
      FIX_MODE=true
      shift
      ;;
    --help)
      echo "I18n Validation Utility"
      echo ""
      echo "Usage: $0 [options]"
      echo ""
      echo "Options:"
      echo "  --backend     Validate backend (Rust) translations"
      echo "  --frontend    Validate frontend (React) translations"
      echo "  --all         Validate both backend and frontend"
      echo "  --fix         Auto-fix issues where possible"
      echo "  --help        Show this help message"
      exit 0
      ;;
  esac
done

# Default to all if no specific mode selected
if [ "$VALIDATE_BACKEND" = false ] && [ "$VALIDATE_FRONTEND" = false ]; then
  VALIDATE_BACKEND=true
  VALIDATE_FRONTEND=true
fi

# Track overall status
EXIT_CODE=0

echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  I18n Validation Utility${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""

# Validate backend translations
if [ "$VALIDATE_BACKEND" = true ]; then
  echo -e "${YELLOW}🦀 Validating Backend (Rust) Translations${NC}"
  echo -e "${BLUE}───────────────────────────────────────────────────────────${NC}"

  cd "$PROJECT_ROOT"

  # Check if ampel-i18n-builder exists
  if ! cargo metadata --format-version 1 2>/dev/null | grep -q '"name":"ampel-i18n-builder"'; then
    echo -e "${YELLOW}⚠️  ampel-i18n-builder not found in workspace${NC}"
    echo -e "${YELLOW}   Skipping backend validation${NC}"
  else
    # Build the i18n builder
    echo "Building ampel-i18n-builder..."
    if cargo build --package ampel-i18n-builder --quiet 2>&1; then
      echo -e "${GREEN}✅ Build successful${NC}"
    else
      echo -e "${RED}❌ Build failed${NC}"
      EXIT_CODE=1
    fi

    # Check translation coverage
    echo ""
    echo "Checking translation coverage (threshold: ${COVERAGE_THRESHOLD}%)..."
    if cargo run --package ampel-i18n-builder --quiet -- check --min-coverage "$COVERAGE_THRESHOLD" 2>&1; then
      echo -e "${GREEN}✅ Coverage check passed${NC}"
    else
      echo -e "${RED}❌ Coverage check failed${NC}"
      EXIT_CODE=1
    fi

    # Check for missing translations
    echo ""
    echo "Checking for missing translations..."
    MISSING_OUTPUT=$(cargo run --package ampel-i18n-builder --quiet -- missing 2>&1)

    if echo "$MISSING_OUTPUT" | grep -q "No missing translations"; then
      echo -e "${GREEN}✅ No missing translations${NC}"
    else
      echo -e "${YELLOW}⚠️  Missing translations detected:${NC}"
      echo "$MISSING_OUTPUT"

      if [ "$FIX_MODE" = true ]; then
        echo ""
        echo "Running auto-fix..."
        cargo run --package ampel-i18n-builder --quiet -- translate --auto-approve
      fi
    fi

    # Validate YAML files
    echo ""
    echo "Validating YAML syntax..."
    YAML_FILES=$(find crates/ampel-api/locales -name "*.yml" -o -name "*.yaml" 2>/dev/null || true)

    if [ -z "$YAML_FILES" ]; then
      echo -e "${YELLOW}⚠️  No YAML files found${NC}"
    else
      YAML_ERRORS=0

      # Check if yamllint is installed
      if command -v yamllint &> /dev/null; then
        for file in $YAML_FILES; do
          if yamllint -c .yamllint.yml "$file" 2>&1 | grep -v "^$"; then
            YAML_ERRORS=$((YAML_ERRORS + 1))
          fi
        done

        if [ $YAML_ERRORS -eq 0 ]; then
          echo -e "${GREEN}✅ All YAML files valid${NC}"
        else
          echo -e "${RED}❌ $YAML_ERRORS YAML file(s) have errors${NC}"
          EXIT_CODE=1
        fi
      else
        echo -e "${YELLOW}⚠️  yamllint not installed, skipping YAML validation${NC}"
        echo "   Install with: pip install yamllint"
      fi
    fi
  fi

  echo ""
fi

# Validate frontend translations
if [ "$VALIDATE_FRONTEND" = true ]; then
  echo -e "${YELLOW}⚛️  Validating Frontend (React) Translations${NC}"
  echo -e "${BLUE}───────────────────────────────────────────────────────────${NC}"

  cd "$PROJECT_ROOT/frontend"

  # Check if dependencies are installed
  if [ ! -d "node_modules" ]; then
    echo -e "${YELLOW}⚠️  Frontend dependencies not installed${NC}"
    echo "   Run: cd frontend && pnpm install"
    EXIT_CODE=1
  else
    # Validate JSON files
    echo "Validating JSON syntax..."
    JSON_FILES=$(find public/locales -name "*.json" 2>/dev/null || true)

    if [ -z "$JSON_FILES" ]; then
      echo -e "${YELLOW}⚠️  No JSON files found${NC}"
    else
      JSON_ERRORS=0

      for file in $JSON_FILES; do
        if ! node -e "JSON.parse(require('fs').readFileSync('$file', 'utf8'))" 2>/dev/null; then
          echo -e "${RED}❌ Invalid JSON: $file${NC}"
          JSON_ERRORS=$((JSON_ERRORS + 1))

          if [ "$FIX_MODE" = true ]; then
            echo "   Attempting to auto-fix..."
            # Try to reformat with jq if available
            if command -v jq &> /dev/null; then
              jq '.' "$file" > "${file}.tmp" && mv "${file}.tmp" "$file"
              echo -e "${GREEN}   ✅ Fixed${NC}"
            fi
          fi
        fi
      done

      if [ $JSON_ERRORS -eq 0 ]; then
        echo -e "${GREEN}✅ All JSON files valid${NC}"
      else
        echo -e "${RED}❌ $JSON_ERRORS JSON file(s) have errors${NC}"
        EXIT_CODE=1
      fi
    fi

    # Run coverage check
    echo ""
    echo "Checking translation coverage (threshold: ${COVERAGE_THRESHOLD}%)..."

    if [ -f "scripts/i18n-coverage-report.js" ]; then
      if node scripts/i18n-coverage-report.js --check --min-coverage "$COVERAGE_THRESHOLD" 2>&1; then
        echo -e "${GREEN}✅ Coverage check passed${NC}"
      else
        echo -e "${RED}❌ Coverage check failed${NC}"
        EXIT_CODE=1

        # Show detailed report
        echo ""
        echo "Detailed coverage report:"
        node scripts/i18n-coverage-report.js --format text
      fi
    else
      echo -e "${YELLOW}⚠️  Coverage report script not found${NC}"
    fi

    # Check for missing keys
    echo ""
    echo "Checking for missing translation keys..."

    if [ -f "scripts/i18n-coverage-report.js" ]; then
      MISSING_OUTPUT=$(node scripts/i18n-coverage-report.js --check-missing 2>&1 || true)

      if echo "$MISSING_OUTPUT" | grep -q "Missing keys found"; then
        echo -e "${YELLOW}⚠️  Missing keys detected${NC}"
        echo "$MISSING_OUTPUT"
        EXIT_CODE=1
      else
        echo -e "${GREEN}✅ No missing keys${NC}"
      fi
    fi
  fi

  cd "$PROJECT_ROOT"
  echo ""
fi

# Final summary
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"

if [ $EXIT_CODE -eq 0 ]; then
  echo -e "${GREEN}✅ All validation checks passed${NC}"
else
  echo -e "${RED}❌ Validation failed${NC}"

  if [ "$FIX_MODE" = false ]; then
    echo ""
    echo -e "${YELLOW}💡 Tip: Run with --fix to auto-fix some issues${NC}"
  fi
fi

echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"

exit $EXIT_CODE
