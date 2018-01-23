// mod server;
mod request;
mod response;

pub use self::request::Request;
pub use self::response::Response;

use failure::Error;

/// the http service trait
/// user code should supply a type that impl the `call` method for the http server
///
pub trait HttpService {
    fn handle(&self, request: Request) -> Result<Response, Error>;
}
