//! Crate prelude

pub use crate::error::DotrError;

pub type Result<T> = core::result::Result<T, DotrError>;

pub use std::format as f;
