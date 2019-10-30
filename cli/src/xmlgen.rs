/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::borrow::Cow;

use ::xml::attribute::Attribute;
use ::xml::name::Name;
use ::xml::namespace::Namespace;
use ::xml::writer::events::XmlEvent;
use prosidy::{Block, BlockTag, Document, Inline, InlineTag, Literal, Text};

pub struct XMLGen<'r, 'a> {
    queue: Vec<Item<'r, 'a>>,
    namespace: &'r Namespace,
    tag_prefix: Option<&'r str>,
}

impl<'r, 'a> XMLGen<'r, 'a> {
    pub fn new<I, S>(
        seed: I,
        namespace: &'r Namespace,
        stylesheets: S,
        tag_prefix: Option<&'r str>,
    ) -> Self
    where
        I: Into<Item<'r, 'a>>,
        S: IntoIterator<Item = &'r str>,
    {
        let mut queue = Vec::with_capacity(128);
        queue.push(seed.into());
        for style in stylesheets {
            queue.push(Item::Stylesheet(style));
        }
        XMLGen {
            tag_prefix,
            queue,
            namespace,
        }
    }

    fn start_element(&self, name: &'r str, attributes: Cow<'r, [Attribute<'r>]>) -> XmlEvent<'r> {
        XmlEvent::StartElement {
            name: self.qualified_name(name),
            namespace: Cow::Borrowed(self.namespace),
            attributes,
        }
    }

    fn qualified_name(&self, name: &'r str) -> Name<'r> {
        Name {
            local_name: name,
            namespace: None,
            prefix: self.tag_prefix,
        }
    }

    fn push_block(&mut self, block: &'r Block<'a>) {
        match block {
            Block::Literal(lit) => self.queue.push(Item::Lit(lit, true)),
            Block::Tag(tag) => self.queue.push(tag.into()),
            Block::Content(inline) => self.queue.push(Item::Paragraph(inline)),
        }
    }

    fn push_inline(&mut self, inline: &'r Inline<'a>) {
        let item = match inline {
            Inline::Text(text) => text.into(),
            Inline::Tag(tag) => tag.into(),
            Inline::SoftBreak => Item::SoftBreak,
            Inline::Literal(lit) => Item::Lit(lit, false),
        };
        self.queue.push(item);
    }

    fn next_event<F>(&mut self, item: Item<'r, 'a>, emit: F)
    where
        F: FnOnce(XmlEvent<'r>),
    {
        match item {
            Item::Stylesheet(href) => {
                emit(XmlEvent::processing_instruction(
                    "xml-stylesheet",
                    Some(href),
                ));
            }
            Item::Document(doc) => {
                let mut attributes = Vec::with_capacity(1 + doc.props().len());
                attributes.push(Attribute {
                    name: TITLE,
                    value: doc.title().as_str(),
                });
                for (k, opt_v) in doc.props().iter() {
                    attributes.push(Attribute {
                        name: self.qualified_name(&k),
                        value: opt_v.map(|v| v.as_str()).unwrap_or(""),
                    });
                }
                self.queue.push(Item::End);
                self.queue.push(Item::Close);
                for block in doc.content().iter().rev() {
                    self.push_block(block);
                }
                emit(XmlEvent::StartElement {
                    name: DOCUMENT,
                    attributes: Cow::Owned(attributes),
                    namespace: Cow::Borrowed(self.namespace),
                });
            }
            Item::BlockTag(tag) => {
                self.queue.push(Item::Close);
                for block in tag.content().iter().rev() {
                    self.push_block(block);
                }
                let attrs: Vec<_> = tag
                    .props()
                    .settings()
                    .map(|(k, v)| Attribute {
                        name: self.qualified_name(&k),
                        value: v.clone().as_str(),
                    })
                    .collect();
                emit(self.start_element(&tag.name(), attrs.into()));
            }
            Item::InlineTag(tag) => {
                self.queue.push(Item::Close);
                for inline in tag.content().iter().rev() {
                    self.push_inline(inline);
                }
                let attrs: Vec<_> = tag
                    .props()
                    .settings()
                    .map(|(k, v)| Attribute {
                        name: self.qualified_name(&k),
                        value: v.clone().as_str(),
                    })
                    .collect();
                emit(self.start_element(&tag.name(), attrs.into()));
            }
            Item::Lit(lit, is_block) => {
                self.queue.push(Item::Close);
                self.queue.push(Item::LitContent(lit));
                emit(XmlEvent::StartElement {
                    name: if is_block { LITERAL } else { LITERAL_TEXT },
                    attributes: Default::default(),
                    namespace: Cow::Borrowed(self.namespace),
                });
            }
            Item::Text(text) => {
                emit(XmlEvent::Characters(text.as_str()));
            }
            Item::SoftBreak => {
                emit(XmlEvent::Characters(" "));
            }
            Item::Close => {
                emit(XmlEvent::EndElement { name: None });
            }
            Item::Paragraph(paragraph) => {
                self.queue.push(Item::Close);
                for item in paragraph.iter().rev() {
                    self.push_inline(item);
                }
                emit(XmlEvent::StartElement {
                    name: PARAGRAPH,
                    attributes: Default::default(),
                    namespace: Cow::Borrowed(self.namespace),
                })
            }
            Item::LitContent(content) => {
                emit(XmlEvent::Characters(&content));
            }
            Item::End => {
                self.queue.clear();
            }
        }
    }
}

impl<'r, 'a: 'r> Iterator for XMLGen<'r, 'a> {
    type Item = XmlEvent<'r>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut out = None;
        while out.is_none() {
            let item = self.queue.pop()?;
            self.next_event(item, |event| out = Some(event));
        }
        out
    }
}

#[derive(Clone, Debug)]
pub enum Item<'r, 'a> {
    Stylesheet(&'r str),
    Document(&'r Document<'a>),
    BlockTag(&'r BlockTag<'a>),
    InlineTag(&'r InlineTag<'a>),
    Lit(&'r Literal<'a>, bool),
    LitContent(&'r Literal<'a>),
    Paragraph(&'r Vec<Inline<'a>>),
    Text(&'r Text<'a>),
    SoftBreak,
    Close,
    End,
}

impl<'r, 'a> From<&'r Document<'a>> for Item<'r, 'a> {
    fn from(item: &'r Document<'a>) -> Self {
        Item::Document(item)
    }
}

impl<'r, 'a> From<&'r BlockTag<'a>> for Item<'r, 'a> {
    fn from(item: &'r BlockTag<'a>) -> Self {
        Item::BlockTag(item)
    }
}

impl<'r, 'a> From<&'r InlineTag<'a>> for Item<'r, 'a> {
    fn from(item: &'r InlineTag<'a>) -> Self {
        Item::InlineTag(item)
    }
}

impl<'r, 'a> From<&'r Text<'a>> for Item<'r, 'a> {
    fn from(item: &'r Text<'a>) -> Self {
        Item::Text(item)
    }
}

const DOCUMENT: Name = Name {
    local_name: "document",
    prefix: Some("prosidy"),
    namespace: None,
};

const PARAGRAPH: Name = Name {
    local_name: "paragraph",
    prefix: Some("prosidy"),
    namespace: None,
};

const LITERAL: Name = Name {
    local_name: "literal",
    prefix: Some("prosidy"),
    namespace: None,
};

const LITERAL_TEXT: Name = Name {
    local_name: "literal-text",
    prefix: Some("prosidy"),
    namespace: None,
};

const TITLE: Name = Name {
    local_name: "title",
    prefix: Some("prosidy"),
    namespace: None,
};
