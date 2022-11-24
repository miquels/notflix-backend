use std::time::{Duration, SystemTime};
use anyhow::Result;

use crate::db;
use crate::util::{Id, Rfc3339Time, some_or_return};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Session {
    pub username: String,
    pub user_id: i64,
    pub sessionid: String,
}

impl Session {

    // Create new session.
    pub async fn create(txn: &mut db::TxnHandle<'_>, user_id: i64, username: &str) -> Result<Session> {
        let sessionid = Id::new().to_string();
        let now = Rfc3339Time::new(SystemTime::now());

        sqlx::query!(
            r#"
                INSERT INTO sessions(user_id, sessionid, created, updated)
                VALUES(?, ?, ?, ?)"#,
            user_id,
            sessionid,
            now,
            now,
        )
        .execute(&mut *txn)
        .await?;
        log::info!("create_session: created new session for user_id {} session {}", user_id, sessionid);
        Ok(Session {
            username: username.to_string(),
            user_id,
            sessionid
        })
    }

    // Find session in the database.
    pub async fn find(txn: &mut db::TxnHandle<'_>, session_id: &str, timeout: Option<std::time::Duration>) -> Result<Option<Session>> {
        let row = sqlx::query!(
            r#"
                SELECT
                    u.username AS "username",
                    s.user_id AS "user_id",
                    s.sessionid AS "sessionid",
                    s.updated AS "updated: Rfc3339Time"
                FROM sessions s, users u
                WHERE s.user_id = u.id AND s.sessionid = ?"#,
            session_id
        )
        .fetch_optional(&mut *txn)
        .await?;

        let s = some_or_return!(row, {
            log::debug!("find_session: session '{}' not found in db", session_id);
            Ok(None)
        });

        if let Some(timeout) = timeout {
            let now = SystemTime::now();
            if s.updated.as_systemtime() + timeout < now {
                sqlx::query!(
                    r#"
                        DELETE FROM sessions WHERE sessionid = ?"#,
                    session_id
                )
                .execute(&mut *txn)
                .await?;
                log::info!("find_session: session {} for {}: timeout ({})", s.sessionid, s.username, s.updated);
                return Ok(None);
            }
        }

        let now = SystemTime::now();
        if let Ok(d) = now.duration_since(s.updated.as_systemtime()) {
            if d >= Duration::from_secs(300) {
                let now = Rfc3339Time::new(now);
                sqlx::query!(
                    r#"
                        UPDATE sessions SET updated = ? WHERE sessionid = ?"#,
                    now,
                    session_id,
                )
                .execute(&mut *txn)
                .await?;
            }
        }

        Ok(Some(Session {
            username: s.username,
            user_id: s.user_id,
            sessionid: s.sessionid,
        }))
    }

    // Delete session in the database.
    pub async fn delete(txn: &mut db::TxnHandle<'_>, sessionid: &str) -> Result<()> {
        sqlx::query!(
            r#"
                DELETE FROM sessions WHERE sessionid = ?"#,
            sessionid
        )
        .execute(&mut *txn)
        .await?;

        Ok(())
    }
}
