/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde::{Deserialize, Serialize};

use crate::literal::Literal;
use crate::tag::InlineTag;
use crate::types::Text;

#[derive(Clone, Debug, Deserialize, Eq, From, PartialEq, Serialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Inline<'a> {
    SoftBreak,
    #[serde(borrow)]
    Tag(InlineTag<'a>),
    #[serde(borrow)]
    Text(Text<'a>),
    #[serde(borrow)]
    Literal(Literal<'a>),
}

impl<'a> Inline<'a> {
    pub fn as_tag(&self) -> Option<&InlineTag<'a>> {
        if let Inline::Tag(tag) = self {
            Some(tag)
        } else {
            None
        }
    }

    pub fn as_mut_tag(&mut self) -> Option<&mut InlineTag<'a>> {
        if let Inline::Tag(tag) = self {
            Some(tag)
        } else {
            None
        }
    }

    pub fn as_text(&self) -> Option<Text<'a>> {
        if let Inline::Text(text) = self {
            Some(text.clone())
        } else {
            None
        }
    }
}
