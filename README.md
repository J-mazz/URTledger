# DynamicInventoryEngine

Rust backend + Slint UI for a dynamic inventory ledger.

This repo contains the typed spec model, SQLite persistence, and a Slint UI skeleton.

Next steps: run `cargo build` and `cargo test`. Use `build.sh` to run the host build.

---

## 📦 Packaging & Installation

**Packaging (build and bundle)** ✅

- A helper script `package_app.sh` exists in the project root. It builds a release binary, copies an `app_icon.png` (if present), and creates a compressed installer archive at `dist/URTledger_v1.0.0_Installer.tar.gz`.
- To provide a custom icon, place `app_icon.png` in the project root before running the script. (If none is provided, `assets/URTlogo.png` is a good fallback.)

Commands:

```bash
# Create the installer (release build)
./package_app.sh
```

**Installing (what your father will do)** 🚀

- Extract the archive and run the bundled installer:

```bash
tar -xzf URTledger_v1.0.0_Installer.tar.gz
cd URTledger
./install.sh
```

- The installer places the app in `~/.local/share/URTledger` and creates a desktop entry in `~/.local/share/applications` (so it appears in the Applications menu). The launcher is `~/.local/share/URTledger/launch.sh`.

**Quick smoke test (no permanent changes)** 🧪

Run the installer with a temporary HOME to verify it behaves as expected without modifying your real profile:

```bash
TMPDIR=$(mktemp -d)
# extract + install into temporary home
tar -xzf dist/URTledger_v1.0.0_Installer.tar.gz -C "$TMPDIR"
env HOME="$TMPDIR/home" bash -c "cd '$TMPDIR/URTledger' && ./install.sh"
```

**Notes** ⚠️

- The app's SQLite DB is stored in `~/.local/share/URTledger` (so the database won't appear in random folders).
- If you want a different packaging version string, edit `VERSION` in `package_app.sh` before running it.

---

If you'd like, I can add a short one-page README for your father with step-by-step screenshots. Let me know!