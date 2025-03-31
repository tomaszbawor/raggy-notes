pub use crate::error::AppError;

pub type Result<T> = std::result::Result<T, AppError>;
