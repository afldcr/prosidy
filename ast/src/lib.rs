/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#[macro_use]
extern crate derive_more;

pub use block::Block;
pub use document::Document;
pub use inline::Inline;
pub use literal::Literal;
pub use node::Node;
pub use tag::{BlockTag, InlineTag, Tag};
pub use types::{Key, PropSet, Text};

mod block;
mod document;
mod inline;
mod literal;
mod node;
mod tag;
mod types;

#[macro_export]
macro_rules! props {
    ($($key:ident $(= $val:expr)?),* $(,)?) => {{
        let mut props = $crate::PropSet::new();
        $($crate::_insert_prop!(props, $key $(= $val)?);)*
        props
    }}
}

#[doc(hidden)]
#[macro_export]
macro_rules! _insert_prop {
    ($props:ident, $key:ident) => {
        $props.set(stringify!($key));
    };
    ($props:ident, $key:ident = $val:expr) => {
        $props.put(stringify!($key), $val);
    };
}

#[test]
fn test_serde() {
    use serde::Deserialize;
    let mut props = PropSet::new();
    props.set("test");
    props.put("language", "en");
    props.put("author", "J Alexander Feldman-Crough");
    let mut doc = Document::new(props, vec![]);
    let content = doc.content_mut();
    let heading = BlockTag::new(
        "h1",
        PropSet::new(),
        vec![vec![Inline::from(Text::from("hello world"))].into()],
    );
    content.push(heading.into());
    let actual = doc;
    let json = serde_json::to_string_pretty(&actual).unwrap();
    let mut serde = serde_json::Deserializer::from_str(json.as_str());
    let expected = Document::deserialize(&mut serde).unwrap();
    assert_eq!(actual, expected);
}
