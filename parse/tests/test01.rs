/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use pretty_assertions::assert_eq;
use prosidy_ast::*;
use prosidy_parse::{parse_document, Result};

const SOURCE: &str = include_str!("test01.pro");

#[test]
fn test_simple() -> Result<()> {
    let actual = parse_document(SOURCE)?;
    assert_eq!(actual, expected());
    Ok(())
}

fn expected() -> Document<'static> {
    Document::new(
        props!(),
        vec![Block::Content(vec![Inline::Text(Text::from(
            "This is a simple document with only a single paragraph.",
        ))])],
    )
}
