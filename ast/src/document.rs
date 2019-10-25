/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::block::Block;
use crate::types::{PropSet, Text};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
/// The abstract syntax-tree of a Prosidy document.
pub struct Document<'a> {
    #[serde(borrow, flatten)]
    meta: Meta<'a>,
    #[serde(borrow)]
    content: Vec<Block<'a>>,
}

impl<'a> Document<'a> {
    pub fn new(meta: Meta<'a>, content: Vec<Block<'a>>) -> Self {
        Document { meta, content }
    }

    pub fn content(&self) -> &[Block<'a>] {
        &self.content
    }

    pub fn content_mut(&mut self) -> &mut Vec<Block<'a>> {
        &mut self.content
    }

    pub fn meta(&self) -> &Meta<'a> {
        &self.meta
    }

    pub fn meta_mut(&mut self) -> &mut Meta<'a> {
        &mut self.meta
    }
}

impl<'a> Deref for Document<'a> {
    type Target = Meta<'a>;
    fn deref(&self) -> &Meta<'a> {
        self.meta()
    }
}

impl<'a> DerefMut for Document<'a> {
    fn deref_mut(&mut self) -> &mut Meta<'a> {
        self.meta_mut()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
/// Metadata for a Prosidy document.
pub struct Meta<'a> {
    #[serde(borrow)]
    title: Text<'a>,
    #[serde(borrow, flatten)]
    props: PropSet<'a>,
}

impl<'a> Meta<'a> {
    pub fn new<T>(title: T, props: PropSet<'a>) -> Self
    where
        T: Into<Text<'a>>,
    {
        let title = title.into();
        Meta { title, props }
    }

    pub fn new_default<T>(title: T) -> Self
    where
        T: Into<Text<'a>>,
    {
        Meta::new(title, PropSet::new())
    }

    pub fn title(&self) -> &Text<'a> {
        &self.title
    }

    pub fn set_title(&mut self, title: Text<'a>) {
        self.title = title;
    }

    pub fn props(&self) -> &PropSet<'a> {
        &self.props
    }

    pub fn props_mut(&mut self) -> &mut PropSet<'a> {
        &mut self.props
    }
}
