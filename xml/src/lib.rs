/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use quick_xml::Result;
use quick_xml::events::{BytesStart, BytesEnd, BytesText, Event};
use prosidy_ast::*;

pub use quick_xml;

/// A trait used to encode a structure into one or more [`Event`]s.
pub trait XML {
    /// Write zero or more XML events via the `emit` function.
    fn to_events<F>(&self, emit: &mut F) -> Result<()>
    where
        F: for<'a> FnMut(Event<'a>) -> Result<()>;
}

impl<T> XML for [T]
where
    T: XML,
{
    fn to_events<F>(&self, emit: &mut F) -> Result<()>
    where
        F: for<'a> FnMut(Event<'a>) -> Result<()>,
    {
        for item in self.iter() {
            item.to_events(emit)?;
        }
        Ok(())
    }
}

impl<'p> XML for Block<'p> {
    fn to_events<F>(&self, emit: &mut F) -> Result<()>
    where
        F: for<'a> FnMut(Event<'a>) -> Result<()>,
    {
        match self {
            Block::Tag(tag) => tag.to_events(emit),
            Block::Literal(lit) => lit.to_events(emit),
            Block::Content(ct) => {
                let start = BytesStart::borrowed_name(TAG_PARAGRAPH.as_bytes());
                emit(Event::Start(start))?;
                ct.as_slice().to_events(emit)?;
                let end = BytesEnd::borrowed(TAG_PARAGRAPH.as_bytes());
                emit(Event::End(end))
            }
        }
    }
}

impl<'p> XML for Document<'p> {
    fn to_events<F>(&self, emit: &mut F) -> Result<()>
    where
        F: for<'a> FnMut(Event<'a>) -> Result<()>,
    {
        let mut start = BytesStart::borrowed_name(TAG_DOCUMENT.as_bytes());
        insert_props(&mut start, self.props());
        emit(Event::Start(start))?;
        self.content().to_events(emit)?;
        let end = BytesEnd::borrowed(TAG_DOCUMENT.as_bytes());
        emit(Event::End(end))?;
        Ok(())
    }
}

impl<'p> XML for Inline<'p> {
    fn to_events<F>(&self, emit: &mut F) -> Result<()>
    where
        F: for<'a> FnMut(Event<'a>) -> Result<()>,
    {
        match self {
            Inline::Tag(tag) => tag.to_events(emit),
            Inline::Literal(lit) => lit.to_events(emit),
            Inline::Text(text) => {
                let text = BytesText::from_plain_str(&text);
                emit(Event::Text(text))
            }
            Inline::SoftBreak => {
                let start = BytesStart::borrowed_name(TAG_SOFTBREAK.as_bytes());
                emit(Event::Empty(start))
            }
        }
    }
}

impl<'p> XML for Literal<'p> {
    fn to_events<F>(&self, emit: &mut F) -> Result<()>
    where
        F: for<'a> FnMut(Event<'a>) -> Result<()>,
    {
        let start = BytesStart::borrowed_name(TAG_LITERAL.as_bytes());
        emit(Event::Start(start))?;
        let text = BytesText::from_plain_str(&self);
        emit(Event::Text(text))?;
        let end = BytesEnd::borrowed(TAG_LITERAL.as_bytes());
        emit(Event::End(end))
    }
}

impl<'p, T> XML for Tag<'p, T>
where
    T: XML,
{
    fn to_events<F>(&self, emit: &mut F) -> Result<()>
    where
        F: for<'a> FnMut(Event<'a>) -> Result<()>,
    {
        let name: &[u8] = self.name().as_str().as_bytes();
        let mut start = BytesStart::borrowed_name(name);
        insert_props(&mut start, self.props());
        if self.content().is_empty() {
            emit(Event::Empty(start))
        } else {
            emit(Event::Start(start))?;
            self.content().to_events(emit)?;
            emit(Event::End(BytesEnd::borrowed(name)))
        }
    }
}


fn insert_props<'a>(start: &mut BytesStart<'a>, props: &PropSet<'a>) {
    for (name, opt_value) in props.iter() {
        start.push_attribute((
                name.as_str(),
                opt_value.unwrap_or(Text::EMPTY).as_str(),
        ));
    }
}

pub const TAG_DOCUMENT: &str = "prosidy:document";
pub const TAG_LITERAL: &str = "prosidy:literal";
const TAG_PARAGRAPH: &str = "prosidy:paragraph";
const TAG_SOFTBREAK: &str = "prosidy:softbreak";
