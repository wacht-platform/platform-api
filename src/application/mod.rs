mod error;
mod http;
mod router;
mod state;

use sqlx::PgPool;

pub use error::AppError;
pub use http::*;
pub use state::AppState;

pub fn new(pool: PgPool) -> axum::Router {
    let state = AppState::new(pool);
    router::create_router(state)
}
