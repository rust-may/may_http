#[macro_use]
extern crate log;
extern crate http;
extern crate httparse;
extern crate may;
extern crate time;

pub mod client;
pub mod server;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
