/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::fmt::Display;

use pest::iterators::{Pair, Pairs};

use crate::error::{Error, ErrorKind};
use crate::parse::Rule;

pub trait ResultExt<T, E>: Sized {
    fn recover(self) -> Result<Option<T>, E>;

    fn recover_default(self) -> Result<T, E>
    where
        T: Default,
    {
        self.recover().map(Option::unwrap_or_default)
    }

    fn unwrap_display(self) -> T
    where
        E: Display;
}

pub trait PairsExt<'p>: Iterator<Item = Pair<'p, Rule>> + Sized {
    fn peek(&self) -> Option<Pair<'p, Rule>>;

    fn rule(&mut self, rule: Rule) -> Result<Pair<'p, Rule>, Error> {
        if self.peek().filter(|x| x.as_rule() == rule).is_some() {
            Ok(self.next().unwrap())
        } else {
            Err(ErrorKind::NoMatch.into())
        }
    }

    fn assert_empty(self) -> Result<(), Error> {
        if let Some(err) = Error::trailing(self.map(|x| x.as_rule())) {
            Err(err)
        } else {
            Ok(())
        }
    }

    fn with_atom<F, T>(&mut self, rule: Rule, f: F) -> Result<T, Error>
    where
        F: FnOnce(&'p str) -> Result<T, Error>,
    {
        let pair = self.rule(rule)?;
        let span = pair.as_span();
        f(pair.as_str()).map_err(|e| e.annotate(rule, span))
    }

    fn with_block<'a, F, T>(&mut self, rule: Rule, f: F) -> Result<T, Error>
    where
        F: FnOnce(&mut Pairs<'p, Rule>) -> Result<T, Error>,
    {
        let pair = self.rule(rule)?;
        let span = pair.as_span();
        let mut pairs = pair.into_inner();
        let out = f(&mut pairs).map_err(|e| e.annotate(rule, span.clone()))?;
        pairs.assert_empty().map_err(|e| e.annotate(rule, span))?;
        Ok(out)
    }
}
