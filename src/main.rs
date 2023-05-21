#[macro_use]
extern crate thiserror;

mod resolvers;

use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    EmptyMutation, EmptySubscription, Request, Response, Schema,
};
use axum::{
    extract::Extension,
    handler::Handler,
    http::{HeaderValue, Method, StatusCode},
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};

use resolvers::QueryRoot;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

pub type BlogSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

async fn graphql_handler(schema: Extension<BlogSchema>, req: Json<Request>) -> Json<Response> {
    schema.execute(req.0).await.into()
}

async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(GraphQLPlaygroundConfig::new("/")))
}

async fn notfound_handler() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "not found")
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

#[tokio::main]
async fn main() {
    let server = async {
        let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();

        let app = Router::new()
            .route("/", get(graphql_playground).post(graphql_handler))
            .layer(
                CorsLayer::new()
                    // 一旦現段階で想定してるのはブログだけ
                    .allow_origin("*".parse::<HeaderValue>().unwrap())
                    .allow_methods([Method::GET, Method::POST, Method::OPTIONS]),
            )
            .layer(Extension(schema));

        let app = app.fallback(notfound_handler.into_service());

        let addr = SocketAddr::from(([127, 0, 0, 1], 8001));
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    };

    tokio::join!(server);
}
