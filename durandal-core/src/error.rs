pub type Result<T> = std::result::Result<T, DurandalError>;

/// Enumerates the possible errors returned from this library
#[derive(Debug)]
pub enum DurandalError {
    /// This indicates that a requested external command could not be found.
    UnknownExternalCommand(String),

    /// Represents all other cases of IOError
    IOError(std::io::Error),
}

impl std::error::Error for DurandalError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            DurandalError::UnknownExternalCommand(_) => None,
            DurandalError::IOError(ref err) => Some(err),
        }
    }
}

impl std::fmt::Display for DurandalError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            DurandalError::UnknownExternalCommand(ref name) => {
                write!(f, "The external command '{}' could not be found", name)
            }
            DurandalError::IOError(ref err) => err.fmt(f),
        }
    }
}

impl From<std::io::Error> for DurandalError {
    fn from(err: std::io::Error) -> DurandalError {
        DurandalError::IOError(err)
    }
}
