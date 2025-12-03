# Frontend Makefile
# Run from frontend/ directory or via root Makefile

.PHONY: help install dev build test lint format type-check clean preview

# Default target
help:
	@echo "Frontend targets:"
	@echo "  install     - Install dependencies with pnpm"
	@echo "  dev         - Start development server"
	@echo "  build       - Build for production"
	@echo "  test        - Run tests"
	@echo "  test-watch  - Run tests in watch mode"
	@echo "  test-cov    - Run tests with coverage"
	@echo "  lint        - Run ESLint"
	@echo "  format      - Format code with Prettier"
	@echo "  format-check- Check formatting"
	@echo "  type-check  - Run TypeScript type checking"
	@echo "  clean       - Remove build artifacts"
	@echo "  preview     - Preview production build"

# Dependencies
install:
	pnpm install

# Development
dev:
	pnpm run dev

# Build
build:
	pnpm run build

preview:
	pnpm run preview

# Testing
test:
	pnpm run test -- --run

test-watch:
	pnpm run test

test-cov:
	pnpm run test -- --run --coverage

# Quality
lint:
	pnpm run lint

format:
	pnpm exec prettier --write "src/**/*.{ts,tsx,css,json}"

format-check:
	pnpm exec prettier --check "src/**/*.{ts,tsx,css,json}"

type-check:
	pnpm run type-check

# Cleanup
clean:
	rm -rf dist node_modules/.vite
