mod error;
mod http;
mod router;
mod state;

pub use error::AppError;
pub use http::*;
pub use state::AppState;

pub fn new(app_state: AppState) -> axum::Router {
    router::create_router(app_state)
}
