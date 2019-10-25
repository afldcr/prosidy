/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::types::Text;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Eq, Deserialize, Deref, From, PartialEq, Serialize)]
pub struct Literal<'a>(#[serde(borrow)] Text<'a>);
