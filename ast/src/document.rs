/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde::{Deserialize, Serialize};

use crate::block::Block;
use crate::types::PropSet;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
/// The abstract syntax-tree of a Prosidy document.
pub struct Document<'a> {
    #[serde(borrow)]
    props: PropSet<'a>,
    #[serde(borrow)]
    content: Vec<Block<'a>>,
}

impl<'a> Document<'a> {
    pub fn new(props: PropSet<'a>, content: Vec<Block<'a>>) -> Self {
        Document { props, content }
    }

    pub fn content(&self) -> &[Block<'a>] {
        &self.content
    }

    pub fn content_mut(&mut self) -> &mut Vec<Block<'a>> {
        &mut self.content
    }

    pub fn props(&self) -> &PropSet<'a> {
        &self.props
    }

    pub fn props_mut(&mut self) -> &mut PropSet<'a> {
        &mut self.props
    }
}
