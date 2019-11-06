/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use pretty_assertions::assert_eq;
use prosidy_ast::*;
use prosidy_parse::{parse_document, Result};

const SOURCE: &str = include_str!("test03.pro");

#[test]
fn test_escape() -> Result<()> {
    let actual = parse_document(SOURCE)?;
    assert_eq!(actual, expected());
    Ok(())
}

fn expected() -> Document<'static> {
    Document::new(
        props! {
            title = "Tags",
        },
        vec![
            Tag::new("simple", props! {}, vec![]).into(),
            Tag::new(
                "props",
                props! {
                    foo,
                    bar = "baz",
                },
                vec![],
            )
            .into(),
            Tag::new(
                "nested",
                props! {},
                vec![Block::Content(vec![Text::new("Content!").into()])],
            )
            .into(),
            Tag::new(
                "multiline",
                props! {},
                vec![Block::Content(vec![
                    Text::new("Some ").into(),
                    Tag::new("em", props! {ru = "еще"}, vec![Text::new("more").into()]).into(),
                    Text::new(" content!").into(),
                ])],
            )
            .into(),
            Tag::new(
                "propsnested",
                props! {foo, bar = "baz"},
                vec![Block::Content(vec![
                    Text::new("Even ").into(),
                    Tag::new("em", props! {es = "más"}, vec![Text::new("more").into()]).into(),
                    Text::new(" content!").into(),
                ])],
            )
            .into(),
            Tag::new(
                "propsmultiline",
                props! {qux, baz = "foo"},
                vec![Tag::new(
                    "nestedinmultiline",
                    props! {},
                    vec![Block::Content(vec![Text::new("Ok then").into()])],
                )
                .into()],
            )
            .into(),
            BlockTag::new(
                "named-end",
                props!(),
                vec![
                    Block::Content(vec![
                        Text::new("This block has a named start/end delimiter.").into(),
                    ]),
                ],
            ).into(),
            Tag::new(
                "lit",
                props!(),
                vec![Block::Literal(Literal::from(Text::from(
                    "#this{isn\'t} valid at all!\n#:\n#:\n#:\n",
                )))],
            )
            .into(),
            Tag::new(
                "lit",
                props!(flag, withprops = "true"),
                vec![Literal::from(Text::from("    this literal has properties!\n")).into()],
            )
            .into(),
            Tag::new(
                "content",
                props!(),
                vec![Tag::new(
                    "lit",
                    props!(),
                    vec![Literal::from(Text::from("        Literals can be nested!\n")).into()],
                )
                .into()],
            )
            .into(),
        ],
    )
}
