/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::block::Block;
use crate::inline::Inline;
use crate::types::{Key, PropSet};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
/// A named marker for a region of a document.
///
/// The [`BlockTag`](type.BlockTag.html) and [`InlineTag`](type.InlineTag.html) type synonyms are
/// monomorphized, recursive versions of this type.
pub struct Tag<'a, T> {
    name: Key,
    #[serde(borrow, flatten)]
    props: PropSet<'a>,
    content: Vec<T>,
}

impl<'a, T> Tag<'a, T> {
    #[inline]
    pub fn new<K: Into<Key>>(name: K, props: PropSet<'a>, content: Vec<T>) -> Self {
        let name = name.into();
        Tag {
            name,
            props,
            content,
        }
    }
}

impl<'a, T> Tag<'a, T> {
    #[inline]
    pub fn name(&self) -> &Key {
        &self.name
    }

    #[inline]
    pub fn set_name<K: Into<Key>>(&mut self, name: K) {
        self.name = name.into();
    }

    #[inline]
    pub fn props(&self) -> &PropSet<'a> {
        &self.props
    }

    #[inline]
    pub fn props_mut(&mut self) -> &mut PropSet<'a> {
        &mut self.props
    }

    #[inline]
    pub fn content(&self) -> &[T] {
        &self.content
    }

    #[inline]
    pub fn content_mut(&mut self) -> &mut Vec<T> {
        &mut self.content
    }
}

impl<'a, T> Deref for Tag<'a, T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        self.content()
    }
}

impl<'a, T> DerefMut for Tag<'a, T> {
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.content
    }
}

/// A [`Tag`](struct.Tag.html) annotating [`Block`](enum.Block.html) elements.
pub type BlockTag<'a> = Tag<'a, Block<'a>>;

/// A [`Tag`](struct.Tag.html) annotating [`Inline`](enum.Inline.html) elements.
pub type InlineTag<'a> = Tag<'a, Inline<'a>>;
