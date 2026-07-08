# ADR-0005: Cursor Format for Payment History Pagination

**Status:** Accepted  
**Date:** 2026-07-08  
**Deciders:** Pulsar Contributors  
**Closes:** #230 (SC-052)  
**Supplements:** ADR-0003 (Cursor-Based Pagination)

## Context

`get_merchant_payment_history` and `get_payer_payment_history` return a
`PaymentPage` that includes a `next_cursor: Option<Bytes>` field.  Callers
pass this value back as `cursor` on the next call to retrieve the following
page.

ADR-0003 established *that* we use cursor-based pagination, but it did not
document *what the cursor bytes actually contain* or what guarantees callers
can rely on.  This gap caused two problems:

1. Off-chain integrators had no way to understand, validate, or display the
   cursor.  Some attempted to decode or construct cursors manually, which is
   fragile.
2. Any future change to the cursor's internal encoding would silently break
   existing integrations with no migration path.

## Decision

**The cursor is the raw `order_id` bytes of the last record on the page**,
taken directly from `PaymentRecord.order_id`.  No additional encoding,
versioning, or wrapping is applied.

This encoding is the simplest possible option and maps directly to the
existing storage model: the paginator maintains a sorted list of `order_id`
bytes and uses the cursor to find the resume point within that list.

### Why not base64 or version-prefixed encoding?

| Option | Pros | Cons |
|---|---|---|
| Raw `order_id` bytes (chosen) | No encoding overhead; trivial to implement; zero on-chain cost | Not human-readable; callers must not decode |
| Base64-wrapped bytes | Human-readable in JSON | Extra encoding/decoding on every page; no semantic gain on-chain |
| Version-prefixed `[version: u8, ...bytes]` | Future-proof | One extra byte per cursor; version byte is meaningless until a second encoding exists; premature abstraction |

A version-prefix adds complexity without benefit today.  If the encoding ever
needs to change, it should be done as part of a deliberate, breaking contract
upgrade (see Migration Note below) rather than through a hidden version field.

### Cursor stability guarantees

| Property | Guarantee |
|---|---|
| Same sort order across pages | ✅ Cursor reliably resumes at the correct position |
| Changed `sort_field` or `sort_order` | ❌ Cursor is invalid; restart from `cursor = None` |
| New payments added during iteration | ✅ Stable — only forward iteration; new appended payments appear on later pages |
| Payment archived (removed) during iteration | ⚠️ The archived record is simply skipped; no error is returned |
| Cursor from a different entity (e.g. payer cursor used for merchant query) | ❌ Undefined behaviour — the cursor may not exist in the target index |

## Consequences

### Positive
- Cursor format is now fully documented; off-chain integrators know exactly
  what the bytes mean.
- No change to the on-chain ABI or stored data — purely a documentation
  improvement.
- Establishes a clear migration contract for future changes.

### Negative
- The raw-bytes format is not human-readable.  Integrators who need to display
  a page indicator must maintain their own counter client-side.
- Only forward iteration is supported; there is no way to seek to an arbitrary
  page or iterate backward.

### Neutral
- The `total` field in `PaymentPage` reflects the count of records matching the
  filter *before* pagination.  It allows a progress indicator but does not
  enable direct page computation without full iteration.

## Migration note

If the cursor encoding is ever changed (e.g. adopting a version-prefixed or
base64-encoded format):

1. This ADR must be marked **Superseded** and a new ADR written that describes
   the new format and migration path.
2. The contract version (`DataKey::ContractVersion`) must be incremented.
3. A "Breaking Changes" section must be added to `README.md` documenting:
   - Which contract version introduced the change.
   - How to detect stale cursors (e.g. version byte mismatch).
   - Whether a migration tool / script is available.
4. Existing callers that cache cursors across a contract upgrade must flush
   their cursor caches and restart pagination from `cursor = None`.
