use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Peer node identity for P2P discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerNode {
    pub id: String,
    pub address: String,
    pub port: u16,
    pub project_name: String,
    pub last_seen: u64,
}

/// Sync protocol message exchanged between peers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncMessage {
    /// Peer announces itself
    Announce(PeerNode),
    /// Request: what operations have you seen since this timestamp?
    PullRequest {
        since_timestamp: u64,
        project_name: String,
    },
    /// Response: here are my operations since the requested timestamp
    PullResponse {
        operations: Vec<SyncOperation>,
    },
    /// Push: new operations to share
    Push {
        operations: Vec<SyncOperation>,
    },
    /// Acknowledge receipt
    Ack {
        count: usize,
    },
}

/// A single sync operation (log entry)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncOperation {
    pub op_type: OpType,
    pub monad_id: String,
    pub timestamp: u64,
    pub payload: serde_json::Value,
}

/// Type of sync operation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OpType {
    /// A monad was created or updated
    Upsert,
    /// A monad was archived
    Archive,
    /// A ring was expanded
    RingExpand,
}

/// Local sync state tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    pub peers: Vec<PeerNode>,
    pub last_sync_timestamp: u64,
    pub operations_synced: u64,
}

impl SyncState {
    pub fn new() -> Self {
        Self {
            peers: Vec::new(),
            last_sync_timestamp: 0,
            operations_synced: 0,
        }
    }

    pub fn update_peer(&mut self, peer: PeerNode) {
        if let Some(existing) = self.peers.iter_mut().find(|p| p.id == peer.id) {
            *existing = peer;
        } else {
            self.peers.push(peer);
        }
    }

    pub fn remove_stale_peers(&mut self, max_age_secs: u64) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.peers.retain(|p| now.saturating_sub(p.last_seen) < max_age_secs);
    }
}

/// Save sync state to disk
pub fn save_sync_state(state: &SyncState, data_dir: &PathBuf) -> anyhow::Result<()> {
    let path = data_dir.join("sync_state.json");
    let json = serde_json::to_string_pretty(state)?;
    std::fs::write(path, json)?;
    Ok(())
}

/// Load sync state from disk
pub fn load_sync_state(data_dir: &PathBuf) -> SyncState {
    let path = data_dir.join("sync_state.json");
    if !path.exists() {
        return SyncState::new();
    }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

impl Default for SyncState {
    fn default() -> Self {
        Self::new()
    }
}
