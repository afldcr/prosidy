/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::borrow::{Borrow, Cow};
use std::fmt::{self, Debug, Display, Formatter};
use std::iter::FromIterator;
use std::ops::Deref;
use std::sync::Arc;

use serde::de::{Deserialize, Deserializer, Error as DeError, Visitor};
use serde::ser::{Serialize, Serializer};

use super::key::Key;

#[derive(Clone)]
pub enum Text<'a> {
    Borrowed(&'a str),
    Owned(Arc<str>),
}

impl Text<'static> {
    pub const EMPTY: Text<'static> = Text::new("");
}

impl<'a> Text<'a> {
    #[inline]
    pub const fn new(s: &'a str) -> Self {
        Text::Borrowed(s)
    }

    #[inline]
    pub fn into_owned(self) -> Text<'static> {
        match self {
            Text::Borrowed(s) => Text::Owned(Arc::from(s)),
            Text::Owned(s) => Text::Owned(s),
        }
    }

    #[inline]
    pub fn as_str(&self) -> &'a str {
        match *self {
            Text::Borrowed(s) => s,
            Text::Owned(ref s) => unsafe { &*(s.deref() as *const str) },
        }
    }

    #[inline]
    pub fn borrowed(&self) -> bool {
        !self.owned()
    }

    pub fn owned(&self) -> bool {
        match *self {
            Text::Borrowed(_) => false,
            Text::Owned(_) => true,
        }
    }
}

impl<'a> AsRef<str> for Text<'a> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'a> Borrow<str> for Text<'a> {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<'a> Debug for Text<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        fmt.debug_tuple("Text").field(&self.as_str()).finish()
    }
}

impl<'a> Default for Text<'a> {
    fn default() -> Self {
        Text::EMPTY
    }
}

impl<'a> Deref for Text<'a> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl<'a> Display for Text<'a> {
    #[inline]
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        Display::fmt(self.as_str(), fmt)
    }
}

impl<'a> Eq for Text<'a> {}

impl<'a> From<&'a str> for Text<'a> {
    fn from(s: &'a str) -> Self {
        Text::Borrowed(s)
    }
}

impl<'a> From<String> for Text<'a> {
    fn from(s: String) -> Self {
        Text::Owned(s.into())
    }
}

impl<'a> From<Key> for Text<'a> {
    fn from(s: Key) -> Self {
        Text::Owned(s.arc())
    }
}

impl<'a> From<Arc<str>> for Text<'a> {
    fn from(s: Arc<str>) -> Self {
        Text::Owned(s)
    }
}

impl<'a> From<Cow<'a, str>> for Text<'a> {
    fn from(cow: Cow<'a, str>) -> Self {
        match cow {
            Cow::Owned(s) => Text::Owned(s.into()),
            Cow::Borrowed(s) => Text::Borrowed(s),
        }
    }
}

impl<'a> FromIterator<Text<'a>> for Text<'a> {
    fn from_iter<I: IntoIterator<Item = Text<'a>>>(iter: I) -> Self {
        let mut iter = iter.into_iter();
        let first = iter.next().unwrap_or_default();
        if let Some(second) = iter.next() {
            let mut buf = String::with_capacity(first.len() + second.len());
            buf.push_str(&first);
            buf.push_str(&second);
            for next in iter {
                buf.push_str(&next);
            }
            Text::Owned(buf.into())
        } else {
            first
        }
    }
}

/// Text deserializes no-copy (if possible) via Serde
///
/// ```rust
/// # use prosidy_ast::Text;
/// let raw_json = r#""hello, world""#;
/// let text: Text = serde_json::from_str(raw_json).unwrap();
/// assert!(text.borrowed());
/// ```
impl<'a, 'de: 'a> Deserialize<'de> for Text<'a> {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct V;
        impl<'a> Visitor<'a> for V {
            type Value = Text<'a>;

            fn expecting(&self, fmt: &mut Formatter) -> fmt::Result {
                write!(fmt, "a string (owned or borrowed)")
            }

            fn visit_str<E: DeError>(self, s: &str) -> Result<Text<'a>, E> {
                Ok(Text::Borrowed(s).into_owned())
            }

            fn visit_borrowed_str<E: DeError>(self, s: &'a str) -> Result<Text<'a>, E> {
                Ok(Text::Borrowed(s))
            }
        }
        de.deserialize_str(V)
    }
}

/// Equality is always done via the string reference; ownership and lifetimes do not affect this
/// comparison.
///
/// ```rust
/// # use prosidy_ast::Text;
/// let t1 = Text::from("foo");
/// let t2 = t1.clone().to_owned();
/// assert_eq!(t1, t2);
/// ```
impl<'a, 'b> PartialEq<Text<'b>> for Text<'a> {
    fn eq(&self, other: &Text<'b>) -> bool {
        self.as_str() == other.as_str()
    }
}

impl<'a> Serialize for Text<'a> {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(&self)
    }
}
