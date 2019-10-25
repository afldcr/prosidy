/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::fmt::{self, Display, Formatter};
use std::io::Error as IOError;
use std::result::Result as StdResult;

use pest::error::Error as PestError;

use crate::parse::Rule;
use crate::traits::ResultExt;

#[derive(Debug, thiserror::Error)]
pub struct Error {
    spans: Vec<Location>,
    #[source]
    kind: ErrorKind,
}

impl Error {
    pub fn annotate(mut self, rule: Rule, span: pest::Span) -> Self {
        let start = span.start();
        let end = span.end();
        let loc = Location { start, end, rule };
        self.spans.push(loc);
        self
    }

    pub fn trailing<I: Iterator<Item = Rule>>(trailing: I) -> Option<Self> {
        let trailing: Vec<_> = trailing.collect();
        if trailing.is_empty() {
            None
        } else {
            Some(ErrorKind::Trailing(trailing).into())
        }
    }
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        writeln!(fmt, "{}", self.kind)?;
        if !self.spans.is_empty() {
            writeln!(fmt, "trace:")?;
            for span in self.spans.iter() {
                writeln!(
                    fmt,
                    "    in rule {rule:?}, {start}-{end}",
                    rule = span.rule,
                    start = span.start,
                    end = span.end,
                )?
            }
        }
        Ok(())
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error {
            spans: Vec::new(),
            kind,
        }
    }
}

impl<T> ResultExt<T, Error> for Result<T> {
    fn recover(self) -> Result<Option<T>> {
        if let Some(ErrorKind::NoMatch) = self.as_ref().err().map(|e| &e.kind) {
            Ok(None)
        } else {
            self.map(Some)
        }
    }

    fn unwrap_display(self) -> T {
        self.unwrap_or_else(|e| panic!("{}", e))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("Invalid escape sequence {0:?}")]
    InvalidEscape(String),
    #[error("IO Error: {0:}")]
    IOError(#[from] IOError),
    #[error("No match.")]
    NoMatch,
    #[error("Syntax error: {0:}")]
    SyntaxError(#[from] PestError<Rule>),
    #[error("Trailing rules: {0:?}")]
    Trailing(Vec<Rule>),
}

#[derive(Debug)]
pub struct Location {
    pub rule: Rule,
    pub start: usize,
    pub end: usize,
}

pub type Result<T> = StdResult<T, Error>;
