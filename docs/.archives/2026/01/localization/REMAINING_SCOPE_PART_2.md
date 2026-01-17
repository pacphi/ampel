# Remaining I18n Scope - Part 2

## Overview

This document outlines the remaining internationalization work for the Ampel project. Part 1 covered the primary UI components (Dashboard, Settings core, Repositories, Merge, Analytics). Part 2 addresses the remaining components with hardcoded English strings.

**Date**: 2026-01-17
**Status**: Implementation Plan
**Languages**: 27 (including RTL: Arabic, Hebrew)

---

## 1. Components Requiring Translation

### 1.1 Settings Components

| Component                | File Path                                                       | Priority | Estimated Keys |
| ------------------------ | --------------------------------------------------------------- | -------- | -------------- |
| NotificationsSettings    | `frontend/src/components/settings/NotificationsSettings.tsx`    | High     | 35+            |
| BehaviorSettings         | `frontend/src/components/settings/BehaviorSettings.tsx`         | High     | 20+            |
| TokenInstructions        | `frontend/src/components/settings/TokenInstructions.tsx`        | Medium   | 45+            |
| RepositoryFilterSettings | `frontend/src/components/settings/RepositoryFilterSettings.tsx` | Medium   | 15+            |
| AccountCard              | `frontend/src/components/settings/AccountCard.tsx`              | High     | 15+            |

### 1.2 Account Management Pages

| Component        | File Path                                          | Priority | Estimated Keys |
| ---------------- | -------------------------------------------------- | -------- | -------------- |
| AccountsListPage | `frontend/src/pages/settings/AccountsListPage.tsx` | High     | 25+            |
| AddAccountPage   | `frontend/src/pages/settings/AddAccountPage.tsx`   | High     | 30+            |
| EditAccountPage  | `frontend/src/pages/settings/EditAccountPage.tsx`  | High     | 20+            |

### 1.3 Merge Components

| Component   | File Path                                       | Priority | Estimated Keys |
| ----------- | ----------------------------------------------- | -------- | -------------- |
| MergeDialog | `frontend/src/components/merge/MergeDialog.tsx` | High     | 25+            |

### 1.4 Error Handling

| Component     | File Path                                   | Priority | Estimated Keys |
| ------------- | ------------------------------------------- | -------- | -------------- |
| ErrorBoundary | `frontend/src/components/ErrorBoundary.tsx` | Medium   | 10+            |

---

## 2. New Translation Namespaces

Based on the analysis, we need to add the following namespaces:

### 2.1 `accounts` Namespace (NEW)

For account management UI including:

- Account listing and cards
- Add/Edit account forms
- Token validation states
- Provider-specific labels

### 2.2 `notifications` Namespace (NEW)

For notification settings including:

- Slack integration settings
- Email/SMTP configuration
- Merge notification options
- Test message functionality

### 2.3 `behavior` Namespace (NEW)

For behavior settings including:

- Merge strategies
- Auto-merge options
- Branch deletion settings
- Approval requirements

### 2.4 `providers` Namespace (NEW)

For provider-specific content including:

- Token creation instructions (GitHub, GitLab, Bitbucket)
- Permission scopes
- Step-by-step guides

### 2.5 Updates to Existing Namespaces

- `errors`: Add ErrorBoundary messages
- `settings`: Add repository visibility filter labels
- `merge`: Add MergeDialog-specific strings

---

## 3. Translation Key Structure

### 3.1 accounts.json

```json
{
  "title": "Provider Accounts",
  "description": "Manage your connected Git provider accounts",
  "addAccount": "Add Account",
  "addFirstAccount": "Add Your First Account",
  "noAccountsYet": "No accounts connected yet",
  "card": {
    "status": {
      "valid": "Valid",
      "invalid": "Invalid",
      "expired": "Expired",
      "pending": "Pending",
      "default": "Default"
    },
    "repositories": "repositories",
    "validated": "Validated",
    "expires": "Expires",
    "neverExpires": "Never expires",
    "actions": {
      "edit": "Edit account",
      "delete": "Delete account",
      "validate": "Validate token",
      "setDefault": "Set as default"
    }
  },
  "add": {
    "title": "Add Provider Account",
    "subtitle": "Connect a new Git provider account",
    "backToAccounts": "Back to Accounts",
    "form": {
      "provider": "Provider",
      "selectProvider": "Select a provider",
      "accountLabel": "Account Label",
      "accountLabelPlaceholder": "e.g., Work GitHub, Personal GitLab",
      "accountLabelDescription": "A friendly name to identify this account",
      "token": "Personal Access Token",
      "tokenPlaceholder": "Enter your personal access token",
      "tokenDescription": "Your token is encrypted and stored securely"
    },
    "submit": "Connect Account",
    "submitting": "Connecting..."
  },
  "edit": {
    "title": "Edit Account",
    "subtitle": "Update account settings",
    "backToAccounts": "Back to Accounts",
    "notFound": "Account not found",
    "accountInfo": "Account Information",
    "form": {
      "accountLabel": "Account Label",
      "accountLabelPlaceholder": "e.g., Work GitHub",
      "token": "Personal Access Token",
      "tokenPlaceholder": "Enter new token to update",
      "tokenDescription": "Leave empty to keep current token",
      "currentToken": "Current token"
    },
    "submit": "Save Changes",
    "submitting": "Saving..."
  },
  "delete": {
    "title": "Delete Account",
    "description": "Are you sure you want to delete the account \"{{label}}\"? This will not delete repositories, but they will need to be re-linked to another account.",
    "cancel": "Cancel",
    "confirm": "Delete",
    "deleting": "Deleting..."
  },
  "toast": {
    "connected": "Account connected",
    "connectedDescription": "Successfully connected {{provider}} account",
    "connectionFailed": "Failed to add account",
    "updated": "Account updated",
    "updatedDescription": "Account settings have been saved",
    "updateFailed": "Failed to update account",
    "deleted": "Account deleted",
    "deletedDescription": "The account has been removed successfully",
    "deleteFailed": "Failed to delete account",
    "tokenValid": "Token valid",
    "tokenValidDescription": "The account token is valid and active",
    "tokenInvalid": "Token invalid",
    "tokenInvalidDescription": "The token is invalid or expired",
    "validationFailed": "Validation failed",
    "defaultUpdated": "Default account updated",
    "defaultUpdatedDescription": "The default account has been changed",
    "defaultUpdateFailed": "Failed to update default"
  }
}
```

### 3.2 notifications.json

```json
{
  "title": "Notification Settings",
  "slack": {
    "title": "Slack Notifications",
    "enable": "Enable Slack notifications",
    "enableDescription": "Send notifications to a Slack channel",
    "webhookUrl": "Webhook URL",
    "webhookUrlPlaceholder": "https://hooks.slack.com/services/...",
    "webhookUrlDescription": "Create an incoming webhook in your Slack workspace",
    "channel": "Channel (optional)",
    "channelPlaceholder": "#general",
    "channelDescription": "Override the default channel configured in the webhook",
    "testMessage": "Send Test Message",
    "testSending": "Sending...",
    "testSuccess": "Test message sent",
    "testFailed": "Failed to send test message"
  },
  "email": {
    "title": "Email Notifications",
    "enable": "Enable email notifications",
    "enableDescription": "Send notifications via email",
    "smtpHost": "SMTP Host",
    "smtpHostPlaceholder": "smtp.example.com",
    "smtpPort": "SMTP Port",
    "smtpPortPlaceholder": "587",
    "smtpUser": "SMTP Username",
    "smtpUserPlaceholder": "user@example.com",
    "smtpPassword": "SMTP Password",
    "smtpPasswordPlaceholder": "Enter password",
    "fromAddress": "From Address",
    "fromAddressPlaceholder": "noreply@example.com",
    "toAddresses": "To Addresses",
    "toAddressesPlaceholder": "team@example.com, dev@example.com",
    "toAddressesDescription": "Comma-separated list of email addresses"
  },
  "merge": {
    "title": "Merge Notifications",
    "notifyOnSuccess": "Notify on successful merge",
    "notifyOnSuccessDescription": "Send notification when PRs are merged successfully",
    "notifyOnFailure": "Notify on merge failure",
    "notifyOnFailureDescription": "Send notification when merge operations fail",
    "notifyOnConflict": "Notify on merge conflicts",
    "notifyOnConflictDescription": "Send notification when merge conflicts are detected"
  },
  "toast": {
    "updated": "Settings updated",
    "updatedDescription": "Notification settings have been saved",
    "updateFailed": "Failed to update settings"
  }
}
```

### 3.3 behavior.json

```json
{
  "title": "Merge Behavior",
  "strategy": {
    "title": "Default Merge Strategy",
    "description": "Choose how pull requests should be merged by default",
    "squash": "Squash and merge",
    "squashDescription": "Combine all commits into one",
    "merge": "Create a merge commit",
    "mergeDescription": "All commits preserved",
    "rebase": "Rebase and merge",
    "rebaseDescription": "Linear commit history"
  },
  "delay": {
    "title": "Merge Delay (seconds)",
    "description": "Wait time between consecutive merges",
    "placeholder": "5"
  },
  "options": {
    "deleteBranch": "Delete branches after merge",
    "deleteBranchDescription": "Automatically delete source branches after successful merge",
    "requireApproval": "Require approval before merge",
    "requireApprovalDescription": "Wait for manual approval before executing merge",
    "allowNoReviews": "Allow merge without reviews",
    "allowNoReviewsDescription": "Allow merging PRs that have no reviews"
  },
  "toast": {
    "updated": "Settings updated",
    "updatedDescription": "Merge behavior settings have been saved",
    "updateFailed": "Failed to update settings"
  }
}
```

### 3.4 providers.json

```json
{
  "title": "Provider Instructions",
  "github": {
    "title": "GitHub Token Instructions",
    "steps": {
      "step1": "Go to GitHub Settings → Developer settings → Personal access tokens → Tokens (classic)",
      "step2": "Click \"Generate new token (classic)\"",
      "step3": "Give your token a descriptive name",
      "step4": "Select the required scopes:",
      "step5": "Click \"Generate token\" and copy it immediately"
    },
    "scopes": {
      "repo": "repo (Full control of private repositories)",
      "readOrg": "read:org (Read org and team membership)",
      "readUser": "read:user (Read user profile data)"
    },
    "note": "Fine-grained tokens are also supported with repository access permissions."
  },
  "gitlab": {
    "title": "GitLab Token Instructions",
    "steps": {
      "step1": "Go to GitLab → User Settings → Access Tokens",
      "step2": "Enter a token name and optional expiration date",
      "step3": "Select the required scopes:",
      "step4": "Click \"Create personal access token\" and copy it"
    },
    "scopes": {
      "api": "api (Full API access)",
      "readApi": "read_api (Read-only API access)",
      "readRepository": "read_repository (Read repository access)"
    }
  },
  "bitbucket": {
    "title": "Bitbucket App Password Instructions",
    "steps": {
      "step1": "Go to Bitbucket → Personal settings → App passwords",
      "step2": "Click \"Create app password\"",
      "step3": "Enter a label for the password",
      "step4": "Select the required permissions:",
      "step5": "Click \"Create\" and copy the password"
    },
    "permissions": {
      "accountRead": "Account: Read",
      "repositoriesRead": "Repositories: Read",
      "repositoriesWrite": "Repositories: Write",
      "pullRequestsRead": "Pull requests: Read",
      "pullRequestsWrite": "Pull requests: Write"
    }
  },
  "common": {
    "securityNote": "Keep your token secure and never share it publicly.",
    "expirationWarning": "Consider setting an expiration date for better security.",
    "minimalScopes": "Only grant the minimum required permissions."
  }
}
```

### 3.5 Updates to errors.json

Add to existing errors.json:

```json
{
  "boundary": {
    "title": "Something went wrong",
    "description": "An unexpected error occurred. Please try again.",
    "details": "Error Details (Dev Only)",
    "tryAgain": "Try Again",
    "reload": "Reload Page"
  }
}
```

### 3.6 Updates to settings.json

Add repository visibility section:

```json
{
  "repositories": {
    "visibility": {
      "title": "Repository Visibility Filters",
      "description": "Choose which types of repositories to display",
      "public": "Show public repositories",
      "publicDescription": "Display repositories that are publicly accessible",
      "private": "Show private repositories",
      "privateDescription": "Display repositories that are private",
      "archived": "Show archived repositories",
      "archivedDescription": "Display repositories that have been archived",
      "bitbucketNote": "Note: Bitbucket visibility may differ based on workspace settings"
    }
  }
}
```

### 3.7 Updates to merge.json

Add MergeDialog section:

```json
{
  "dialog": {
    "title": "Merge Pull Request",
    "subtitle": "Configure merge options for this PR",
    "strategy": {
      "title": "Merge Strategy",
      "squash": "Squash and merge",
      "squashDescription": "Combine all commits into one",
      "merge": "Create a merge commit",
      "mergeDescription": "All commits preserved",
      "rebase": "Rebase and merge",
      "rebaseDescription": "Linear commit history"
    },
    "deleteBranch": "Delete branch after merge",
    "cancel": "Cancel",
    "merge": "Merge PR",
    "merging": "Merging..."
  },
  "toast": {
    "success": "PR Merged",
    "successDescription": "Successfully merged #{{number}}: {{title}}",
    "failed": "Merge failed",
    "failedDescription": "Failed to merge #{{number}}: {{error}}"
  }
}
```

---

## 4. Implementation Steps

### Phase 1: Create New Locale Files

1. Create `frontend/public/locales/en/accounts.json`
2. Create `frontend/public/locales/en/notifications.json`
3. Create `frontend/public/locales/en/behavior.json`
4. Create `frontend/public/locales/en/providers.json`
5. Update `frontend/public/locales/en/errors.json` with boundary section
6. Update `frontend/public/locales/en/settings.json` with visibility section
7. Update `frontend/public/locales/en/merge.json` with dialog section

### Phase 2: Update i18n Configuration

1. Register new namespaces in `frontend/src/i18n/index.ts`
2. Update `frontend/src/i18n/types.ts` with new interfaces

### Phase 3: Update Components

1. Add `useTranslation` hooks to each component
2. Replace all hardcoded strings with `t()` calls
3. Ensure proper namespace usage

### Phase 4: Run Translation Builder

```bash
cd crates/ampel-i18n-builder
cargo run -- translate \
  --input ../frontend/public/locales/en \
  --output ../frontend/public/locales \
  --languages all \
  --format json
```

### Phase 5: Regenerate Types

```bash
cd crates/ampel-i18n-builder
cargo run -- generate-types \
  --input ../frontend/public/locales/en \
  --output ../frontend/src/i18n/types.ts
```

### Phase 6: Testing

1. Run frontend tests: `make test-frontend`
2. Verify all translations load correctly
3. Test RTL languages (Arabic, Hebrew)
4. Validate placeholder interpolation

---

## 5. File Organization

### Frontend Locales Structure

```
frontend/public/locales/
├── en/
│   ├── accounts.json      (NEW)
│   ├── analytics.json
│   ├── behavior.json      (NEW)
│   ├── common.json
│   ├── dashboard.json
│   ├── errors.json        (UPDATE)
│   ├── merge.json         (UPDATE)
│   ├── notifications.json (NEW)
│   ├── providers.json     (NEW)
│   ├── repositories.json
│   ├── settings.json      (UPDATE)
│   └── validation.json
├── ar/
│   └── ... (all namespaces)
├── ... (25 more languages)
└── zh-TW/
    └── ... (all namespaces)
```

### Backend Locales Structure

```
crates/ampel-api/locales/
├── en/
│   └── api.yaml
├── ... (all 27 languages)
```

---

## 6. Validation Checklist

- [ ] All hardcoded strings extracted
- [ ] All new namespaces created
- [ ] i18n configuration updated
- [ ] Types regenerated
- [ ] All 27 languages translated
- [ ] RTL languages tested
- [ ] Placeholder interpolation verified
- [ ] Frontend tests passing
- [ ] No console warnings for missing keys

---

## 7. Estimated Effort

| Task                    | Estimated Time |
| ----------------------- | -------------- |
| Create locale files     | 1 hour         |
| Update components       | 2 hours        |
| Run translation builder | 30 minutes     |
| Regenerate types        | 15 minutes     |
| Testing & validation    | 1 hour         |
| **Total**               | ~5 hours       |

---

## 8. Dependencies

- `ampel-i18n-builder` crate with valid API keys in `.env`
- Translation service providers (Systran, DeepL, Google, OpenAI)
- Node.js and pnpm for frontend build

---

## 9. Rollback Plan

If issues arise:

1. Revert locale file changes via git
2. Restore previous `types.ts`
3. Components will continue working with hardcoded strings

---

_Document version: 2.0_
_Last updated: 2026-01-17_
