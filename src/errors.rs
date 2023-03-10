use image::ImageError;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum MyError {
    /// Unrecoverable logic errors.
    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Invalid commandline option: {0}")]
    InvalidArgument(String),
}


impl From<ImageError>  for MyError {
    fn from(err: ImageError) -> Self {
        MyError::InternalError(err.to_string())
    }
}