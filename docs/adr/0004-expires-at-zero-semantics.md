# ADR-0004: `PaymentOrder.expires_at == 0` Means "No Expiry"

**Status:** Accepted  
**Date:** 2026-07-08  
**Deciders:** Pulsar Contributors  
**Closes:** #232 (SC-054)

## Context

`PaymentOrder` contains an `expires_at: u64` field intended to let merchants
set a deadline on an order.  During initial implementation, `0` was chosen as
a sentinel meaning "no expiry" — the contract skips the expiry check when
`expires_at == 0`.

This behaviour was never documented in the struct definition, the README, or
an ADR.  Integrators reading only the struct signature had no way to know that
`0` is special, and could accidentally create orders that never expire when
they intended to use a real timestamp.

### Options considered

1. **Reject `expires_at == 0`** — treat zero as invalid input; callers must
   always supply a future timestamp.
2. **Keep zero as "no expiry", add documentation** — document the sentinel
   clearly in code comments, the README, and this ADR.
3. **Use `Option<u64>`** — make the field optional at the type level;
   `None` means no expiry, `Some(ts)` means a deadline.

## Decision

**Option 2 — keep `expires_at == 0` as the "no expiry" sentinel and document
it everywhere.**

Rationale:

- Option 1 is a breaking change.  Existing deployed contracts and integrations
  that already send `expires_at = 0` would break immediately.
- Option 3 requires a type change to `PaymentOrder`.  Because `PaymentOrder`
  is a `#[contracttype]` serialised into Soroban XDR, changing the type would
  break binary compatibility with all stored and in-flight orders (another
  breaking change).
- Option 2 preserves backward compatibility while eliminating ambiguity through
  documentation.

The decision not to reject `expires_at == 0` is further justified by real
use-cases where a hard deadline genuinely is not needed: standing orders,
subscription billing triggers, and test/sandbox environments.  Replay
protection is provided independently by the unique `order_id` — once an order
is processed its ID is recorded on-chain, and any subsequent use of the same
`order_id` fails with `PaymentAlreadyExists`.

## Consequences

### Positive
- No breaking change to the on-chain interface or existing integrations.
- The semantics are now unambiguous: `0` = no expiry, any other value = Unix
  deadline in seconds.
- Integrators reading the struct doc comment, the README table, or this ADR
  will immediately understand the field.

### Negative
- The `u64` type does not communicate intent by itself; callers must read the
  documentation.  A future major version of the contract could migrate to
  `Option<u64>` with an explicit migration path.

### Neutral
- If a future team decides to reject `0` or change the sentinel, they must
  update this ADR (mark it Superseded), bump the contract version, and add a
  migration note in the README.

## Migration note

If this behaviour is ever changed (e.g. `0` becomes invalid), the change must:

1. Supersede this ADR with a new one explaining the migration path.
2. Increment the contract version stored under `DataKey::ContractVersion`.
3. Add a migration note in `README.md` under a "Breaking Changes" section.
4. Provide a tooling/script path for integrators to detect affected orders.
