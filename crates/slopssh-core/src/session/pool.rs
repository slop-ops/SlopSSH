//! Reusable SSH connection pool with per-session limits.

use std::collections::HashMap;
use std::sync::Arc;

use super::info::SessionInfo;
use crate::ssh::auth::AuthMethod;
use crate::ssh::connection::{
    ClientHandler, ConnectionOptions, RemoteForwardMap, SshConnection, SshError,
};

/// A single pooled SSH connection with its in-use flag.
struct PooledConnection {
    connection: SshConnection,
    in_use: bool,
}

/// Pool that manages reusable SSH connections keyed by session ID.
pub struct ConnectionPool {
    pools: HashMap<String, Vec<PooledConnection>>,
    max_per_session: usize,
}

impl ConnectionPool {
    /// Creates a new pool with the given per-session connection limit.
    pub fn new(max_per_session: usize) -> Self {
        Self {
            pools: HashMap::new(),
            max_per_session,
        }
    }

    /// Returns a handle to an existing idle connection, creates a new one if
    /// capacity allows, or returns an error if the pool is exhausted.
    pub async fn get_or_create(
        &mut self,
        session_info: &SessionInfo,
        auth_method: &AuthMethod,
        options: &ConnectionOptions,
    ) -> Result<Arc<russh::client::Handle<ClientHandler>>, SshError> {
        if let Some(pool) = self.pools.get_mut(&session_info.id) {
            for conn in pool.iter_mut() {
                if !conn.in_use && conn.connection.is_connected() {
                    conn.in_use = true;
                    return conn
                        .connection
                        .handle()
                        .cloned()
                        .ok_or(SshError::NotConnected);
                }
            }

            if pool.len() < self.max_per_session {
                let forward_map: RemoteForwardMap =
                    Arc::new(tokio::sync::Mutex::new(HashMap::new()));
                let connection = SshConnection::connect_with_options(
                    session_info.clone(),
                    auth_method.clone(),
                    options.clone(),
                    forward_map,
                )
                .await?;
                let handle = connection.handle().cloned().ok_or(SshError::NotConnected)?;
                pool.push(PooledConnection {
                    connection,
                    in_use: true,
                });
                return Ok(handle);
            }

            Err(SshError::Other("Connection pool exhausted".to_string()))
        } else {
            let forward_map: RemoteForwardMap = Arc::new(tokio::sync::Mutex::new(HashMap::new()));
            let connection = SshConnection::connect_with_options(
                session_info.clone(),
                auth_method.clone(),
                options.clone(),
                forward_map,
            )
            .await?;
            let handle = connection.handle().cloned().ok_or(SshError::NotConnected)?;
            let session_id = session_info.id.clone();
            self.pools.insert(
                session_id,
                vec![PooledConnection {
                    connection,
                    in_use: true,
                }],
            );
            Ok(handle)
        }
    }

    /// Marks the first in-use connection for a session as released.
    pub fn release(&mut self, session_id: &str) {
        if let Some(pool) = self.pools.get_mut(session_id) {
            for conn in pool.iter_mut() {
                if conn.in_use {
                    conn.in_use = false;
                    break;
                }
            }
        }
    }

    /// Drops disconnected connections and removes empty session entries.
    pub async fn cleanup(&mut self) {
        let mut to_remove = Vec::new();
        for (session_id, pool) in &mut self.pools {
            pool.retain(|conn| conn.in_use || conn.connection.is_connected());
            if pool.is_empty() {
                to_remove.push(session_id.clone());
            }
        }
        for id in to_remove {
            self.pools.remove(&id);
        }
    }

    /// Disconnects and removes all connections for a given session.
    pub async fn close_session(&mut self, session_id: &str) {
        if let Some(pool) = self.pools.remove(session_id) {
            for mut conn in pool {
                let _ = conn.connection.disconnect().await;
            }
        }
    }

    /// Disconnects and removes all pooled connections.
    pub async fn close_all(&mut self) {
        for (_, pool) in self.pools.drain() {
            for mut conn in pool {
                let _ = conn.connection.disconnect().await;
            }
        }
    }

    /// Returns the number of currently in-use connections for a session.
    pub fn active_count(&self, session_id: &str) -> usize {
        self.pools
            .get(session_id)
            .map(|pool| pool.iter().filter(|c| c.in_use).count())
            .unwrap_or(0)
    }

    /// Returns the total number of connections (idle + active) for a session.
    pub fn total_count(&self, session_id: &str) -> usize {
        self.pools
            .get(session_id)
            .map(|pool| pool.len())
            .unwrap_or(0)
    }
}

impl Default for ConnectionPool {
    fn default() -> Self {
        Self::new(3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_pool_new() {
        let pool = ConnectionPool::new(5);
        assert_eq!(pool.active_count("nonexistent"), 0);
        assert_eq!(pool.total_count("nonexistent"), 0);
    }

    #[test]
    fn test_connection_pool_default() {
        let pool = ConnectionPool::default();
        assert_eq!(pool.active_count("nonexistent"), 0);
    }

    #[test]
    fn test_active_count_nonexistent() {
        let pool = ConnectionPool::new(3);
        assert_eq!(pool.active_count("no-such-session"), 0);
    }

    #[test]
    fn test_total_count_nonexistent() {
        let pool = ConnectionPool::new(3);
        assert_eq!(pool.total_count("no-such-session"), 0);
    }

    #[test]
    fn test_release_nonexistent() {
        let mut pool = ConnectionPool::new(3);
        pool.release("nonexistent");
    }

    #[tokio::test]
    async fn test_cleanup_empty() {
        let mut pool = ConnectionPool::new(3);
        pool.cleanup().await;
        assert_eq!(pool.active_count("any"), 0);
    }

    #[tokio::test]
    async fn test_close_session_nonexistent() {
        let mut pool = ConnectionPool::new(3);
        pool.close_session("nonexistent").await;
    }

    #[tokio::test]
    async fn test_close_all_empty() {
        let mut pool = ConnectionPool::new(3);
        pool.close_all().await;
    }

    #[test]
    fn test_pool_max_per_session_custom() {
        let pool = ConnectionPool::new(10);
        assert_eq!(pool.active_count("x"), 0);
        assert_eq!(pool.total_count("x"), 0);
    }

    #[test]
    fn test_pool_max_per_session_one() {
        let pool = ConnectionPool::new(1);
        assert_eq!(pool.active_count("x"), 0);
    }
}
