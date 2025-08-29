# HodeauxLedger / Trust Architecture

The **substrate of truth** — a cryptographically signed, append-only ledger that anchors identity, policy, provenance, and time.

---

## 📦 Crates & Services

### `hodeauxledger-core`

-   The canonical **R⬢ (Rhex) record model** and validation logic.
-   Handles hashing, chaining, canonicalization, schema refs, and verification.
-   Append-only rules are enforced here.

### `hodeauxledger-io`

-   IO utilities for persisting and caching R⬢.
-   SQLite cache drivers, file readers/writers, and projection helpers.
-   Bridges core records into practical storage.

### `hodeauxledger-proto`

-   Network framing and protocol design.
-   Wire-level encoding of R⬢ for transport.
-   Connection negotiation primitives (`system:hello`, framing, nonce handling).

### `hodeauxledger-services`

-   Implements **stewards** and system services.
-   Policy enforcement, quorum checks, schema validation, and health monitors.
-   Runs the background logic that “keeps the chain honest.”

### `keytool`

-   CLI utility for **Ed25519 key management**.
-   Generates root keys, daily keys, device keys.
-   Supports `key:grant` / `key:revoke` workflows.
-   Signs and verifies R⬢

### `ledger`

-   CLI and library for **R⬢ crafting**.
-   Build and validate records.
-   Used for local authoring and testing of ledger entries.

### `usher`

-   Transport client for the ledger.
-   Connects to ushers, submits/receives R⬢, syncs scope heads.
-   CLI for interacting with the network as a participant.

### `usherd`

-   Host your own usher node.
-   Runs as a combined gRPC/HTTP server.
-   Accepts incoming records, applies validation, forwards to peers.
-   Acts as a transport and authority signer for assigned scopes.

---

## ⚡ Principles

-   **Immutable** — append-only, never edit or delete.
-   **Transparent** — every record provable, every signature verifiable.
-   **Universal** — motors, food, news, identity — all scopes fit.
-   **People-first** — built under **HodoTrust**, no dark patterns.

---

## 🔑 Quickstart

```bash
# Build and test core crates
cargo build --workspace
cargo test --workspace

# Generate a key
cargo run -p keytool -- keygen --scope myscope.veroself

# Craft a record
cargo run -p ledger -- record:create --scope myscope.veroself --data '{"hello":"world"}'

# Submit to usher
cargo run -p usher -- submit myrecord.rhex --host localhost --port 1984
```

---

## 📜 Canon

-   Records (R⬢) are ≤1.5 KB; `data` field ≤1024 bytes.
-   See [R⬢ Structure](docs/RHEX-STRUCT.md) for data structure
-   All signing is Ed25519.
-   Keys and policies are **in-band**, not external.

---

## 🧩 Roadmap

-   Full steward suite (💩 flags → info/warn/error).
-   Policy-driven schema enforcement.
-   Usher peer discovery and gossip protocol.
-   Visual explorers (glyph strips, Matrix rain).

---

## 👀 Final Note

The ledger is forever. Build as if history is reading — because it is.
