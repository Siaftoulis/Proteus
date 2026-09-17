//! In-Memory Event Bus & Cross-Subsystem Messaging Broker for Proteus BOS.
//! Decouples core domains (Tickets, Audits, Appointments, Rules Engine, Integrations)
//! through thread-safe publish/subscribe event routing.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMessage {
    pub id: String,
    pub topic: String,
    pub payload: Value,
    pub timestamp_ms: i64,
}

impl EventMessage {
    pub fn new(topic: &str, payload: Value) -> Self {
        Self {
            id: Uuid::now_v7().to_string(),
            topic: topic.to_string(),
            payload,
            timestamp_ms: Utc::now().timestamp_millis(),
        }
    }
}

type EventHandler = Arc<dyn Fn(&EventMessage) + Send + Sync + 'static>;

#[derive(Clone, Default)]
pub struct EventBus {
    subscribers: Arc<Mutex<HashMap<String, Vec<EventHandler>>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Subscribe to a specific event topic with a callback handler.
    pub fn subscribe<F>(&self, topic: &str, handler: F)
    where
        F: Fn(&EventMessage) + Send + Sync + 'static,
    {
        let mut subs = self.subscribers.lock().unwrap();
        subs.entry(topic.to_string())
            .or_default()
            .push(Arc::new(handler));
    }

    /// Publish an event message to all subscribers of the topic.
    /// Returns the number of handlers invoked.
    pub fn publish(&self, topic: &str, payload: Value) -> usize {
        let msg = EventMessage::new(topic, payload);
        let handlers = {
            let subs = self.subscribers.lock().unwrap();
            subs.get(topic).cloned().unwrap_or_default()
        };

        let count = handlers.len();
        for handler in handlers {
            handler(&msg);
        }
        count
    }

    /// Number of active subscribers for a given topic.
    pub fn subscriber_count(&self, topic: &str) -> usize {
        let subs = self.subscribers.lock().unwrap();
        subs.get(topic).map(|v| v.len()).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_publish_subscribe_flow() {
        let bus = EventBus::new();
        let received_counter = Arc::new(AtomicUsize::new(0));

        let counter_clone = Arc::clone(&received_counter);
        bus.subscribe("ticket.created", move |msg| {
            assert_eq!(msg.topic, "ticket.created");
            assert_eq!(msg.payload["customer"], "Giannis");
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        let dispatched = bus.publish("ticket.created", json!({ "customer": "Giannis" }));
        assert_eq!(dispatched, 1);
        assert_eq!(received_counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_multiple_subscribers_on_topic() {
        let bus = EventBus::new();
        let counter1 = Arc::new(AtomicUsize::new(0));
        let counter2 = Arc::new(AtomicUsize::new(0));

        let c1 = Arc::clone(&counter1);
        bus.subscribe("inventory.low_stock", move |_| {
            c1.fetch_add(1, Ordering::SeqCst);
        });

        let c2 = Arc::clone(&counter2);
        bus.subscribe("inventory.low_stock", move |_| {
            c2.fetch_add(1, Ordering::SeqCst);
        });

        assert_eq!(bus.subscriber_count("inventory.low_stock"), 2);
        let dispatched = bus.publish("inventory.low_stock", json!({ "sku": "TECH-01" }));
        assert_eq!(dispatched, 2);
        assert_eq!(counter1.load(Ordering::SeqCst), 1);
        assert_eq!(counter2.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_empty_topic_publish_returns_zero() {
        let bus = EventBus::new();
        let dispatched = bus.publish("unsubscribed.topic", json!({}));
        assert_eq!(dispatched, 0);
    }
}
