use async_graphql::{http::GraphiQLSource, EmptySubscription, Schema};
use async_graphql_axum::GraphQL;
use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use tower_http::cors::CorsLayer;

use crate::graphql::{Mutation, Query};

pub fn setup_router(
    schema: Schema<Query, Mutation, EmptySubscription>,
    cors: CorsLayer,
    is_dev: bool,
) -> Router {
    let router = Router::new()
        .route_service("/", GraphQL::new(schema.clone()))
        .layer(cors);

    if is_dev {
        tracing::info!("GraphiQL playground enabled at /graphiql");
        router.route(
            "/graphiql",
            get(graphiql).post_service(GraphQL::new(schema)),
        )
    } else {
        router
    }
}

async fn graphiql() -> impl IntoResponse {
    Html(
        GraphiQLSource::build()
            .endpoint("/graphiql")
            .subscription_endpoint("/ws")
            .finish(),
    )
}
