// SPDX-License-Identifier: MIT

use soroban_sdk::{contracttype, Address, Bytes, BytesN, String, Vec};

// ── Merchant ──────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MerchantCategory {
    Retail,
    Food,
    Services,
    Digital,
    Other,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Merchant {
    pub address: Address,
    pub name: String,
    pub description: String,
    pub contact_info: String,
    pub category: MerchantCategory,
    pub active: bool,
    pub registered_at: u64,
    pub signing_public_key: Option<BytesN<32>>,
}

// ── Payment ───────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PaymentStatus {
    Completed,
    PartiallyRefunded,
    FullyRefunded,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaymentOrder {
    pub order_id: Bytes,
    pub merchant_address: Address,
    pub payer: Address,
    pub token: Address,
    pub amount: i128,
    pub description: String,
    /// Unix timestamp (seconds) when the order expires. A value of `0`
    /// is treated as "never expires" (an order that does not expire).
    /// This special-case is relied upon by existing integrations and is
    /// intentionally accepted by the contract.
    pub expires_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaymentRecord {
    pub order_id: Bytes,
    pub merchant_address: Address,
    pub payer: Address,
    pub token: Address,
    pub amount: i128,
    pub refunded_amount: i128,
    pub pending_refund_amount: i128,
    pub status: PaymentStatus,
    pub paid_at: u64,
    pub description: String,
}

// ── Refund ────────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RefundStatus {
    Pending,
    Approved,
    Rejected,
    Completed,
    /// Payer has escalated a merchant-rejected refund for admin resolution.
    Disputed,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RefundRecord {
    pub refund_id: Bytes,
    pub order_id: Bytes,
    pub amount: i128,
    pub reason: String,
    pub status: RefundStatus,
    pub initiated_by: Address,
    pub initiated_at: u64,
    /// Latest ledger timestamp at which an admin may resolve a dispute.
    pub dispute_deadline: u64,
    /// Set when the payer disputes a merchant rejection. Empty string if not disputed.
    pub dispute_reason: String,
}

// ── Multisig ──────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultisigPayment {
    pub payment_id: Bytes,
    pub order: PaymentOrder,
    pub required_signers: Vec<Address>,
    pub signatures: Vec<Address>,
    pub executed: bool,
    pub expires_at: u64,
    pub created_at: u64,
}

// ── Query helpers ─────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SortField {
    Date,
    Amount, // .
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StatusFilter {
    Any,
    Completed,
    PartiallyRefunded,
    FullyRefunded,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaymentFilter {
    pub date_start: Option<u64>,
    pub date_end: Option<u64>,
    pub amount_min: Option<i128>,
    pub amount_max: Option<i128>,
    /// Filter by one or more token contract addresses. `None` matches all tokens.
    /// An empty list also matches all tokens (treated as no filter).
    pub tokens: Option<Vec<Address>>,
    pub status: StatusFilter,
}

/// A single page of payment records returned by history query functions.
///
/// # Cursor format
///
/// `next_cursor` is the raw `order_id` bytes of the **last record on this
/// page**.  It is intentionally opaque to on-chain callers — the contract
/// only needs to find the cursor value inside its sorted ID list to resume
/// iteration.
///
/// For off-chain clients the bytes are the exact bytes of the `order_id` that
/// was stored when the payment was processed.  The encoding depends on how the
/// caller originally constructed the `order_id`:
///
/// - If the caller used a UTF-8 string (e.g. `"ORDER_001"`), the bytes are
///   the UTF-8 representation of that string.
/// - If the caller used a hash or other binary ID, the bytes are the raw
///   binary without any additional encoding.
///
/// Off-chain clients should treat `next_cursor` as an **opaque blob** and
/// round-trip it verbatim.  Do **not** attempt to decode or construct cursors
/// from scratch — the internal sort order may change in a future version and
/// the cursor value would silently point to a different position.  See
/// [ADR-0003](../../../../docs/adr/0003-pagination-design.md) and
/// [ADR-0005](../../../../docs/adr/0005-cursor-format.md) for the full design
/// rationale and migration guidance.
///
/// # Pagination flow
///
/// ```text
/// first call:  cursor = None   → returns page 1, next_cursor = Some(<id>)
/// second call: cursor = Some(<id>) → returns page 2, next_cursor = Some(<id>)
/// ...
/// last page:   cursor = Some(<id>) → returns page N, next_cursor = None
/// ```
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaymentPage {
    /// The payment records on this page (at most `limit`, capped at 100).
    pub records: Vec<PaymentRecord>,
    /// Opaque pagination cursor pointing to the last record on the page.
    ///
    /// Current format: raw `order_id` bytes of the last record. Callers that
    /// transport the cursor over textual channels (CLI, HTTP) should encode
    /// it (for example as base64). The contract treats the cursor as an opaque
    /// `Bytes` value and will start the next page after the matching `order_id`.
    ///
    /// NOTE: changing this format is a breaking change. Any future change
    /// should use a versioned encoding and include a migration note in an ADR.
    pub next_cursor: Option<Bytes>,
    /// Total number of matching records across all pages (before truncation to
    /// `limit`).  Useful for progress indicators but not for computing page
    /// count without iterating (because filtered totals are not pre-computed).
    pub total: u32,
}

// ── Stats ─────────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlobalStats {
    pub total_payments: u64,
    pub total_volume: i128,
    pub total_refunds: u64,
    pub total_refund_volume: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MerchantStats {
    pub merchant_address: Address,
    pub total_payments: u64,
    pub total_volume: i128,
    pub total_refunds: u64,
    pub total_refund_volume: i128,
}

// ── Admin ─────────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminConfig {
    pub admins: Vec<Address>,
    pub threshold: u32,
}

// ── Subscription ──────────────────────────────────────────────────────────────

/// Defines the recurring payment terms for a subscription.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionPlan {
    /// Payment interval in seconds (e.g. 2_592_000 for 30 days).
    pub interval: u64,
    /// Amount charged per interval, in the smallest token unit.
    pub amount: i128,
    /// Token contract address used for recurring charges.
    pub token: Address,
}

/// Lifecycle state of a subscription.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SubscriptionStatus {
    Active,
    Cancelled,
}

/// Persisted state for a single payer–merchant subscription.
///
/// # Off-chain scheduler requirement
/// Soroban contracts cannot autonomously schedule execution. An off-chain
/// scheduler service MUST call `process_subscription_payment` at each interval
/// boundary. The contract enforces correctness (idempotency, interval guard,
/// status checks) but relies on the scheduler for timely invocation.
///
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionState {
    /// Unique subscription identifier (caller-supplied).
    pub subscription_id: Bytes,
    pub payer: Address,
    pub merchant: Address,
    pub plan: SubscriptionPlan,
    pub status: SubscriptionStatus,
    /// Ledger timestamp when the subscription was created.
    pub created_at: u64,
    /// Ledger timestamp of the most recent successful payment (0 if none yet).
    pub last_charged_at: u64,
}

// ── Storage keys ──────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    AdminConfig,
    ContractVersion,
    Merchant(Address),
    Payment(Bytes),
    /// Flat payment index list per merchant.
    MerchantPayments(Address),
    /// Flat payment index list per payer.
    PayerPayments(Address),
    /// Global flat payment index.
    GlobalPaymentIndex,
    Refund(Bytes),
    Multisig(Bytes),
    CleanupPeriod,
    DefaultMultisigExpiry,
    GlobalStats,
    AllRefunds,
    WhitelistEnabled,
    Whitelist(Address),
    OrderRefundCount(Bytes),
    ArchivedPayment(Bytes),
    TokenAllowlistEnabled,
    AllowedToken(Address),
    Subscription(Bytes),
    MerchantStats(Address),
    // Connection pooling storage keys
    PoolConfig,
    PoolStats,
    PoolConnection(u32),
}
