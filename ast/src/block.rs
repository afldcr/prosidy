/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde::{Deserialize, Serialize};

use crate::inline::Inline;
use crate::literal::Literal;
use crate::tag::BlockTag;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, From, Serialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Block<'a> {
    #[serde(borrow)]
    Content(Vec<Inline<'a>>),
    #[serde(borrow)]
    Literal(Literal<'a>),
    #[serde(borrow)]
    Tag(BlockTag<'a>),
}

impl<'a> Block<'a> {
    pub fn as_content(&self) -> Option<&Vec<Inline<'a>>> {
        if let Block::Content(content) = self {
            Some(content)
        } else {
            None
        }
    }

    pub fn as_mut_content(&mut self) -> Option<&mut Vec<Inline<'a>>> {
        if let Block::Content(content) = self {
            Some(content)
        } else {
            None
        }
    }

    pub fn as_tag(&self) -> Option<&BlockTag<'a>> {
        if let Block::Tag(tag) = self {
            Some(tag)
        } else {
            None
        }
    }

    pub fn as_mut_tag(&mut self) -> Option<&mut BlockTag<'a>> {
        if let Block::Tag(tag) = self {
            Some(tag)
        } else {
            None
        }
    }
}
