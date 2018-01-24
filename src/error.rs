// use failure::Error;

#[derive(Debug, Fail)]
pub enum HttpError {
    #[fail(display = "invalid method name: {}", name)] InvalidMethod { name: String },
}
