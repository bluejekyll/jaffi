// Copyright 2022 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Error types for the crate

#![deny(missing_docs)]

use std::fmt;

use enum_as_inner::EnumAsInner;
use thiserror::Error;

/// The error kind for errors that get returned in the crate
#[derive(Debug, EnumAsInner, Error)]
#[non_exhaustive]
pub enum ErrorKind {
    /// An error with an arbitrary message, referenced as &'static str
    #[error("{0}")]
    Message(&'static str),

    /// An error with an arbitrary message, stored as String
    #[error("{0}")]
    Msg(String),

    /// An IO error ocurred
    #[error("{0}")]
    Io(#[from] std::io::Error),

    /// An error occurred in the templating engine
    #[error("{0}")]
    Template(#[from] tinytemplate::error::Error),

    /// An error occurred with the cafebabe library
    #[error("{0}")]
    Cafebabe(#[from] cafebabe::ParseError),
}

/// The error type for errors that get returned in the crate
#[derive(Error, Debug)]
#[non_exhaustive]
pub struct Error {
    /// Kind of error that ocurred
    pub kind: Box<ErrorKind>,
}

impl Error {
    /// Get the kind of the error
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.kind, f)
    }
}

impl<E> From<E> for Error
where
    E: Into<ErrorKind>,
{
    fn from(error: E) -> Self {
        let kind: ErrorKind = error.into();

        Self {
            kind: Box::new(kind),
        }
    }
}

impl From<&'static str> for Error {
    fn from(msg: &'static str) -> Self {
        ErrorKind::Message(msg).into()
    }
}

impl From<String> for Error {
    fn from(msg: String) -> Self {
        ErrorKind::Msg(msg).into()
    }
}

impl From<Error> for String {
    fn from(e: Error) -> Self {
        e.to_string()
    }
}

/// A trait marking a type which implements From<Error> and
/// std::error::Error types as well as Clone + Send
pub(crate) trait FromError: From<Error> + std::error::Error + Clone {}

impl<E> FromError for E where E: From<Error> + std::error::Error + Clone {}
