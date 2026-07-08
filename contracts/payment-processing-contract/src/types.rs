use soroban_sdk::{contracttype, Address, Bytes, BytesN, String, Vec};

// ── Subscription ──────────────────────────────────────────────────────────────

/// The billing interval for a subscription plan.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BillingInterval {
    /// Billed once every 7 days.
    Weekly,
    /// Billed once every 30 days.
    Monthly,
    /// Billed once every 365 days.
    Yearly,
}

/// A subscription plan created by a merchant.
///
/// Subscribers are tracked via the `MerchantSubscriptions(merchant_address)`
/// storage index.  Each individual subscription is stored under
/// `DataKey::Subscription(subscription_id)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionPlan {
    /// Globally unique identifier for this subscription plan.
    pub plan_id: Bytes,
    /// Address of the merchant who owns this plan.
    pub merchant_address: Address,
    /// Token contract used for billing.
    pub token: Address,
    /// Amount charged per billing cycle, in the token's smallest denomination.
    pub amount: i128,
    /// Human-readable plan name (max 64 bytes).
    pub name: String,
    /// Human-readable plan description (max 256 bytes).
    pub description: String,
    /// Billing cadence.
    pub interval: BillingInterval,
    /// Whether new subscribers can join this plan.
    pub active: bool,
    /// Unix timestamp (seconds) when the plan was created.
    pub created_at: u64,
}

/// Current lifecycle state of a subscriber's subscription.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SubscriptionStatus {
    /// Subscription is active and billing is ongoing.
    Active,
    /// Subscriber paused the subscription voluntarily.
    Paused,
    /// Subscription was cancelled and is no longer billable.
    Cancelled,
}

/// The state of a single subscriber enrolled in a `SubscriptionPlan`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionState {
    /// Globally unique identifier for this subscription instance.
    pub subscription_id: Bytes,
    /// The plan this subscription belongs to.
    pub plan_id: Bytes,
    /// Merchant who owns the plan.
    pub merchant_address: Address,
    /// Subscriber (payer) address.
    pub subscriber: Address,
    /// Unix timestamp (seconds) when the subscriber enrolled.
    pub subscribed_at: u64,
    /// Unix timestamp (seconds) of the next scheduled billing event.
    pub next_billing_at: u64,
    /// Cumulative number of successful billing cycles.
    pub billing_count: u64,
    /// Current lifecycle state.
    pub status: SubscriptionStatus,
}

/// Paginated response for subscription queries.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionPage {
    /// The subscriptions on this page.
    pub records: Vec<SubscriptionState>,
    /// Cursor pointing to the last record on this page.
    ///
    /// Pass this value as `cursor` in the next call to retrieve the following
    /// page.  `None` means this is the last page.
    pub next_cursor: Option<Bytes>,
    /// Total number of subscriptions in the index (before pagination).
    pub total: u32,
}

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
    Amount,
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
    pub token: Option<Address>,
    pub status: StatusFilter,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaymentPage {
    pub records: Vec<PaymentRecord>,
    pub next_cursor: Option<Bytes>,
    pub total: u32,
}

// ── Global stats ──────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlobalStats {
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

// ── Storage keys ──────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    ContractVersion,
    Merchant(Address),
    Payment(Bytes),
    MerchantPaymentChunk(Address, u32),
    MerchantPaymentCount(Address),
    PayerPaymentChunk(Address, u32),
    PayerPaymentCount(Address),
    Refund(Bytes),
    Multisig(Bytes),
    CleanupPeriod,
    DefaultMultisigExpiry,
    GlobalPaymentChunk(u32),
    GlobalPaymentCount,
    GlobalStats,
    AllPayments,
    AllRefunds,
    WhitelistEnabled,
    Whitelist(Address),
    /// Individual subscription plan keyed by plan ID.
    SubscriptionPlan(Bytes),
    /// Individual subscription state (subscriber + plan) keyed by subscription ID.
    Subscription(Bytes),
    /// Per-merchant index: `Vec<Bytes>` of subscription IDs for fast enumeration.
    ///
    /// Updated atomically on `subscribe` / `cancel_subscription`.
    /// TTL is extended on every read and write.
    MerchantSubscriptions(Address),
}
