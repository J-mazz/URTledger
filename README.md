# DynamicInventoryEngine

Rust backend + Slint UI for a dynamic inventory ledger.

This repo contains the typed spec model, SQLite persistence, and a Slint UI skeleton.

Next steps: run `cargo build` and `cargo test`. Use `build.sh` to run the host build.

---

## 📦 Packaging & Installation

**Packaging (build and bundle)** ✅

- A helper script `package_app.sh` exists in the project root and creates a compressed installer archive at `dist/URTledger_v1.0.0_Installer.tar.gz`.


Commands:

```bash
# Create the installer (release build)
./package_app.sh
```

**Installing** 🚀

- Extract the archive and run the bundled installer:

```bash
tar -xzf URTledger_v1.0.0_Installer.tar.gz
cd URTledger
./install.sh
```

- The installer places the app in `~/.local/share/URTledger` and creates a desktop entry in `~/.local/share/applications` The launcher is `~/.local/share/URTledger/launch.sh`.

