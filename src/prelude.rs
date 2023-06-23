//! Crate prelude

pub use crate::error::DottError;

pub type Result<T> = core::result::Result<T, DottError>;

pub use std::format as f;
