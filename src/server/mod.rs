mod server_impl;
mod request;
mod response;

pub use self::request::Request;
pub use self::response::Response;
pub use self::server_impl::HttpServer;

use std::io::Write;
// use failure::Error;

/// the http service trait
/// user code should supply a type that impl the `handle` method for the http server
///
pub trait HttpService<T:Write> {
    /// Receives a `Request`/`Response` pair, and should perform some action on them.
    ///
    /// This could reading from the request, and writing to the response.
    fn handle(&self, request: Request, Response<T>);
}

impl<F, T:Write> HttpService<T> for F
where
    F: Fn(Request, Response<T>),
    F: Sync + Send,
{
    fn handle(&self, req: Request, res: Response<T>) {
        self(req, res)
    }
}
