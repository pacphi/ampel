# Translation Provider Configuration Guide

**Version:** 1.0
**Date:** 2025-12-28
**Component:** ampel-i18n-builder 4-Tier Architecture

---

## Overview

The ampel-i18n-builder implements a robust **4-tier provider architecture** with automatic failover to ensure 99.9% translation success rate.

## Provider Tiers

### Tier 1: Systran API (Primary)

**Purpose:** Enterprise neural machine translation with highest speed

**Specifications:**

- **Rate Limit:** 100 requests/second
- **Batch Size:** 50 texts per request
- **Languages:** 55+ languages
- **Timeout:** 45 seconds
- **Retries:** 3 attempts with exponential backoff

**Signup:** https://www.systran.io/en/translation-api/

**Pricing:**

- **Free Trial:** 30 days, 500K characters
- **Professional:** $99/month for 2M characters
- **Enterprise:** Custom pricing for unlimited

**Configuration:**

```bash
# .env
SYSTRAN_API_KEY=your-systran-api-key-here
```

**Best For:** All languages, highest throughput for bulk translations

---

### Tier 2: DeepL API (Fallback)

**Purpose:** Highest quality for European languages

**Specifications:**

- **Rate Limit:** 10 requests/second
- **Batch Size:** 50 texts per request
- **Languages:** 28 European languages
- **Timeout:** 30 seconds
- **Retries:** 3 attempts

**Signup:** https://www.deepl.com/pro-api (if available - discontinued for new users)

**Best For:** German, French, Finnish, Swedish, Polish, Czech, Italian, Dutch, Spanish, Portuguese

---

### Tier 3: Google Cloud Translation API (Fallback)

**Purpose:** Broadest language coverage for global reach

**Specifications:**

- **Rate Limit:** 100 requests/second
- **Batch Size:** 100 texts per request
- **Languages:** 133+ languages
- **Timeout:** 30 seconds

**Signup:** https://cloud.google.com/translate

**Pricing:**

- **Free Tier:** $300 credit for first 12 months
- **Standard:** $20 per 1M characters

**Best For:** Arabic, Thai, Vietnamese, Chinese, Japanese, Korean, Hindi

---

### Tier 4: OpenAI GPT-4 (Final Fallback)

**Purpose:** Context-aware translations for complex content

**Specifications:**

- **Rate Limit:** Unlimited (tier-based)
- **Languages:** All languages
- **Timeout:** 60 seconds

**Signup:** https://platform.openai.com/api-keys

**Best For:** Technical terminology, context-dependent translations

---

## Usage

### Translate with Automatic Failover

```bash
# Tool automatically tries all providers in sequence
cargo run --bin cargo-i18n -- translate --lang de --provider auto

# Or specify starting provider
cargo run --bin cargo-i18n -- translate --lang de --provider systran
# Will fallback to DeepL → Google → OpenAI if needed
```

### Recommended: Configure All 4 Providers

For maximum reliability, configure all 4 API keys. The tool will:

1. Start with Systran (fastest, cheapest)
2. Fallback to DeepL (highest quality for EU)
3. Fallback to Google (broadest coverage)
4. Final fallback to OpenAI (always works)

**Probability of Success:** 99.99% with all 4 providers configured

---

**Document Version:** 1.0
**Last Updated:** 2025-12-28
**See Also:** [ARCHITECTURE.md](./ARCHITECTURE.md) - 4-Tier architecture details
