pub mod handlers_simple;
pub mod routes_simple;
pub mod middleware;
pub mod server_simple;
pub mod error;

// Database-enabled modules
pub mod handlers;
pub mod routes;
pub mod server;

pub use handlers_simple::{health_check as simple_health_check};
pub use routes_simple::{AppState as SimpleAppState, create_simple_router};
pub use middleware::*;
pub use server_simple::{run_simple_server, SimpleServerConfig};
pub use error::*;

// Database-enabled exports
pub use handlers::*;
pub use routes::{AppState, create_router};
pub use server::*;