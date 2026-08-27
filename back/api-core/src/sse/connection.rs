use std::collections::HashSet;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock, Weak};
use std::time::Duration;

use dashmap::DashMap;
use futures::StreamExt;
use log::{error, info, warn};
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::sse::envelope::SseEnvelope;
use crate::sse::traits::{ConnectionMetadata, SseEventWithMetadata, SseTarget};

const RECONNECT_DELAY: Duration = Duration::from_secs(5);
const CLEANUP_INTERVAL: Duration = Duration::from_secs(60);

struct Connection<M: ConnectionMetadata> {
    id: Uuid,
    metadata: RwLock<M>,
    sender: broadcast::Sender<String>,
}

pub struct SseConnectionManager<E, M, T>
where
    E: SseEventWithMetadata<M>,
    M: ConnectionMetadata,
    T: SseTarget<M>,
{
    redis_url: String,
    channel: String,
    connections: DashMap<Uuid, Arc<Connection<M>>>,
    by_routing_key: DashMap<String, Vec<Weak<Connection<M>>>>,
    _event: PhantomData<E>,
    _target: PhantomData<T>,
}

impl<E, M, T> SseConnectionManager<E, M, T>
where
    E: SseEventWithMetadata<M>,
    M: ConnectionMetadata,
    T: SseTarget<M>,
{
    pub fn new(
        redis_url: impl Into<String>,
        channel: impl Into<String>,
    ) -> Arc<Self> {
        let manager = Arc::new(Self {
            redis_url: redis_url.into(),
            channel: channel.into(),
            connections: DashMap::new(),
            by_routing_key: DashMap::new(),
            _event: PhantomData,
            _target: PhantomData,
        });

        let subscriber = Arc::downgrade(&manager);
        tokio::spawn(run_subscriber(
            manager.redis_url.clone(),
            manager.channel.clone(),
            subscriber,
        ));

        let cleanup = Arc::downgrade(&manager);
        tokio::spawn(run_cleanup(cleanup));

        manager
    }

    pub fn connect(
        self: &Arc<Self>,
        metadata: M,
    ) -> (broadcast::Receiver<String>, Uuid, ConnectionGuard<E, M, T>) {
        let id = Uuid::new_v4();
        let (tx, rx) = broadcast::channel::<String>(128);

        let conn = Arc::new(Connection {
            id,
            metadata: RwLock::new(metadata.clone()),
            sender: tx,
        });

        for key in metadata.routing_keys() {
            self.by_routing_key
                .entry(key)
                .or_default()
                .push(Arc::downgrade(&conn));
        }

        self.connections.insert(id, Arc::clone(&conn));

        let guard = ConnectionGuard {
            connection_id: id,
            manager: Arc::downgrade(self),
        };

        (rx, id, guard)
    }

    pub fn update_metadata(&self, connection_id: Uuid, new_metadata: M) {
        let Some(conn) = self.connections.get(&connection_id) else {
            return;
        };
        self.reindex_connection(&conn, new_metadata);
    }

    pub fn disconnect(&self, connection_id: Uuid) {
        let Some((_, conn)) = self.connections.remove(&connection_id) else {
            return;
        };

        let metadata = conn.metadata.read().unwrap().clone();
        for key in metadata.routing_keys() {
            if let Some(mut bucket) = self.by_routing_key.get_mut(&key) {
                bucket.retain(|weak| match weak.upgrade() {
                    Some(c) => c.id != connection_id,
                    None => false,
                });
            }
        }
    }

    async fn dispatch(self: &Arc<Self>, payload: &str) {
        let envelope: SseEnvelope<E, T> = match serde_json::from_str(payload) {
            Ok(e) => e,
            Err(e) => {
                warn!("[SseConnectionManager] Failed to parse envelope: {}", e);
                return;
            }
        };

        let event_json = match serde_json::to_string(&envelope.event) {
            Ok(s) => s,
            Err(e) => {
                error!("[SseConnectionManager] Failed to serialize event: {}", e);
                return;
            }
        };

        let new_metadata = envelope.event.metadata_update();
        let keys = envelope.target.routing_keys();

        if keys.is_empty() {
            for entry in self.connections.iter() {
                let conn = entry.value();
                if envelope.target.matches(&*conn.metadata.read().unwrap()) {
                    self.send_to_connection(conn, &event_json, new_metadata.as_ref());
                }
            }
        } else {
            let primary = &keys[0];
            if let Some(bucket) = self.by_routing_key.get(primary) {
                for weak in bucket.iter() {
                    if let Some(conn) = weak.upgrade() {
                        if envelope.target.matches(&*conn.metadata.read().unwrap()) {
                            self.send_to_connection(&conn, &event_json, new_metadata.as_ref());
                        }
                    }
                }
            }
        }
    }

    fn send_to_connection(
        &self,
        conn: &Arc<Connection<M>>,
        payload: &str,
        new_metadata: Option<&M>,
    ) {
        if let Some(meta) = new_metadata {
            self.reindex_connection(conn, meta.clone());
        }
        let _ = conn.sender.send(payload.to_string());
    }

    fn reindex_connection(&self, conn: &Arc<Connection<M>>, new_metadata: M) {
        let old_keys: HashSet<String> = conn
            .metadata
            .read()
            .unwrap()
            .routing_keys()
            .into_iter()
            .collect();
        let new_keys: HashSet<String> = new_metadata.routing_keys().into_iter().collect();

        for key in old_keys.difference(&new_keys) {
            if let Some(mut bucket) = self.by_routing_key.get_mut(key) {
                bucket.retain(|weak| match weak.upgrade() {
                    Some(c) => c.id != conn.id,
                    None => false,
                });
            }
        }

        for key in new_keys.difference(&old_keys) {
            self.by_routing_key
                .entry(key.clone())
                .or_default()
                .push(Arc::downgrade(conn));
        }

        *conn.metadata.write().unwrap() = new_metadata;
    }

    fn cleanup_dead_refs(&self) {
        for mut bucket in self.by_routing_key.iter_mut() {
            bucket.retain(|weak| weak.upgrade().is_some());
        }
    }
}

async fn run_subscriber<E, M, T>(
    redis_url: String,
    channel: String,
    manager: Weak<SseConnectionManager<E, M, T>>,
) where
    E: SseEventWithMetadata<M>,
    M: ConnectionMetadata,
    T: SseTarget<M>,
{
    loop {
        info!(
            "[SseConnectionManager] Connecting Redis subscriber to {}",
            redis_url
        );
        match try_subscribe(&redis_url, &channel, &manager).await {
            Ok(()) => {
                warn!("[SseConnectionManager] Subscriber exited, reconnecting...");
            }
            Err(e) => {
                error!(
                    "[SseConnectionManager] Subscriber error: {}, reconnecting...",
                    e
                );
            }
        }
        tokio::time::sleep(RECONNECT_DELAY).await;
    }
}

async fn try_subscribe<E, M, T>(
    redis_url: &str,
    channel: &str,
    manager: &Weak<SseConnectionManager<E, M, T>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    E: SseEventWithMetadata<M>,
    M: ConnectionMetadata,
    T: SseTarget<M>,
{
    let client = redis::Client::open(redis_url)?;
    let mut pubsub = client.get_async_pubsub().await?;
    pubsub.subscribe(channel).await?;

    info!(
        "[SseConnectionManager] Subscribed to Redis channel '{}'",
        channel
    );

    let mut stream = pubsub.on_message();
    while let Some(msg) = stream.next().await {
        let payload = msg.get_payload::<String>()?;
        if let Some(mgr) = manager.upgrade() {
            mgr.dispatch(&payload).await;
        } else {
            break;
        }
    }

    Ok(())
}

async fn run_cleanup<E, M, T>(manager: Weak<SseConnectionManager<E, M, T>>)
where
    E: SseEventWithMetadata<M>,
    M: ConnectionMetadata,
    T: SseTarget<M>,
{
    let mut interval = tokio::time::interval(CLEANUP_INTERVAL);
    loop {
        interval.tick().await;
        if let Some(mgr) = manager.upgrade() {
            mgr.cleanup_dead_refs();
        } else {
            break;
        }
    }
}

pub struct ConnectionGuard<E, M, T>
where
    E: SseEventWithMetadata<M>,
    M: ConnectionMetadata,
    T: SseTarget<M>,
{
    connection_id: Uuid,
    manager: Weak<SseConnectionManager<E, M, T>>,
}

impl<E, M, T> ConnectionGuard<E, M, T>
where
    E: SseEventWithMetadata<M>,
    M: ConnectionMetadata,
    T: SseTarget<M>,
{
    pub fn new(connection_id: Uuid, manager: Weak<SseConnectionManager<E, M, T>>) -> Self {
        Self {
            connection_id,
            manager,
        }
    }
}

impl<E, M, T> Drop for ConnectionGuard<E, M, T>
where
    E: SseEventWithMetadata<M>,
    M: ConnectionMetadata,
    T: SseTarget<M>,
{
    fn drop(&mut self) {
        if let Some(mgr) = self.manager.upgrade() {
            mgr.disconnect(self.connection_id);
        }
    }
}
