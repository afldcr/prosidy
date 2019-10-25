/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::block::Block;
use crate::document::Document;
use crate::inline::Inline;

#[derive(Copy, Clone, Debug, From)]
pub enum Node<'r, 'a> {
    Document(&'r Document<'a>),
    Block(&'r Block<'a>),
    Inline(&'r Inline<'a>),
}

impl<'r, 'a> Node<'r, 'a> {
    pub fn push_children<F>(self, mut f: F)
    where
        F: FnMut(Node<'r, 'a>),
    {
        match self {
            Node::Document(doc) => {
                for child in doc.content().iter().rev() {
                    f(child.into());
                }
            }
            Node::Block(Block::Tag(tag)) => {
                for child in tag.content().iter().rev() {
                    f(child.into());
                }
            }
            Node::Block(Block::Content(inline)) => {
                for child in inline.iter().rev() {
                    f(child.into());
                }
            }
            Node::Inline(Inline::Tag(tag)) => {
                for child in tag.content().iter().rev() {
                    f(child.into());
                }
            }
            Node::Inline(Inline::SoftBreak)
            | Node::Inline(Inline::Literal(_))
            | Node::Inline(Inline::Text(_))
            | Node::Block(Block::Literal(_)) => {}
        }
    }
}
