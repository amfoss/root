use async_graphql::http::GraphiQLSource;
use axum::response::{Html, IntoResponse};

pub async fn graphiql() -> impl IntoResponse {
    Html(
        GraphiQLSource::build()
            .endpoint("/")
            .subscription_endpoint("/ws")
            .finish(),
    )
}
