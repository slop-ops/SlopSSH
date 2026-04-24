use super::info::SessionInfo;
use crate::ssh::auth::AuthMethod;
use crate::ssh::connection::{ConnectionOptions, SshConnection, SshError};
use std::collections::HashMap;

struct PooledConnection {
    connection: SshConnection,
    in_use: bool,
}

pub struct ConnectionPool {
    pools: HashMap<String, Vec<PooledConnection>>,
    max_per_session: usize,
}

impl ConnectionPool {
    pub fn new(max_per_session: usize) -> Self {
        Self {
            pools: HashMap::new(),
            max_per_session,
        }
    }

    pub async fn get_or_create(
        &mut self,
        session_info: &SessionInfo,
        auth_method: &AuthMethod,
        options: &ConnectionOptions,
    ) -> Result<PooledConnectionGuard, SshError> {
        if let Some(pool) = self.pools.get_mut(&session_info.id) {
            for conn in pool.iter_mut() {
                if !conn.in_use && conn.connection.is_connected() {
                    conn.in_use = true;
                    return Ok(PooledConnectionGuard {
                        session_id: session_info.id.clone(),
                        index: 0,
                    });
                }
            }

            if pool.len() < self.max_per_session {
                let connection = SshConnection::connect_with_options(
                    session_info.clone(),
                    auth_method.clone(),
                    options.clone(),
                )
                .await?;
                pool.push(PooledConnection {
                    connection,
                    in_use: true,
                });
                return Ok(PooledConnectionGuard {
                    session_id: session_info.id.clone(),
                    index: pool.len() - 1,
                });
            }

            Err(SshError::Other("Connection pool exhausted".to_string()))
        } else {
            let connection = SshConnection::connect_with_options(
                session_info.clone(),
                auth_method.clone(),
                options.clone(),
            )
            .await?;
            let session_id = session_info.id.clone();
            self.pools.insert(
                session_id.clone(),
                vec![PooledConnection {
                    connection,
                    in_use: true,
                }],
            );
            Ok(PooledConnectionGuard {
                session_id,
                index: 0,
            })
        }
    }

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

    pub async fn cleanup(&mut self) {
        let mut to_remove = Vec::new();
        for (session_id, pool) in &mut self.pools {
            pool.retain(|conn| conn.in_use || conn.connection.is_connected());
            pool.iter_mut().for_each(|conn| {
                if !conn.in_use {
                    conn.in_use = false;
                }
            });
            if pool.is_empty() {
                to_remove.push(session_id.clone());
            }
        }
        for id in to_remove {
            self.pools.remove(&id);
        }
    }

    pub async fn close_all(&mut self) {
        for (_, pool) in self.pools.drain() {
            for mut conn in pool {
                let _ = conn.connection.disconnect().await;
            }
        }
    }

    pub fn active_count(&self, session_id: &str) -> usize {
        self.pools
            .get(session_id)
            .map(|pool| pool.iter().filter(|c| c.in_use).count())
            .unwrap_or(0)
    }

    pub fn total_count(&self, session_id: &str) -> usize {
        self.pools
            .get(session_id)
            .map(|pool| pool.len())
            .unwrap_or(0)
    }
}

pub struct PooledConnectionGuard {
    session_id: String,
    index: usize,
}

impl PooledConnectionGuard {
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn index(&self) -> usize {
        self.index
    }
}

impl Default for ConnectionPool {
    fn default() -> Self {
        Self::new(3)
    }
}
