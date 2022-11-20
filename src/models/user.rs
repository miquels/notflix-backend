use anyhow::Result;
use sha_crypt::{Sha512Params, sha512_simple, sha512_check};

use crate::db;
use crate::util::ok_or_return;

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub email: Option<String>,
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct UpdateUser {
    pub id: i64,
    pub username: Option<String>,
    pub password: Option<String>,
    pub email: Option<String>,
}

impl User {
    pub async fn lookup(dbh: &mut db::TxnHandle<'_>, username: &str) -> Result<Option<User>> {
        // Find the item in the database.
        let r = sqlx::query_as!(
            User,
            r#"
                SELECT *
                FROM users
                WHERE username = ?"#,
            username
        )
        .fetch_optional(dbh)
        .await?;

        Ok(r)
    }

    pub fn verify(&self, password: &str) -> bool {
        sha512_check(password, &self.password).is_ok()
    }

    pub async fn get_users(dbh: &mut db::TxnHandle<'_>) -> Result<Vec<User>> {
        let r = sqlx::query_as!(
            User,
            r#"SELECT id, username, '' AS password, email FROM users"#,
        )
        .fetch_all(dbh)
        .await?;

        Ok(r)
    }

    pub async fn insert(&mut self, txn: &mut db::TxnHandle<'_>) -> Result<i64> {
        let params = ok_or_return!(Sha512Params::new(10_000), |_| {
            bail!("unexpected error in sha_crypt::Sha512Params::new");
        });
        let hashed = ok_or_return!(sha512_simple(&self.password, &params), |_| {
            bail!("unexpected error in sha-crypt::sha512_simple");
        });

        let id = sqlx::query!(
            r#"
                INSERT INTO users(username, password, email)
                VALUES(?, ?, ?)"#,
            self.username,
            hashed,
            self.email,
        )
        .execute(&mut *txn)
        .await?
        .last_insert_rowid();

        Ok(id)
    }

    pub async fn delete(txn: &mut db::TxnHandle<'_>, user_id: i64) -> Result<bool> {
        let r = sqlx::query!(
            r#"SELECT id FROM users WHERE id = ?"#,
            user_id
        )
        .fetch_optional(&mut *txn)
        .await?;
        if r.is_none() {
            return Ok(false);
        }

        sqlx::query!(
            r#"DELETE FROM users WHERE id = ?"#,
            user_id
        )
        .execute(&mut *txn)
        .await?;

        Ok(true)
    }
}

impl UpdateUser {
    pub async fn update(&self, txn: &mut db::TxnHandle<'_>) -> Result<bool> {
        let mut sql = "UPDATE users SET ".to_string();
        let mut args: Vec<&str> = Vec::new();
        if self.username.is_some() {
            args.push("username = ?");
        }
        if self.password.is_some() {
            args.push("password = ?");
        }
        if self.email.is_some() {
            args.push("email = ?");
        }
        sql.push_str(&args.join(", "));
        sql.push_str(" WHERE id = ?");

        let mut q = sqlx::query(&sql);
        if let Some(username) = self.username.as_ref() {
            q = q.bind(username);
        }
        if let Some(password) = self.password.as_ref() {
            q = q.bind(password);
        }
        if let Some(email) = self.email.as_ref() {
            q = q.bind(email);
        }
        q = q.bind(&self.id);

        let nr = q.execute(&mut *txn).await?.rows_affected();

        Ok(nr > 0)
    }
}
