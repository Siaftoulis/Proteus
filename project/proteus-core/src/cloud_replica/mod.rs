//! Sovereign Multi-Store Encrypted Cloud Replica & Disaster Recovery Engine (Phase 14).
//! Provides zero-knowledge AES-256-GCM chunked snapshot encryption, Argon2id key derivation,
//! and automated disaster recovery restore verification for multi-store retail resilience.
//! Strict Rule 1 (100% Original Codebase), Rule 5 (Zero Mock Data), Rule 6 (Compliance & Security).

pub mod snapshot;
pub mod manifest;
pub mod uploader;
pub mod restore;

pub use snapshot::{
    capture_sqlite_snapshot_bytes, decrypt_snapshot_bundle, derive_replica_key,
    encrypt_snapshot_bytes, ChunkedSnapshotBundle, EncryptedChunk, ReplicaError,
    DEFAULT_CHUNK_SIZE_BYTES,
};
pub use manifest::{
    compute_bundle_payload_hash, compute_manifest_seal, create_signed_manifest,
    get_latest_manifest, init_replica_ledger_schema, record_and_verify_manifest,
    verify_manifest_integrity, ReplicaManifestHeader,
};
pub use uploader::{
    build_s3_sigv4_headers, dispatch_upload_with_retry, enqueue_bundle_for_upload,
    fetch_pending_upload_tasks, hmac_sha256, init_replica_outbox_schema,
    update_upload_task_status, S3ReplicaConfig, UploadBatchResult, UploadStatus, UploadTask,
};
pub use restore::{
    execute_dry_run_verification, init_dr_audit_schema, list_dr_audit_logs, log_dr_event,
    restore_snapshot_to_file, DisasterRecoveryResult, DrAuditRecord, DryRunReport,
    SQLITE_MAGIC_HEADER,
};
