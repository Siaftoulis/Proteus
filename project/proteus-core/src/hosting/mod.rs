//! Multi-Tenant Cloud Hosting, Environment Isolation & Storage Quota Infrastructure.

pub mod isolation;
pub mod billing;
pub mod migration;
pub mod canary;

pub use isolation::{
    DEFAULT_TENANT_QUOTA_BYTES, StorageQuotaError, TenantContainer, TenantIsolationManager,
    TenantStorageUsage,
};
pub use billing::{
    calculate_vps_quote, init_vps_billing_schema, record_vps_invoice, BillingLineItem,
    VpsBillingQuote, VpsProviderRates, VpsResourceSpec, DEFAULT_VAT_PERCENT,
    DEFAULT_VPS_MARKUP_PERCENT,
};
pub use migration::{
    MigrationDirection, MigrationError, MigrationManager, MigrationManifest,
};
pub use canary::{
    CanaryHealthReport, CanaryMigratorAgent, CanaryStage, SchemaHealthStatus,
};
