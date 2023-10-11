#![allow(unused)]

fn main() {
    println!("Hello, world!");
}

use anyhow::{Result, Context};
use sqlx::{Postgres, postgres::PgPoolOptions, Pool};

type Db = Pool<Postgres>;

async fn new_db_pool() -> Result<Db> {
    let db_url = std::env::var("DATABASE_URL").context("DATABASE_URL failed")?;
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .context("Connecting to DB failed")
}

async fn init_test() -> Db {
    use tokio::sync::OnceCell;
    static INIT: OnceCell<Db> = OnceCell::const_new();

    let mm = INIT
        .get_or_init(|| async { new_db_pool().await.unwrap() })
        .await;
    
    mm.clone()
}

#[derive(Debug, Clone)]
struct Task {
    pub id: i64,
    pub title: String,
}

#[derive(Debug, Clone)]
struct Id {
    id: i64,
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    #[serial_test::serial]
    async fn create_ok() -> Result<()> {
        // -- Setup
        let db = init_test().await;
        let fx_title = "create_ok title";

        // -- Exec
        let res = sqlx::query_as!(
            Id,
            "INSERT INTO task (title) VALUES ($1) RETURNING id",
            fx_title
        )
        .fetch_one(&db)
        .await
        .context("create Task failed")?;
        
        // -- Get
        let id = res.id;
        let task = sqlx::query_as!(Task, "SELECT * FROM task WHERE id = $1", id)
            .fetch_one(&db)
            .await
            .context("select Task failed")?;
        assert_eq!(task.title, fx_title);
        
        // -- Cleanup
        let count = sqlx::query!("DELETE from task WHERE id = $1", id)
            .execute(&db)
            .await
            .context("delete Task failed")?
            .rows_affected();
        assert_eq!(count, 1, "Did not delete the row?");

        Ok(())
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn get_err_not_found() -> Result<()> {
        // -- Setup
        let db = init_test().await;
        let fx_id = 100;

        // -- Exec
        let res = sqlx::query_as!(Task, "SELECT * FROM tasK WHERE id = $1", fx_id)
            .fetch_optional(&db)
            .await
            .context("select Task failed")?;

        // -- Check
        assert!(
            matches!(
                res,
                None,
            ),
            "Did a task get found?"
        );

        Ok(())
    }
    
}