use chrono::{DateTime, Utc};
use sqlx::{Pool, Sqlite, SqlitePool};

use async_graphql::{Context, ErrorExtensions, FieldError, FieldResult, Object, SimpleObject};

#[derive(SimpleObject, sqlx::FromRow)]
pub struct Post {
    id: i32,
    title: String,
    category: Option<String>,
    contents: Option<String>,
    pub_date: DateTime<Utc>,
    open: i8,
}

#[derive(SimpleObject, sqlx::FromRow)]
pub struct Category {
    name: String,
}

#[derive(SimpleObject)]
pub struct Posts {
    current: i32,
    next: Option<i32>,
    prev: Option<i32>,
    category: String,
    page_size: i32,
    results: Vec<Post>,
}

pub struct QueryRoot;

#[derive(Debug, Error)]
pub enum BlogError {
    #[error("投稿が存在しません")]
    NotFoundPost,

    #[error("投稿が存在しません")]
    NotFoundPosts,

    #[error("ServerError")]
    ServerError(String),
}

impl ErrorExtensions for BlogError {
    fn extend(&self) -> FieldError {
        self.extend_with(|err, e| match err {
            BlogError::NotFoundPost => e.set("code", "NOT_FOUND"),
            BlogError::NotFoundPosts => e.set("code", "NOT_FOUND"),
            BlogError::ServerError(reason) => e.set("reason", reason.to_string()),
        })
    }
}

pub async fn db_connect() -> Pool<Sqlite> {
    SqlitePool::connect("todos.db3").await.unwrap()
}

/**
 * resolvers
 */
#[Object]
impl QueryRoot {
    #[allow(non_snake_case)]
    async fn getTasks(&self, _ctx: &Context<'_>) -> FieldResult<Option<Vec<Task>>> {
        let db = db_connect().await;
        let tasks = get_tasks(&db).await;
        match tasks {
            Ok(tasks) => Ok(Some(tasks)),
            Err(err) => return Err(FieldError::new(err.to_string())),
        }
    }

    #[allow(non_snake_case)]
    async fn getTask(&self, _ctx: &Context<'_>, id: i8) -> FieldResult<Option<Task>> {
        let db = db_connect().await;

        let task = get_task(&db, id).await;

        match task {
            Ok(task) => Ok(Some(task)),
            Err(err) => return Err(FieldError::new(err.to_string())),
        }
    }
}

#[derive(Debug, Error)]
pub enum TaskError {
    #[error("タスクが存在しません")]
    NotFoundTask,

    #[error("タスクが存在しません")]
    NotFoundTasks,

    #[error("サーバーエラー")]
    ServerError(String),
}

#[derive(SimpleObject, sqlx::FromRow)]
pub struct Task {
    id: i32,
    user_id: i32,
    title: String,
    description: String,
    due_date: String,
    category: Category, // created_at: NaiveDateTime,
}

#[derive(SimpleObject)]
pub struct Tasks {
    result: Vec<Task>,
}

pub async fn get_tasks(db: &Pool<Sqlite>) -> Result<Vec<Task>, TaskError> {
    let tasks = sqlx::query_as::<_, Task>(
        r#"
        SELECT *
        FROM tasks
        JOIN task_categories ON tasks.id = task_categories.task_id
        JOIN categories ON task_categories.category_id = categories.id
    "#,
    )
    .fetch_all(db)
    .await;

    match tasks {
        Ok(tasks) => Ok(tasks),
        Err(_) => return Err(TaskError::NotFoundTasks),
    }
}

pub async fn get_task(db: &Pool<Sqlite>, id: i8) -> Result<Task, TaskError> {
    let task = sqlx::query_as::<_, Task>(
        r#"
        SELECT *
        FROM tasks
        WHERE id = ?;
    "#,
    )
    .bind(id)
    .fetch_one(db)
    .await;

    match task {
        Ok(task) => Ok(task),
        Err(_) => return Err(TaskError::NotFoundTasks),
    }
}
