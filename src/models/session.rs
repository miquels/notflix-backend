use std::time::SystemTime;
use anyhow::Result;

use crate::db;
use crate::util::{Rfc3339Time, some_or_return};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Session {
    pub username: String,
    pub user_id: i64,
    pub sessionid: String,
}

impl Session {

    // Create new session.
    pub async fn create(txn: &mut db::TxnHandle<'_>, user_id: i64, username: &str) -> Result<Session> {
        let sessionid = loop {
            let id = nanoid::nanoid!();
            if id.starts_with("-") || id.starts_with("_") || id.ends_with("-") || id.ends_with("_") {
                continue;
            }
            break id;
        };
        let now = Rfc3339Time::new(SystemTime::now());

        sqlx::query!(
            r#"
                INSERT INTO sessions(user_id, sessionid, timestamp)
                VALUES(?, ?, ?)"#,
            user_id,
            sessionid,
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
                    s.timestamp AS "timestamp: Rfc3339Time"
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
            if *s.timestamp.as_systemtime() + timeout < now {
                sqlx::query!(
                    r#"
                        DELETE FROM sessions WHERE sessionid = ?"#,
                    session_id
                )
                .execute(&mut *txn)
                .await?;
                log::info!("find_session: session {} for {}: timeout ({})", s.sessionid, s.username, s.timestamp);
                return Ok(None);
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