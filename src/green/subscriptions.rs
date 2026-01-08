use crate::ui::Message;
use iced::Task;
use iced::futures::StreamExt;
use serde::Deserialize;
use std::sync::Arc;
use surrealdb::engine::local::Db;
use surrealdb::sql::Uuid;
use surrealdb::Surreal;
use tokio::sync::{Mutex, mpsc};
use tokio::task::JoinHandle;

#[derive(Debug, Clone)]
pub enum LiveUpdate {
    Focus,
    Siblings,
    Children,
    Ancestors,
}

struct LiveSubscription {
    live_id: Uuid,
    task: JoinHandle<()>,
}

struct SubscriptionInner {
    db: Arc<Surreal<Db>>,
    updates_tx: mpsc::UnboundedSender<LiveUpdate>,
    active: Vec<LiveSubscription>,
}

#[derive(Clone)]
pub struct SubscriptionManager {
    inner: Arc<Mutex<SubscriptionInner>>,
    updates_rx: Arc<Mutex<mpsc::UnboundedReceiver<LiveUpdate>>>,
}

#[derive(Debug, Deserialize)]
struct IdentityRecord {
    node_id: String,
}

impl SubscriptionManager {
    pub fn new(db: Arc<Surreal<Db>>) -> Self {
        let (updates_tx, updates_rx) = mpsc::unbounded_channel();
        Self {
            inner: Arc::new(Mutex::new(SubscriptionInner {
                db,
                updates_tx,
                active: Vec::new(),
            })),
            updates_rx: Arc::new(Mutex::new(updates_rx)),
        }
    }

    pub fn listen_task(&self) -> Task<Message> {
        let receiver = self.updates_rx.clone();
        Task::perform(
            async move { Self::next_update(receiver).await },
            |update| match update {
                Some(update) => Message::LiveUpdate(update),
                None => Message::None,
            },
        )
    }

    pub fn subscribe_slice_task(
        &self,
        focus_salsa_id: String,
        focus_node_id: String,
        parent_node: Option<String>,
        ancestors: Vec<String>,
    ) -> Task<Message> {
        let inner = self.inner.clone();
        Task::perform(
            async move {
                Self::subscribe_slice(inner, focus_salsa_id, focus_node_id, parent_node, ancestors)
                    .await
            },
            Message::SubscriptionsUpdated,
        )
    }

    pub fn unsubscribe_task(&self) -> Task<Message> {
        let inner = self.inner.clone();
        Task::perform(
            async move { Self::unsubscribe_all(inner).await },
            Message::SubscriptionsUpdated,
        )
    }

    async fn next_update(
        receiver: Arc<Mutex<mpsc::UnboundedReceiver<LiveUpdate>>>,
    ) -> Option<LiveUpdate> {
        let mut guard = receiver.lock().await;
        guard.recv().await
    }

    async fn subscribe_slice(
        inner: Arc<Mutex<SubscriptionInner>>,
        focus_salsa_id: String,
        focus_node_id: String,
        parent_node: Option<String>,
        ancestors: Vec<String>,
    ) -> Result<(), String> {
        Self::unsubscribe_all(inner.clone()).await?;

        let (db, updates_tx) = {
            let guard = inner.lock().await;
            (guard.db.clone(), guard.updates_tx.clone())
        };

        let focus_node = resolve_identity_node(&db, &focus_salsa_id)
            .await
            .unwrap_or(focus_node_id);

        let mut subscriptions = Vec::new();
        subscriptions.push(
            start_live_query(
                db.clone(),
                updates_tx.clone(),
                LiveUpdate::Focus,
                "LIVE SELECT * FROM identity WHERE salsa_id = $focus_id",
                vec![("focus_id", focus_salsa_id.clone())],
            )
            .await?,
        );

        if let Some(parent_node) = parent_node {
            subscriptions.push(
                start_live_query(
                    db.clone(),
                    updates_tx.clone(),
                    LiveUpdate::Siblings,
                    "LIVE SELECT out.* FROM child_of WHERE in = $parent_node ORDER BY index",
                    vec![("parent_node", parent_node)],
                )
                .await?,
            );
        }

        subscriptions.push(
            start_live_query(
                db.clone(),
                updates_tx.clone(),
                LiveUpdate::Children,
                "LIVE SELECT out.* FROM child_of WHERE in = $focus_node ORDER BY index",
                vec![("focus_node", focus_node)],
            )
            .await?,
        );

        for ancestor_id in ancestors {
            subscriptions.push(
                start_live_query(
                    db.clone(),
                    updates_tx.clone(),
                    LiveUpdate::Ancestors,
                    "LIVE SELECT * FROM identity WHERE salsa_id = $ancestor_id",
                    vec![("ancestor_id", ancestor_id)],
                )
                .await?,
            );
        }

        let mut guard = inner.lock().await;
        guard.active = subscriptions;
        Ok(())
    }

    async fn unsubscribe_all(inner: Arc<Mutex<SubscriptionInner>>) -> Result<(), String> {
        let (db, active) = {
            let mut guard = inner.lock().await;
            let active = std::mem::take(&mut guard.active);
            (guard.db.clone(), active)
        };

        for subscription in active {
            let _ = db
                .query("KILL $id")
                .bind(("id", subscription.live_id))
                .await
                .map_err(|e| e.to_string())?;
            subscription.task.abort();
        }

        Ok(())
    }
}

async fn start_live_query(
    db: Arc<Surreal<Db>>,
    updates_tx: mpsc::UnboundedSender<LiveUpdate>,
    kind: LiveUpdate,
    query: &str,
    binds: Vec<(&str, String)>,
) -> Result<LiveSubscription, String> {
    let mut response = db.query(query);
    for (key, value) in binds {
        response = response.bind((key, value));
    }

    let mut response = response.await.map_err(|e| e.to_string())?;
    let live_id: Uuid = response.take(0).map_err(|e| e.to_string())?;
    let mut stream = db.listen(live_id).await.map_err(|e| e.to_string())?;

    let task = tokio::spawn(async move {
        while let Some(_notification) = stream.next().await {
            let _ = updates_tx.send(kind.clone());
        }
    });

    Ok(LiveSubscription { live_id, task })
}

async fn resolve_identity_node(
    db: &Surreal<Db>,
    salsa_id: &str,
) -> Result<String, String> {
    let mut response = db
        .query("SELECT node_id FROM identity WHERE salsa_id = $salsa_id")
        .bind(("salsa_id", salsa_id.to_string()))
        .await
        .map_err(|e| e.to_string())?;

    let record: Option<IdentityRecord> = response.take(0).map_err(|e| e.to_string())?;
    Ok(record
        .map(|record| record.node_id)
        .unwrap_or_else(|| salsa_id.to_string()))
}
