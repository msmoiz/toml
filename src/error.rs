/// Error type for the toml crate.
#[derive(Debug, PartialEq)]
pub enum Error {
    Parse,
}

/// Result type for the toml crate.
pub type Result<T> = std::result::Result<T, Error>;
