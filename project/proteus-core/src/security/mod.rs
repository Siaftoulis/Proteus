//! Sovereign Security Architecture & Cryptographic Integrity for Proteus BOS.
//! Implements hardware-sealed key vaults, monotonic time locks, and anti-rollback guards.

pub mod hardware_id;
pub mod time_lock;
pub mod vault;

pub use hardware_id::HardwareIdentity;
pub use time_lock::{
    TimeIntegrityGuard, TimeIntegrityRecord, TimeLockError, TimeLockStatus, TimeUnlockToken,
    DEFAULT_TOLERANCE_SECS,
};
pub use vault::{
    MasterSession, OperationalSession, VaultError, VaultManager, VaultStatus,
};

