#!/bin/bash

# RTL Visual Testing Runner Script
# Runs comprehensive RTL validation for Ampel Phase 2 i18n

set -e

echo "🚀 Starting RTL Visual Testing Suite"
echo "===================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if Playwright is installed
echo -e "${BLUE}Checking Playwright installation...${NC}"
if ! npx playwright --version &> /dev/null; then
    echo -e "${YELLOW}Playwright not installed. Installing...${NC}"
    pnpm add -D @playwright/test
    npx playwright install
else
    echo -e "${GREEN}✓ Playwright installed${NC}"
fi

# Check if dev server is running
echo ""
echo -e "${BLUE}Checking dev server...${NC}"
if ! curl -s http://localhost:5173 > /dev/null; then
    echo -e "${YELLOW}⚠ Dev server not running on port 5173${NC}"
    echo "Please start the dev server first:"
    echo "  pnpm run dev"
    echo ""
    read -p "Start dev server automatically? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${BLUE}Starting dev server...${NC}"
        pnpm run dev &
        SERVER_PID=$!
        sleep 5
        echo -e "${GREEN}✓ Dev server started (PID: $SERVER_PID)${NC}"
    else
        echo -e "${RED}✗ Cannot run visual tests without dev server${NC}"
        exit 1
    fi
else
    echo -e "${GREEN}✓ Dev server running${NC}"
    SERVER_PID=""
fi

# Step 1: Run integration tests (fast)
echo ""
echo -e "${BLUE}Step 1/4: Running RTL Integration Tests${NC}"
echo "--------------------------------------"
pnpm test tests/i18n/rtl-layout.test.tsx --run

# Step 2: Run BiDi text tests
echo ""
echo -e "${BLUE}Step 2/4: Running Bidirectional Text Tests${NC}"
echo "----------------------------------------"
pnpm test tests/i18n/bidirectional-text.test.tsx --run

# Step 3: Run CSS logical properties validation
echo ""
echo -e "${BLUE}Step 3/4: Running CSS Logical Properties Validation${NC}"
echo "-------------------------------------------------"
pnpm test tests/i18n/css-logical-properties.test.ts --run

# Step 4: Run visual regression tests
echo ""
echo -e "${BLUE}Step 4/4: Running Visual Regression Tests${NC}"
echo "---------------------------------------"

# Run dashboard visual tests
echo ""
echo -e "${YELLOW}Testing Dashboard (Arabic & Hebrew)...${NC}"
npx playwright test rtl-dashboard --reporter=line

# Run settings visual tests
echo ""
echo -e "${YELLOW}Testing Settings (Arabic & Hebrew)...${NC}"
npx playwright test rtl-settings --reporter=line

# Generate HTML report
echo ""
echo -e "${BLUE}Generating test report...${NC}"
npx playwright show-report --quiet &

# Stop dev server if we started it
if [ -n "$SERVER_PID" ]; then
    echo ""
    echo -e "${BLUE}Stopping dev server...${NC}"
    kill $SERVER_PID
    echo -e "${GREEN}✓ Dev server stopped${NC}"
fi

# Summary
echo ""
echo "===================================="
echo -e "${GREEN}✓ RTL Visual Testing Complete!${NC}"
echo "===================================="
echo ""
echo "Test Results:"
echo "  • Integration Tests: PASSED"
echo "  • BiDi Text Tests: PASSED"
echo "  • CSS Validation: PASSED"
echo "  • Visual Regression: CHECK REPORT"
echo ""
echo "View detailed report:"
echo "  npx playwright show-report"
echo ""
echo "Test artifacts:"
echo "  • Screenshots: tests/visual/rtl-*.spec.ts-snapshots/"
echo "  • HTML Report: playwright-report/"
echo "  • Test Results: playwright-report/results.json"
echo ""
