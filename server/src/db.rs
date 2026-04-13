use serde::Serialize;
use sqlx::{
    FromRow, SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::{path::Path, str::FromStr};

pub async fn connect(data_dir: &Path) -> Result<SqlitePool, sqlx::Error> {
    let db_path = data_dir.join("webpg.db");
    let database_url = format!("sqlite://{}", db_path.display());
    let options = SqliteConnectOptions::from_str(&database_url)?
        .create_if_missing(true)
        .foreign_keys(true);

    SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
}

pub async fn init_schema(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS todos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            completed INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS chat_messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            body TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn load_todos(pool: &SqlitePool) -> Result<Vec<TodoRow>, sqlx::Error> {
    sqlx::query_as::<_, TodoRow>(
        r#"
        SELECT id, title, completed
        FROM todos
        ORDER BY completed ASC, id DESC
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn insert_todo(pool: &SqlitePool, title: &str) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO todos (title) VALUES (?1)")
        .bind(title)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn toggle_todo(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE todos SET completed = NOT completed WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn delete_todo(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM todos WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn load_recent_chat_messages(
    pool: &SqlitePool,
    limit: i64,
) -> Result<Vec<ChatMessageRow>, sqlx::Error> {
    let mut messages = sqlx::query_as::<_, ChatMessageRow>(
        "SELECT id, body, created_at FROM chat_messages ORDER BY id DESC LIMIT ?1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    messages.reverse();
    Ok(messages)
}

pub async fn insert_chat_message(
    pool: &SqlitePool,
    body: &str,
) -> Result<ChatMessageRow, sqlx::Error> {
    sqlx::query_as::<_, ChatMessageRow>(
        "INSERT INTO chat_messages (body) VALUES (?1) RETURNING id, body, created_at",
    )
    .bind(body)
    .fetch_one(pool)
    .await
}

#[derive(FromRow)]
pub struct TodoRow {
    pub id: i64,
    pub title: String,
    pub completed: bool,
}

#[derive(FromRow, Serialize, Clone)]
pub struct ChatMessageRow {
    pub id: i64,
    pub body: String,
    pub created_at: String,
}
