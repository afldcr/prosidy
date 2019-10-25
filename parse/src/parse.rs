/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::iter;

use pest::Parser;
use prosidy_ast::*;

use crate::error::{ErrorKind::*, Result};
use crate::traits::*;

pub fn parse_meta<'p>(src: &'p str) -> Result<Meta<'p>> {
    let mut ast = DocumentParser::parse(Rule::Document, src).map_err(SyntaxError)?;
    Meta::parse(&mut ast)
}

pub fn parse_document<'p>(src: &'p str) -> Result<Document<'p>> {
    let mut ast = DocumentParser::parse(Rule::Document, src).map_err(SyntaxError)?;
    let doc = Document::parse(&mut ast)?;
    ast.assert_empty()?;
    Ok(doc)
}

pub trait Parse<'p>: Sized {
    fn parse(pairs: &mut Pairs<'p>) -> Result<Self>;
}

impl<'p, T> Parse<'p> for Vec<T>
where
    T: Parse<'p>,
{
    fn parse(pairs: &mut Pairs<'p>) -> Result<Self> {
        log::debug!("parsing vector");
        let mut buf = Vec::with_capacity(pairs.clone().count());
        while let Some(item) = T::parse(pairs).recover()? {
            buf.push(item);
        }
        buf.shrink_to_fit();
        Ok(buf)
    }
}

impl<'p> Parse<'p> for Block<'p> {
    fn parse(pairs: &mut Pairs<'p>) -> Result<Self> {
        fn content<'p>(pairs: &mut Pairs<'p>) -> Result<Block<'p>> {
            pairs.with_block(Rule::Paragraph, |pairs| {
                log::debug!("parsing block content");
                let content = Vec::parse(pairs)?;
                Ok(Block::Content(content))
            })
        }

        fn tag<'p>(pairs: &mut Pairs<'p>) -> Result<Block<'p>> {
            BlockTag::parse(pairs).map(Block::Tag)
        }

        tag(pairs)
            .recover()
            .transpose()
            .unwrap_or_else(|| content(pairs))
    }
}

impl<'p> Parse<'p> for BlockTag<'p> {
    fn parse(pairs: &mut Pairs<'p>) -> Result<Self> {
        pairs
            .with_block(Rule::BlockTag, |pairs| {
                log::debug!("parsing block tag");
                let name = Key::parse(pairs)?;
                let props = PropSet::parse(pairs).recover_default()?;
                let content = Vec::parse(pairs)?;
                Ok(BlockTag::new(name, props, content))
            })
            .recover()
            .transpose()
            .unwrap_or_else(|| {
                pairs.with_block(Rule::LiteralTag, |pairs| {
                    let name = Key::parse(pairs)?;
                    let props = PropSet::parse(pairs).recover_default()?;
                    let content = Literal::parse(pairs).recover_default()?;
                    Ok(BlockTag::new(name, props, vec![Block::Literal(content)]))
                })
            })
    }
}

impl<'p> Parse<'p> for Document<'p> {
    fn parse(pairs: &mut Pairs<'p>) -> Result<Self> {
        pairs.with_block(Rule::Document, |pairs| {
            log::debug!("parsing document");
            let meta: Meta<'p> = Meta::parse(pairs)?;
            let content = Vec::parse(pairs)?;
            pairs.rule(Rule::EOI)?;
            Ok(Document::new(meta, content))
        })
    }
}

impl<'p> Parse<'p> for Inline<'p> {
    fn parse(pairs: &mut Pairs<'p>) -> Result<Self> {
        fn softbreak<'p>(pairs: &mut Pairs<'p>) -> Result<Inline<'p>> {
            pairs.with_block(Rule::SoftBreak, |_| {
                log::debug!("parsing soft break");
                Ok(Inline::SoftBreak)
            })
        }

        fn text<'p>(pairs: &mut Pairs<'p>) -> Result<Inline<'p>> {
            Text::parse(pairs).map(Inline::Text)
        }

        fn tag<'p>(pairs: &mut Pairs<'p>) -> Result<Inline<'p>> {
            InlineTag::parse(pairs).map(Inline::Tag)
        }

        softbreak(pairs)
            .recover()
            .transpose()
            .unwrap_or_else(|| tag(pairs))
            .recover()
            .transpose()
            .unwrap_or_else(|| text(pairs))
    }
}

impl<'p> Parse<'p> for InlineTag<'p> {
    fn parse(pairs: &mut Pairs<'p>) -> Result<Self> {
        pairs.with_block(Rule::InlineTag, |pairs| {
            log::debug!("parsing inline tag");
            let name = Key::parse(pairs)?;
            let props = PropSet::parse(pairs).recover_default()?;
            let content = pairs
                .with_block(Rule::Paragraph, |pairs| Vec::parse(pairs))
                .recover_default()?;
            Ok(InlineTag::new(name, props, content))
        })
    }
}

impl<'p> Parse<'p> for Literal<'p> {
    fn parse(pairs: &mut Pairs<'p>) -> Result<Self> {
        pairs.with_atom(Rule::Literal, |s| Ok(Literal::from(Text::new(s))))
    }
}

impl<'p> Parse<'p> for Key {
    fn parse(pairs: &mut Pairs<'p>) -> Result<Self> {
        pairs.with_atom(Rule::Key, |s| {
            log::debug!("parsing key");
            Ok(Key::new(s))
        })
    }
}

impl<'p> Parse<'p> for Meta<'p> {
    fn parse(pairs: &mut Pairs<'p>) -> Result<Self> {
        pairs.with_block(Rule::Header, |pairs| {
            log::debug!("parsing header");
            let title = pairs.with_block(Rule::Title, |pairs| Text::parse(pairs))?;
            let props = PropSet::parse(pairs)?;
            Ok(Meta::new(title, props))
        })
    }
}

impl<'p> Parse<'p> for PropSet<'p> {
    fn parse(pairs: &mut Pairs<'p>) -> Result<Self> {
        fn prop<'p>(pairs: &mut Pairs<'p>, props: &mut PropSet<'p>) -> Result<()> {
            pairs.with_block(Rule::Prop, |pairs| {
                log::debug!("parsing prop key-value pair");
                let key = Key::parse(pairs)?;
                let opt_value = pairs
                    .with_block(Rule::QuotedText, |pairs| Text::parse(pairs))
                    .recover()?;
                if let Some(value) = opt_value {
                    props.put(key, value);
                } else {
                    props.set(key);
                }
                Ok(())
            })
        }

        fn header_prop<'p>(pairs: &mut Pairs<'p>, props: &mut PropSet<'p>) -> Result<()> {
            pairs.with_block(Rule::DocumentProp, |pairs| {
                log::debug!("parsing header prop key-value pair");
                let key = Key::parse(pairs)?;
                let opt_value = pairs
                    .with_block(Rule::DocumentPropValue, |pairs| Text::parse(pairs))
                    .recover()?;
                if let Some(value) = opt_value {
                    props.put(key, value);
                } else {
                    props.set(key);
                }
                Ok(())
            })
        }

        fn props<'p>(pairs: &mut Pairs<'p>) -> Result<PropSet<'p>> {
            pairs.with_block(Rule::Props, |pairs| {
                log::debug!("parsing property set");
                let mut props = PropSet::new();
                while prop(pairs, &mut props).recover()?.is_some() {}
                Ok(props)
            })
        }

        fn headers<'p>(pairs: &mut Pairs<'p>) -> Result<PropSet<'p>> {
            pairs.with_block(Rule::DocumentProps, |pairs| {
                log::debug!("parsing header property set");
                let mut props = PropSet::new();
                while header_prop(pairs, &mut props).recover()?.is_some() {}
                Ok(props)
            })
        }

        props(pairs)
            .recover()
            .transpose()
            .unwrap_or_else(|| headers(pairs))
    }
}

impl<'p> Parse<'p> for Text<'p> {
    fn parse(pairs: &mut Pairs<'p>) -> Result<Self> {
        fn plaintext<'p>(pairs: &mut Pairs<'p>) -> Option<Result<Text<'p>>> {
            pairs
                .with_atom(Rule::PlainText, |s| {
                    log::debug!("parsing plain text");
                    Ok(Text::from(s))
                })
                .recover()
                .transpose()
        }

        fn quotetext<'p>(pairs: &mut Pairs<'p>) -> Option<Result<Text<'p>>> {
            pairs
                .with_atom(Rule::PlainQuotedText, |s| {
                    log::debug!("parsing quoted text");
                    Ok(Text::from(s))
                })
                .recover()
                .transpose()
        }

        fn escaped<'p>(pairs: &mut Pairs<'p>) -> Option<Result<Text<'p>>> {
            pairs
                .with_atom(Rule::EscapedPlainText, |s| {
                    log::debug!("parsing plain text escape");
                    match s {
                        r#"\n"# => Ok(Text::Borrowed("\n")),
                        r#"\t"# => Ok(Text::Borrowed("\t")),
                        r#"\\"# => Ok(Text::Borrowed("\\")),
                        r#"\#"# => Ok(Text::Borrowed("#")),
                        r#"\{"# => Ok(Text::Borrowed("{")),
                        r#"\}"# => Ok(Text::Borrowed("}")),
                        _ => Err(InvalidEscape(s.into()).into()),
                    }
                })
                .recover()
                .transpose()
        }

        let mut iter = iter::from_fn(|| {
            plaintext(pairs)
                .or_else(|| escaped(pairs))
                .or_else(|| quotetext(pairs))
        })
        .peekable();

        if iter.peek().is_none() {
            Err(NoMatch.into())
        } else {
            iter.collect()
        }
    }
}

type Pairs<'p> = pest::iterators::Pairs<'p, Rule>;

impl<'p> PairsExt<'p> for Pairs<'p> {
    fn peek(&self) -> Option<Pair<'p>> {
        self.peek()
    }
}

type Pair<'p> = pest::iterators::Pair<'p, Rule>;

#[derive(pest_derive::Parser)]
#[grammar = "document.pest"]
struct DocumentParser;
