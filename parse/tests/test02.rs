/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use pretty_assertions::assert_eq;
use prosidy_ast::*;
use prosidy_parse::{parse_document, Result};

const SOURCE: &str = include_str!("test02.pro");

#[test]
fn test_escape() -> Result<()> {
    let actual = parse_document(SOURCE)?;
    assert_eq!(actual, expected());
    Ok(())
}

fn expected<'b>() -> Document<'static> {
    Document::new(
        Meta::new(
            "002 - Lots of escapes (\\)",
            props! {
                author = "#hash#",
            },
        ),
        vec![Block::Content(vec![
            Text::new("This is a document that contains multiple escape sequences.").into(),
            Inline::SoftBreak,
            Text::new("Escape sequences appear as ").into(),
            InlineTag::new("lit", props! {}, vec![Text::new("\\{").into()]).into(),
            Text::new(", for instance.").into(),
        ])],
    )
}
