# Genesis Preflight v1 — Trust Architecture (R⬢ / GT)

> A tight, dependency-ordered checklist to stand up Root and your first child scope(s) without footguns. Uses your conventions: R⬢ record caps, CBOR canonicalization, Ed25519, tri-signature (Author/Usher/Quorum), and **scope\:create lives in the parent**.

---

## Legend

-   **Roles**: **Author** (creator of content), **Usher** (transport attest), **Quorum** (additional attestor(s)).
-   **“All 3”** = signatures from Author + Usher + Quorum on the same R⬢.
-   **Caps**: `data` ≤ 1024 bytes; typical R⬢ ≤ \~1.5 KB (unless quorum sigs accumulate).
-   **Invariant**: `scope:create` recorded in **parent** scope.

---

## Phase 0 — Prereqs & Materials

-   [ ] Installed: `keytool`, `rhex-pack` (CBOR), `rhex-verify`, `usherd` (Core), SQLite cache.
-   [ ] Hash canon.: JSON→canonical→CBOR before signing.
-   [ ] Time anchor noted: **GT\[0.00.00\@000]** at `js_at: 1752941587614` (Sat, Jul 19, 2025 09:13:07 MST). (For provenance comments only; records still use wallclock `at` field you defined.)
-   [ ] Root ledger path decided (e.g., `~/.trust/core.ledger`); empty or freshly sealed file.

**Artifacts directory layout** (suggested):

```
/genesis/
  keys/
    root.key
    usher-core.key
  rhex/
    00_root_scope.genesis.cbor
    01_root_policy.set.cbor
    02_core_key.grant.cbor
    03_core_authority.grant.cbor
    10_child_scope.request.cbor
    11_child_scope.create.cbor
    12_child_scope.genesis.cbor
    13_child_policy.set.cbor
    14_child_key.grant.cbor
    15_child_authority.grant.cbor
```

---

## Phase 1 — Create Root’s Key

-   [ ] `keytool generate -o genesis/keys/root.key -p <pass>` **or** `--hot`.
-   [ ] Record Root key fingerprint → `ROOT_FP`.
-   [ ] (Optional) derive daily/host device keys if policy will require; store but do **not** use for genesis.

**Gate A (verify):**

-   [ ] `keytool show genesis/keys/root.key` prints Ed25519 pub; matches expected FP.

---

## Phase 2 — R⬢: **Root scope\:genesis** (anchors time)

-   [ ] Build **scope\:genesis** (root) per schema.

    -   `previous_hash: null`
    -   `protocol` & `scope` = root scope identifier.
    -   `record_type: "scope:genesis"`
    -   Minimal `data` (≤1024B).

-   [ ] **Sign as Root for all 3** (Author/Usher/Quorum all satisfied by Root at bootstrap).
-   [ ] Append to ledger; persist CBOR artifact `00_root_scope.genesis.cbor`.

**Gate B:**

-   [ ] `rhex-verify 00_root_scope.genesis.cbor` passes; `previous_hash=null` only in this record.
-   [ ] Ledger cursor now set to hash H₀ (no duplicate `current_hash`).

---

## Phase 3 — R⬢: **Policy\:set (Root)** (minimal, enabling future adds)

-   [ ] Draft **first policy** that allows: key grants, authority grants, scope request/create, and transport attest; nothing else.
-   [ ] Build **policy\:set** per schema (root scope).
-   [ ] **Sign as Root for all 3**.
-   [ ] Append; save `01_root_policy.set.cbor`.

**Gate C:**

-   [ ] Policy passes `rhex-verify` & policy linter (no circular allowances, no wildcard writes beyond listed types).

---

## Phase 4 — Create **Core Usher Key** (for network ops)

-   [ ] `keytool generate -o genesis/keys/usher-core.key -p <pass>`
-   [ ] Note FP → `USHER_FP`.

**Gate D:**

-   [ ] `keytool show` matches `USHER_FP`.

---

## Phase 5 — R⬢: **key\:grant (Root → Usher/Core roles)**

-   [ ] Build **key\:grant** per schema (in **root** scope), assigning `USHER_FP` appropriate role(s) (e.g., `usher`, optional `author` if allowed).
-   [ ] **Sign as Root for all 3**.
-   [ ] Append; save `02_core_key.grant.cbor`.

**Gate E:**

-   [ ] Policy permits this grant; effective roles include `usher`.

---

## Phase 6 — R⬢: **authority\:grant (Usher Core)**

-   [ ] Build **authority\:grant** per schema in **root** scope with host, port, weight.
-   [ ] **Sign as Author with USHER key**.
-   [ ] **Sign as Root for Usher & Quorum** (tri-signature complete).
-   [ ] Append; save `03_core_authority.grant.cbor`.

**Gate F:**

-   [ ] Authority visible in scope table; transport can advertise.

---

## Phase 7 — Child Scope Lifecycle

> Remember: **scope\:create goes in the parent scope**; child’s own ledger starts at its **scope\:genesis**.

### 7.1 R⬢: **scope\:request (parent=root)**

-   [ ] Build **scope\:request** per schema (target child scope name/id).
-   [ ] **Sign with new child’s author key** (if pre-generated) as Author **and** with Usher+Quorum; or use USHER as Author if policy allows.
-   [ ] Append; save `10_child_scope.request.cbor`.

**Gate G:**

-   [ ] Parent policy allows request; no name collision in parent.

### 7.2 R⬢: **scope\:create (parent=root)**

-   [ ] Build **scope\:create** per schema for the child.
-   [ ] **Sign as new key as all 3**, with Root joining quorum (per your safety note).
-   [ ] Append; save `11_child_scope.create.cbor`.

**Gate H:**

-   [ ] Uniqueness enforced (parent refuses duplicate `scope:create`).

### 7.3 R⬢: **scope\:genesis (child)**

-   [ ] Switch ledger context to **child**.
-   [ ] Build child **scope\:genesis** per schema.
-   [ ] **Sign as Root for all 3** (bootstrap pattern) _or_ per child governance if different.
-   [ ] Append; save `12_child_scope.genesis.cbor`.

**Gate I:**

-   [ ] `previous_hash=null` only here in child; verifications pass.

---

## Phase 8 — Child Policies & Grants

### 8.1 R⬢: **policy\:set (child)**

-   [ ] Minimal child policy enabling expected ops (key/authority grants, record types for app domain).
-   [ ] **Sign as Root for all 3** (or child’s designated authority triad).
-   [ ] Append; save `13_child_policy.set.cbor`.

### 8.2 R⬢: **key\:grant (child)**

-   [ ] Grant operational keys for the child (authoring, ushering as needed).
-   [ ] **Sign as Root for all 3**.
-   [ ] Append; save `14_child_key.grant.cbor`.

### 8.3 R⬢: **authority\:grant (child)**

-   [ ] Declare child ushers (host/port/weight).
-   [ ] **Sign as Author using the newly granted key**.
-   [ ] **Sign as Root for Usher & Quorum**.
-   [ ] Append; save `15_child_authority.grant.cbor`.

**Gate J:**

-   [ ] Child scope now routable; K-of-N usher policy validated; transport tests pass.

---

## Verification & Anti-Footgun Checklist

-   [ ] **Canonicalization**: identical CBOR bytes before/after sign; `rhex-verify --canonical` passes.
-   [ ] **Hash uniqueness**: no duplicate `current_hash` (avoid the SQLite `1555` duplicate PK you hit). If collision: re-check CBOR bytes, timestamps, and signature order.
-   [ ] **Signature triad**: verify each record has the expected three roles. Dump who-signed-what and compare to policy.
-   [ ] **Record caps**: `data` ≤ 1024 B; total R⬢ size reasonable; quorum sig growth monitored.
-   [ ] **Parent/child rule**: `scope:create` only present in parent; child’s first record is `scope:genesis`.
-   [ ] **Timestamps**: monotonic non-decreasing `at`; align with GT notes for audit comments.
-   [ ] **Policy lints**: no wildcard writes prior to governance; explicit record allowlist.

---

## Smoke Tests (CLI Examples)

```bash
# 1) Keys
keytool generate -o genesis/keys/root.key -p "$PASS"
keytool generate -o genesis/keys/usher-core.key -p "$PASS"

# 2) Root genesis
rhex-pack scope:genesis \
  --scope root \
  --previous null \
  --data @root_genesis.json \
  | rhex-sign --author genesis/keys/root.key \
  | rhex-sign --usher  genesis/keys/root.key \
  | rhex-sign --quorum genesis/keys/root.key \
  | rhex-append --ledger ~/.trust/core.ledger \
  | tee genesis/rhex/00_root_scope.genesis.cbor

# 3) Minimal policy
rhex-pack policy:set --scope root --data @root_policy.json \
  | rhex-sign --author genesis/keys/root.key \
  | rhex-sign --usher  genesis/keys/root.key \
  | rhex-sign --quorum genesis/keys/root.key \
  | rhex-append --ledger ~/.trust/core.ledger \
  | tee genesis/rhex/01_root_policy.set.cbor

# 4) Grant usher key & authority
rhex-pack key:grant --scope root --data @usher_grant.json \
  | rhex-trisign root,key=root,key=root \
  | rhex-append --ledger ~/.trust/core.ledger \
  | tee genesis/rhex/02_core_key.grant.cbor

rhex-pack authority:grant --scope root --data @usher_authority.json \
  | rhex-sign --author genesis/keys/usher-core.key \
  | rhex-sign --usher  genesis/keys/root.key \
  | rhex-sign --quorum genesis/keys/root.key \
  | rhex-append --ledger ~/.trust/core.ledger \
  | tee genesis/rhex/03_core_authority.grant.cbor
```

---

## Quick Order (TL;DR)

1. Root key → 2) Root `scope:genesis` → 3) Root minimal `policy:set` → 4) Core usher key → 5) `key:grant` (usher role) → 6) `authority:grant` → 7) **Child** `scope:request` (parent=root) → 8) **Child** `scope:create` (parent=root) → 9) **Child** `scope:genesis` → 10) **Child** `policy:set` → 11) **Child** `key:grant` → 12) **Child** `authority:grant`.

---

## Sticky Notes (You-specific gotchas)

-   SQLite PK duplicate (`UNIQUE constraint failed: rhex.current_hash`): almost always canonicalization or accidental re-pack with same fields; diff CBOR bytes.
-   Keep a `manifest.json` in `/genesis/rhex/` listing every artifact, its `current_hash`, and signer FPS for audit.
-   If running K-of-N ushers: add a **transport test** that picks K from weights, then round-trips a ping R⬢ and validates quorum formation.

---

## “Done” Definition (Root is live when…)

-   You can fetch `policy:current` from Root via Core Usher.
-   A new child scope can be requested, created (in parent), and born (child genesis) without manual overrides.
-   `rhex-verify --full-ledger ~/.trust/core.ledger` returns ✅ and transport gossip announces the Core Usher.
