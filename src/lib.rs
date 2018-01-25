extern crate bytes;
#[macro_use]
extern crate failure;
pub extern crate http;
extern crate httparse;
#[macro_use]
extern crate log;
#[macro_use]
extern crate may;
extern crate time;

mod date;
mod buffer;
pub mod body;
pub mod error;
pub mod client;
pub mod server;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
