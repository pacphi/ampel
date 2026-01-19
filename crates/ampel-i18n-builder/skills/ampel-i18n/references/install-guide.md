# 🚀 First-Time Installation Guide for ampel-i18n-builder

**New user? Start here!** This guide walks you through installing everything from scratch.

---

## What You Need

This tool requires **Rust and Cargo** (the Rust package manager). Don't worry if you don't have them — Claude Code can help you install everything in under 5 minutes.

---

## Installation Steps

### Step 1: Check if You Already Have Rust

Ask Claude Code:

```
Can you check if I have Rust and Cargo installed?
Run: cargo --version
```

- ✅ **If it shows a version**: Skip to Step 3
- ❌ **If you see "command not found"**: Continue to Step 2

---

### Step 2: Install Rust (First-Time Only)

**Option A: Let Claude Code Do It (Recommended)**

Tell Claude Code:

```
Please install Rust for me using the official installer.
Guide me through each step, run the commands, and verify it works.

I'm on [macOS/Linux/Windows].
```

Claude will handle the OS-specific installation process.

**Option B: Manual Installation**

- **macOS/Linux**: Open terminal and run:

  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  source $HOME/.cargo/env
  ```

- **Windows**: Download and run the installer from https://rustup.rs/

After installation, **restart your terminal** or run:

```bash
source $HOME/.cargo/env  # macOS/Linux only
```

Verify it worked:

```bash
cargo --version
```

You should see something like `cargo 1.XX.X`.

---

### Step 3: Install ampel-i18n-builder

Tell Claude Code:

```
Please install ampel-i18n-builder for me:
cargo install ampel-i18n-builder

Then verify it's installed by running:
ampel-i18n --version
```

**What this does:**

- Downloads and compiles the latest version from crates.io
- Takes 2-5 minutes depending on your machine
- Installs the `ampel-i18n` command (alias for `ampel-i18n`)

---

### Step 4: Verify Installation

Run either command (they're the same):

```bash
ampel-i18n --version
# OR
ampel-i18n --version
```

✅ **You should see**: `ampel-i18n 0.X.X` or similar

❌ **If you see "command not found"**: See Troubleshooting below

---

## ⚠️ Troubleshooting

### "cargo: command not found"

**Cause**: Your terminal doesn't know where cargo is installed.

**Fix**:

```bash
# macOS/Linux
source $HOME/.cargo/env

# Or add to your shell profile (~/.bashrc, ~/.zshrc, etc.):
export PATH="$HOME/.cargo/bin:$PATH"
```

After running this, restart your terminal and try again.

---

### "ampel-i18n: command not found"

**Cause**: The binary wasn't installed to your PATH.

**Fix**:

```bash
# Check if cargo bin directory is in PATH
echo $PATH | grep -q "\.cargo/bin" && echo "✓ cargo/bin is in PATH" || echo "✗ cargo/bin NOT in PATH"

# If not in PATH, add it:
export PATH="$HOME/.cargo/bin:$PATH"

# Then verify:
ampel-i18n --version
```

---

### "Permission denied" Errors

**Cause**: The installed binary doesn't have execute permissions.

**Fix**:

```bash
chmod +x ~/.cargo/bin/ampel-i18n
```

---

### "Failed to compile" Errors

**Cause**: Missing build dependencies (C compiler, linker, etc.)

**Fix by OS**:

**macOS:**

```bash
xcode-select --install
```

**Ubuntu/Debian:**

```bash
sudo apt-get update
sudo apt-get install build-essential pkg-config libssl-dev
```

**Fedora:**

```bash
sudo dnf install gcc openssl-devel
```

**Windows:**

- Install Visual Studio Build Tools: https://visualstudio.microsoft.com/downloads/
- Select "Desktop development with C++"

---

### Still Stuck?

Ask Claude Code:

```
I'm having trouble installing ampel-i18n-builder. Can you help debug?

Here's the error I'm seeing:
[PASTE ERROR MESSAGE]
```

Claude can diagnose the issue and provide OS-specific solutions.

---

## ✅ Installation Complete!

Once you see the version number, you're ready to go.

**Next steps:**

1. Read `getting-started.md` for a 10-minute quick start
2. Try the sample prompts in `sample-prompts.md`
3. Start translating your app!

**Don't want to read docs?** Just tell Claude Code:

```
/ampel-i18n:localize

I just installed ampel-i18n-builder and want to translate my project.
This is a [React/Vue/Rust] app using [i18next/vue-i18n/rust-i18n].
Walk me through setup from scratch.
```

---

## Command Reference

Both commands work the same (they're aliases):

```bash
ampel-i18n sync              # Generate translations
ampel-i18n coverage          # Check translation status
ampel-i18n missing           # Find untranslated keys
ampel-i18n generate-types    # Create type definitions

# Same as:
ampel-i18n sync
ampel-i18n coverage
ampel-i18n missing
ampel-i18n generate-types
```

---

_Installation taking too long? That's normal for Rust projects (first-time compile can take 5-10 minutes). Grab a coffee! ☕_
