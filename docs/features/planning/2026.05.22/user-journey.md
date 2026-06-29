# User Journeys

## Ampel Upgrade Intelligence

---

## Journey 1 — First-Time Setup (Onboarding)

**Persona:** Thiago (Solo Developer)  
**Goal:** Get Ampel running and managing his repos across GitHub and GitLab

```
Step 1: Install Ampel
  ├── docker compose up (single command)
  ├── Navigate to http://localhost:3000
  └── Create account → org → first team

Step 2: Connect Git Providers
  ├── GitHub: Settings → Providers → Add GitHub
  │   ├── Enter Personal Access Token
  │   ├── System validates credentials
  │   └── ✓ "Connected to github.com as @thiago"
  │
  └── GitLab: Repeat for self-hosted GitLab instance
      └── ✓ "Connected to gitlab.mycompany.com as @thiago"

Step 3: Discover Repositories
  ├── Click "Discover Repositories" (GitHub tab)
  ├── System lists all accessible repos
  ├── Thiago selects 8 repos he wants to manage
  └── Click "Track Selected"

Step 4: Initial Ecosystem Scan (automatic, ~2–5 min per repo)
  ├── Background job: scan_ecosystems runs
  ├── Dashboard shows: "Scanning repos... 3/8 complete"
  ├── Each repo gets ecosystem tags: [Maven] [GitHub Actions] [Docker]
  └── ✓ Scan complete. 8 repos, 4 ecosystems detected.

Step 5: Review Findings
  ├── Dashboard: "23 available upgrades across 8 repos"
  ├── Breakdown: 18 patch, 4 minor, 1 major
  ├── 2 CVEs: 1 critical (EPSS 0.78), 1 moderate
  └── Thiago clicks CVE card → sees: "Affects 3 repos, upgrade available"

Step 6: Configure Auto-Merge Policy
  ├── Settings → Upgrade Policy
  ├── Default shown: "Patch: auto-merge | Minor: manual review | Major: blocked"
  ├── Thiago enables: "Minor: auto-merge after 7 days"
  └── ✓ Policy saved. Applies to all repos.

Step 7: Create First Upgrade PRs
  ├── Click "Create CVE Fix PRs" → PRs created on GitHub and GitLab
  ├── Click "Create Patch Update PRs" → 18 PRs created
  └── Dashboard: "21 PRs open. CI running."

  [3 minutes later]
  ├── 18 patch PRs: CI passed. Awaiting auto-merge gate (3 days).
  └── 3 CVE PRs: CI passed. Auto-merging now (security tier, no age gate).

  [3 days later]
  └── 18 patch PRs auto-merged. Fleet is current on patches.

Total time spent by Thiago: ~15 minutes
```

---

## Journey 2 — Daily Upgrade Review

**Persona:** Priya (Platform Engineer)  
**Goal:** Morning review of fleet upgrade status, 15 minutes before standup

```
Morning Dashboard (8:45am)
  ├── 📊 Fleet Status
  │   ├── 147 repos total
  │   ├── 12 new upgrade plans since yesterday
  │   ├── 7 PRs merged overnight (auto-merge)
  │   ├── 3 PRs need attention (CI failures)
  │   └── 1 new CVE detected (EPSS 0.45, moderate)
  │
  ├── 🔴 Attention Required (3 items)
  │   ├── PR #445 (payment-service): CI failed
  │   │   └── Brain Server: "Test failure in PaymentControllerTest. Similar 
  │   │       failure seen in 4 prior upgrades: root cause is MockMvc 
  │   │       configuration change in Spring Boot 3.3.4. Fix: update test config."
  │   ├── PR #449 (auth-service): Awaiting review (minor update, no auto-merge)
  │   └── PR #451 (reporting-service): Pre-flight blocked
  │       └── "12 deprecated Criteria API usages detected before Hibernate upgrade"
  │
  └── 🟡 Pending (auto-merge eligible, waiting on age gate)
      └── 9 PRs: patch updates, CI green, 1–2 days remaining

Priya's Actions (5 minutes):
  1. PR #445: reads Brain Server explanation → approves suggested test fix
     → System re-runs CI → passes → queued for auto-merge
  2. PR #449: reviews diff (minor Spring Boot update) → approves merge
  3. PR #451: clicks "Pre-flight Report" → reads analysis
     → Creates Jira ticket for `reporting-service` team with attached pre-flight report
     → Marks upgrade as "deferred — waiting on code migration"

Result: 3 items resolved in 5 minutes. Auto-merge handles the other 9 overnight.
```

---

## Journey 3 — Responding to a CVE (Incident-Driven)

**Persona:** Marcus (VP Engineering)  
**Goal:** Contain CVE-2026-XXXX within SLA (4 hours for critical, 24 hours for high)

```
10:12am — CVE Published
  └── Ampel OSV feed detects CVE-2026-XXXX (commons-text, EPSS 0.84)

10:13am — Alert fires
  ├── Notification: "Critical CVE detected — 47 repos affected"
  └── Marcus receives Slack alert (via existing notification hook)

10:15am — Marcus opens Ampel
  ├── CVE dashboard:
  │   ├── Affected repos: 47 (ranked by criticality × staleness)
  │   ├── Patch available: commons-text 1.10 (released 6 days ago, age gate already met)
  │   └── Pre-flight: "No known incompatibilities for this patch"
  │
  └── Marcus clicks "Create security PRs for all 47 repos"

10:17am — PRs created on GitHub Enterprise, GitLab, Azure DevOps
  └── "47 PRs opened. CI running."

10:32am — First results
  ├── 41 PRs: CI passed. Auto-merging (security tier, no age gate).
  └── 6 PRs: issues detected
      ├── 3: CI failures (flaky tests — system flags as known-flaky, offers re-run)
      ├── 2: required reviewer approval needed per branch protection rules
      └── 1: merge conflict (main branch moved ahead since PR opened)

10:45am — Follow-up
  ├── 3 flaky PRs: CI re-run → 2 pass, 1 still failing (real issue)
  ├── 2 approval-required: Marcus reviews and approves manually
  └── 1 conflict: system auto-rebases → CI passes → auto-merges

11:15am — Status
  ├── 46/47 resolved. 1 genuine CI failure under investigation.
  └── Compliance report generated: "46 repos patched within SLA (63 minutes)"

4:00pm — Final resolution
  └── Last repo: developer finds real incompatibility, applies fix, merges manually
```

---

## Journey 4 — Running a Major Version Migration Campaign

**Persona:** Priya (Platform Engineer)  
**Goal:** Migrate all Spring Boot 3.x repos to Spring Boot 4.0.x

```
Week 1 — Planning

  Monday:
    ├── Priya creates migration campaign: "Spring Boot 4 Migration Q2 2026"
    ├── Target: all 35 repos with Spring Boot 3.x
    └── System runs analysis (15 min background job)
    
  Analysis results:
    ├── 35 repos identified
    ├── Pre-flight summary:
    │   ├── 20 repos: clean (readiness > 0.75)
    │   ├── 10 repos: medium complexity (50–200 incompatibility sites)
    │   └── 5 repos: high complexity (200+ sites, custom Security config)
    ├── Estimated total transformation: 3,400 code sites across fleet
    └── Estimated total effort: "~40 hours human review (5 complex repos)"
    
  Priya reviews → adjusts wave assignments:
    ├── Canary: 2 repos (hand-picked, non-critical)
    ├── Wave 1: 18 repos (readiness > 0.75)
    ├── Wave 2: 10 repos (medium)
    └── Wave 3 (human-assisted): 5 repos (high complexity)

  Week 1 / Tuesday: Canary PRs opened
    ├── 2 PRs with: version bump + OpenRewrite code migration patch
    ├── ruvllm generates commit messages explaining each change
    └── CI runs on canaries

  Week 1 / Wednesday: Canary results
    ├── Both canaries: CI passed ✓
    ├── 48h observation window starts
    └── No production issues detected (connected monitoring shows no alerts)

  Week 2 — Wave 1
    Monday: 18 Wave 1 PRs opened
    ├── 15 pass CI immediately
    ├── 2 fail: "RestClientCustomizer" pattern
    │   └── Brain Server: "This pattern affected 7 prior repos.
    │       Resolution: replace with WebClientCustomizer. 
    │       ruvllm can generate the fix — approve?"
    ├── Priya approves: fix generated, PRs updated, CI re-run → both pass
    └── 1 fail: genuine incompatibility not in Brain Server
        └── Priya investigates manually; documents finding → contributes to Brain Server
        
  Week 2 result: 18/18 Wave 1 merged. Brain Server updated with new pattern.

  Week 3 — Wave 2
    ├── 10 PRs opened; Brain Server now has "RestClientCustomizer" pattern
    ├── 9 pass CI first attempt (pattern already resolved automatically)
    └── 1 new issue → documented → fleet learns

  Week 4 — Wave 3 (human-assisted)
    ├── 5 complex repos assigned to respective dev teams
    ├── Each team gets: pre-flight report, Brain Server explanation, suggested fix
    ├── Developers apply fixes; Priya monitors campaign dashboard
    └── All 5 merged by end of week

  Campaign complete: 35/35 repos migrated. Brain Server has 35 new trajectories.
  Next time this migration runs (for another org via federated learning): estimated
  success rate improves from 71% first-run to 88% first-run.
```

---

## Journey 5 — Investigating Why an Upgrade Failed

**Persona:** Thiago (Solo Developer)  
**Goal:** Understand why the Node.js upgrade PR failed and fix it

```
Context: PR #78 on "my-api-service" failed CI
  └── Upgrade: Node.js 20 → 22 (LTS, minor)

Step 1: Thiago views PR in Ampel dashboard
  ├── CI status: ❌ Failed
  ├── Checks: "integration-tests" failed
  └── Click "Why did this fail?"

Step 2: Brain Server Root Cause Analysis
  └── System embeds CI failure message → searches Brain Server
  
  Result:
  ┌─────────────────────────────────────────────────────┐
  │ Root Cause Analysis                                  │
  │                                                      │
  │ Failure pattern matches 12 prior cases.             │
  │                                                      │
  │ Root cause: Node.js 22 changed the default value    │
  │ of `--openssl-legacy-provider` flag. Tests using    │
  │ webpack 4 fail because webpack 4 uses an OpenSSL   │
  │ API removed in Node.js 22.                          │
  │                                                      │
  │ Resolution options:                                  │
  │ 1. Upgrade webpack: 4.x → 5.x (recommended)        │
  │ 2. Add NODE_OPTIONS=--openssl-legacy-provider       │
  │    to CI workflow (workaround, not recommended)     │
  │                                                      │
  │ Similar resolutions: 12 repos · 100% success rate  │
  └─────────────────────────────────────────────────────┘

Step 3: Thiago chooses option 1 (upgrade webpack)
  ├── Click "Create companion upgrade plan: webpack 4 → 5"
  ├── System generates pre-flight for webpack 5 (no incompatibilities)
  └── New PR created: "chore: upgrade webpack from 4 to 5"

Step 4: Both PRs updated
  ├── Node.js 22 PR: re-run CI after webpack upgrade merged
  └── Webpack 5 PR: CI passes → auto-merged (patch tier)

Step 5: Node.js PR CI re-run
  └── ✓ All checks pass. Auto-merged.

Total time: 8 minutes. No Stack Overflow, no manual debugging.
```

---

## Journey 6 — Setting Up Federated Intelligence (Enterprise)

**Persona:** Aiko (DevEx Lead) with 3 business units  
**Goal:** Enable cross-BU learning while preserving code privacy

```
Step 1: Each BU deploys Brain Server sidecar
  ├── BU-1: docker run ruvnet/mcp-brain-server (Cloud Run config)
  ├── BU-2: Same
  ├── BU-3: Same
  └── Enterprise Brain Server (coordinator): Deployed centrally

Step 2: Configure Federation
  ├── Each BU Brain Server: Settings → Federation
  ├── Enterprise coordinator URL: https://brain.internal.company.com/sse
  ├── Privacy settings: ε=1.0 differential privacy enabled
  └── Categories to share: ["upgrade_patterns", "incompatibilities", "recipes"]
      Not shared: source_code, repository_names, proprietary_configs

Step 3: BU-2 runs Python 3.11 → 3.13 migration (20 repos)
  ├── 20 trajectories recorded in BU-2's SONA replay buffer
  ├── SONA runs training cycle: crystallizes "Python313Migration" pattern
  ├── LoRA weight delta generated
  └── Delta submitted to enterprise coordinator (brain_lora_submit)
      Note: only weight deltas, not code, are transmitted

Step 4: BU-1 and BU-3 sync
  ├── brain_lora_latest pulls consensus weights
  ├── BU-1 upgrade planner: "Python 3.11→3.13 + FastAPI" merge confidence = 0.91
  └── BU-1 generates 30 upgrade PRs with high confidence

Step 5: BU-1 results
  ├── 27/30 pass CI first attempt (91% — matches federated confidence prediction)
  ├── 3 failures: new patterns not in BU-2's experience
  ├── Documented → submitted back to federation
  └── Federated confidence updated to 0.87 (more accurate with more data)

Aiko's observation: "We got 91% first-run success on a migration we'd never done,
because another business unit had already done it — and we shared zero code."
```
