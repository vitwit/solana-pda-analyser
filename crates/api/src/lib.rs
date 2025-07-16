pub mod handlers_simple;
pub mod routes_simple;
pub mod middleware;
pub mod server_simple;
pub mod error;

// Database-enabled modules
pub mod handlers;
pub mod routes;
pub mod server;

pub use handlers_simple::*;
pub use routes_simple::*;
pub use middleware::*;
pub use server_simple::*;
pub use error::*;

// Database-enabled exports
pub use handlers::*;
pub use routes::*;
pub use server::*;