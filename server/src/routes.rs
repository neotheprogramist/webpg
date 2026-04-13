use askama::Template;
use salvo::prelude::*;
use serde::Deserialize;
use std::sync::Arc;

use crate::{AppState, db, error::AppError};

#[handler]
pub async fn home(res: &mut Response) {
    res.render(Text::Html(
        HomeTemplate { active: "/" }.render().expect("render home"),
    ));
}

#[handler]
pub async fn todo_page(depot: &mut Depot, res: &mut Response) -> Result<(), AppError> {
    let state = AppError::obtain::<Arc<AppState>>(depot)?;
    let todos = db::load_todos(&state.pool).await?;
    let html = TodoTemplate {
        active: "/todo/",
        todos,
    }
    .render()
    .expect("render todo");
    res.render(Text::Html(html));
    Ok(())
}

#[handler]
pub async fn chat_page(depot: &mut Depot, res: &mut Response) -> Result<(), AppError> {
    let state = AppError::obtain::<Arc<AppState>>(depot)?;
    let messages = db::load_recent_chat_messages(&state.pool, 100).await?;
    let html = ChatTemplate {
        active: "/chat/",
        messages,
        cert_fingerprint_hex: state.cert_fingerprint_hex.clone(),
    }
    .render()
    .expect("render chat");
    res.render(Text::Html(html));
    Ok(())
}

#[handler]
pub async fn create_todo(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
) -> Result<(), AppError> {
    let state = AppError::obtain::<Arc<AppState>>(depot)?;
    let form: CreateTodoForm = req.parse_form().await?;
    let title = form.title.trim();

    if !title.is_empty() {
        db::insert_todo(&state.pool, title).await?;
    }

    res.render(Redirect::other("/todo/"));
    Ok(())
}

#[handler]
pub async fn toggle_todo(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
) -> Result<(), AppError> {
    let state = AppError::obtain::<Arc<AppState>>(depot)?;
    let id: i64 = req.param("id").ok_or(AppError::NotFound)?;
    db::toggle_todo(&state.pool, id).await?;
    res.render(Redirect::other("/todo/"));
    Ok(())
}

#[handler]
pub async fn delete_todo(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
) -> Result<(), AppError> {
    let state = AppError::obtain::<Arc<AppState>>(depot)?;
    let id: i64 = req.param("id").ok_or(AppError::NotFound)?;
    db::delete_todo(&state.pool, id).await?;
    res.render(Redirect::other("/todo/"));
    Ok(())
}

#[derive(Template)]
#[template(path = "pages/index.html")]
struct HomeTemplate {
    active: &'static str,
}

#[derive(Template)]
#[template(path = "pages/todo.html")]
struct TodoTemplate {
    active: &'static str,
    todos: Vec<db::TodoRow>,
}

#[derive(Template)]
#[template(path = "pages/chat.html")]
struct ChatTemplate {
    active: &'static str,
    messages: Vec<db::ChatMessageRow>,
    cert_fingerprint_hex: String,
}

#[derive(Deserialize)]
pub struct CreateTodoForm {
    title: String,
}
