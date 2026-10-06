# AegisRAT — Safe Research Scaffold

This repository is a non-operational architecture scaffold for studying remote administration
and defensive security concepts.

The payload and C2 components are intentionally placeholders. They do not implement:
- persistence
- credential/keylogging
- stealth/evasion
- arbitrary command execution
- file exfiltration
- real remote C2

## Structure

```text
AegisRAT/
├── payload/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│     der as the workspace.

The included server is a local educational stub and does not accept or execute remote commands.
