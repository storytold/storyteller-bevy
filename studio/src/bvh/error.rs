use core::fmt;

use anyhow::anyhow;
use bevy::prelude::Deref;

#[derive(Debug, Deref)]
pub struct Error(pub anyhow::Error);

impl From<anyhow::Error> for Error {
    fn from(value: anyhow::Error) -> Self {
        Self(value)
    }
}

pub trait Anyhow<T> {
    fn anyhow(self) -> anyhow::Result<T>;
}

impl<T, E> Anyhow<T> for Result<T, E>
where
    E: std::error::Error,
{
    fn anyhow(self) -> anyhow::Result<T> {
        self.map_err(|err| anyhow!("{err}"))
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", **self)
    }
}
