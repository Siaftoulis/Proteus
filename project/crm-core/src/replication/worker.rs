//! Resilient Background Outbox Worker & Transport Engine.
//! Provides priority-based batch polling, Full Jitter Exponential Backoff,
//! and Circuit Breaker network suspension to protect local SQLite WAL mode.

use super::{fetch_pending_outbox, mark_outbox_status, OutboxRecord, SyncStatus};
use rand::Rng;
use rusqlite::Connection;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub max_retries: u32,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            base_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(60),
            max_retries: 10,
        }
    }
}

impl RetryPolicy {
    /// Computes full jitter exponential backoff:
    /// Sleep Interval = random(0, min(max_delay, base_delay * 2^attempt))
    pub fn calculate_backoff(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::ZERO;
        }
        let exp = 2u64.saturating_pow(attempt.min(10));
        let max_calculated = self.base_delay.as_millis() as u64 * exp;
        let ceiling = max_calculated.min(self.max_delay.as_millis() as u64);

        if ceiling == 0 {
            return Duration::ZERO;
        }
        let mut rng = rand::thread_rng();
        let jittered_millis = rng.gen_range(0..=ceiling);
        Duration::from_millis(jittered_millis)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    pub failure_threshold: u32,
    pub suspension_duration: Duration,
    state: CircuitState,
    consecutive_failures: u32,
    suspended_until: Option<Instant>,
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new(3, Duration::from_secs(30))
    }
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, suspension_duration: Duration) -> Self {
        Self {
            failure_threshold,
            suspension_duration,
            state: CircuitState::Closed,
            consecutive_failures: 0,
            suspended_until: None,
        }
    }

    pub fn state(&self) -> CircuitState {
        self.state
    }

    /// Checks if a request is permitted. If in Open state and duration has expired,
    /// transitions to HalfOpen to allow a single probe.
    pub fn can_execute(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(until) = self.suspended_until {
                    if Instant::now() >= until {
                        self.state = CircuitState::HalfOpen;
                        self.suspended_until = None;
                        true
                    } else {
                        false
                    }
                } else {
                    true
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    pub fn on_success(&mut self) {
        self.consecutive_failures = 0;
        self.state = CircuitState::Closed;
        self.suspended_until = None;
    }

    pub fn on_failure(&mut self) {
        self.consecutive_failures += 1;
        if self.consecutive_failures >= self.failure_threshold || self.state == CircuitState::HalfOpen {
            self.state = CircuitState::Open;
            self.suspended_until = Some(Instant::now() + self.suspension_duration);
        }
    }

    pub fn reset(&mut self) {
        self.consecutive_failures = 0;
        self.state = CircuitState::Closed;
        self.suspended_until = None;
    }
}

pub trait RemoteTransport: Send + Sync {
    fn send(&self, record: &OutboxRecord) -> Result<(), String>;
    fn ping(&self) -> Result<(), String>;
}

pub struct OutboxWorker {
    pub retry_policy: RetryPolicy,
    pub circuit_breaker: CircuitBreaker,
    pub batch_size: usize,
}

impl Default for OutboxWorker {
    fn default() -> Self {
        Self {
            retry_policy: RetryPolicy::default(),
            circuit_breaker: CircuitBreaker::default(),
            batch_size: 25,
        }
    }
}

impl OutboxWorker {
    pub fn new(retry_policy: RetryPolicy, circuit_breaker: CircuitBreaker, batch_size: usize) -> Self {
        Self {
            retry_policy,
            circuit_breaker,
            batch_size,
        }
    }

    /// Performs one sync cycle step. If the circuit is Open, zero SQLite queries are made.
    pub fn step(&mut self, conn: &Connection, transport: &dyn RemoteTransport) -> Result<usize, String> {
        if !self.circuit_breaker.can_execute() {
            return Ok(0);
        }

        // Fetch prioritized records: Critical -> High -> Normal -> Low
        let pending = fetch_pending_outbox(conn, self.batch_size)?;
        if pending.is_empty() {
            return Ok(0);
        }

        let mut pushed = 0;
        for record in &pending {
            match transport.send(record) {
                Ok(_) => {
                    let _ = mark_outbox_status(conn, &record.id, SyncStatus::Synced);
                    self.circuit_breaker.on_success();
                    pushed += 1;
                }
                Err(e) => {
                    let _ = mark_outbox_status(conn, &record.id, SyncStatus::Failed);
                    self.circuit_breaker.on_failure();
                    return Err(format!("Transport failure on record {}: {}", record.id, e));
                }
            }
        }

        Ok(pushed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::replication::{enqueue_outbox_with_priority, init_outbox_schema, ChangeOp, DataPriority};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_retry_policy_bounds() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.calculate_backoff(0), Duration::ZERO);

        for attempt in 1..=10 {
            let backoff = policy.calculate_backoff(attempt);
            let max_possible = policy.base_delay * 2u32.pow(attempt.min(10));
            let ceiling = max_possible.min(policy.max_delay);
            assert!(backoff <= ceiling);
        }
    }

    #[test]
    fn test_circuit_breaker_transitions() {
        let mut cb = CircuitBreaker::new(3, Duration::from_millis(100));
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.can_execute());

        cb.on_failure();
        cb.on_failure();
        assert_eq!(cb.state(), CircuitState::Closed);

        cb.on_failure(); // 3rd failure -> Open
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.can_execute());

        // Wait for suspension to expire
        std::thread::sleep(Duration::from_millis(110));
        assert!(cb.can_execute());
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        // Success resets to Closed
        cb.on_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    struct MockTransport {
        should_fail: Arc<AtomicBool>,
    }

    impl RemoteTransport for MockTransport {
        fn send(&self, _record: &OutboxRecord) -> Result<(), String> {
            if self.should_fail.load(Ordering::Relaxed) {
                Err("Connection refused".into())
            } else {
                Ok(())
            }
        }
        fn ping(&self) -> Result<(), String> {
            if self.should_fail.load(Ordering::Relaxed) {
                Err("Ping failed".into())
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn test_outbox_worker_priority_dispatch() {
        let conn = Connection::open_in_memory().unwrap();
        init_outbox_schema(&conn).unwrap();

        // Enqueue Low priority first, then Critical
        enqueue_outbox_with_priority(&conn, "logs", "LOG-1", ChangeOp::Insert, "{}", DataPriority::Low).unwrap();
        enqueue_outbox_with_priority(&conn, "payments", "PAY-1", ChangeOp::Insert, "{}", DataPriority::Critical).unwrap();

        let transport = MockTransport {
            should_fail: Arc::new(AtomicBool::new(false)),
        };

        let mut worker = OutboxWorker::default();
        let pushed = worker.step(&conn, &transport).unwrap();
        assert_eq!(pushed, 2);

        // All should be synced
        let pending = fetch_pending_outbox(&conn, 10).unwrap();
        assert_eq!(pending.len(), 0);
    }
}
