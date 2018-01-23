use failure::Error;

#[derive(Debug, Fail)]
enum HttpError {
    #[fail(display = "invalid method name: {}", name)] InvalidMethod { name: String },
}
