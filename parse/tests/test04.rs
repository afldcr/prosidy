/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use pretty_assertions::assert_eq;
use prosidy_ast::*;
use prosidy_parse::{parse_document, Result};

const SOURCE: &str = include_str!("test04.pro");

#[test]
fn test_empty_forms() -> Result<()> {
    let actual = parse_document(SOURCE)?;
    assert_eq!(actual, expected());
    Ok(())
}

fn expected() -> Document<'static> {
    Document::new(
        props! {
            title = "Empty forms",
        },
        vec![
            empty_block(),
            empty_block(),
            empty_block(),
            empty_block(),
            Block::Content(vec![
                empty_inline(),
                Inline::SoftBreak,
                empty_inline(),
                Inline::SoftBreak,
                empty_inline(),
                Inline::SoftBreak,
                empty_inline(),
            ]),
            empty_lit(),
            empty_lit(),
        ],
    )
}

fn empty_block() -> Block<'static> {
    BlockTag::new("block", props!(), vec![]).into()
}

fn empty_inline() -> Inline<'static> {
    InlineTag::new("inline", props!(), vec![]).into()
}

fn empty_lit() -> Block<'static> {
    BlockTag::new("lit", props!(), vec![Block::Literal(Literal::default())]).into()
}
