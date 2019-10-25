/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::borrow::{Borrow, Cow};
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::{Arc, RwLock, Weak};

use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use weak_table::WeakHashSet;

use super::text::Text;

/// An interned string.
///
/// Keys are compared for equality using the location of their pointer, and are created via a
/// [`KeySet`] used to ensure that two keys created from the same string are equal.
///
/// ```rust
/// # use prosidy_ast::Key;
///
/// let str_1 = String::from("foo");
/// let str_2 = String::from("foo");
/// assert!(!std::ptr::eq(str_1.as_str(), str_2.as_str()));
///
/// let key_1 = Key::new(&str_1);
/// let key_2 = Key::new(&str_2);
/// assert_eq!(key_1, key_2);
/// ```
///
/// Keys incur a small creation cost for much faster equality and hashing operations.
#[derive(Clone, Debug)]
pub struct Key(Arc<str>);

impl Key {
    #[inline]
    pub fn new(s: &str) -> Key {
        GLOBAL_KEY_SET.intern(s)
    }

    #[inline]
    pub fn uninterned(s: &str) -> Key {
        Key(Arc::from(s))
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[inline]
    pub fn arc(&self) -> Arc<str> {
        Arc::clone(&self.0)
    }
}

impl AsRef<str> for Key {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for Key {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Deref for Key {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl Display for Key {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        Display::fmt(self.as_str(), fmt)
    }
}

impl Eq for Key {}

impl Hash for Key {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        let ptr: *const u8 = self.0.as_ptr();
        hasher.write_usize(ptr as usize)
    }
}

impl<'a> From<&'a str> for Key {
    fn from(s: &'a str) -> Key {
        Key::new(s)
    }
}

impl From<String> for Key {
    fn from(s: String) -> Key {
        Key::new(&s)
    }
}

impl<'a> From<Text<'a>> for Key {
    fn from(s: Text<'a>) -> Key {
        Key::new(&s)
    }
}

impl PartialEq for Key {
    #[inline]
    fn eq(&self, other: &Key) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl<'a> Deserialize<'a> for Key {
    fn deserialize<D: Deserializer<'a>>(de: D) -> Result<Key, D::Error> {
        let s = Cow::<'a, str>::deserialize(de)?;
        Ok(Key::new(&s))
    }
}

impl Serialize for Key {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(&self)
    }
}

/// A context for creating interned Keys.
///
/// For a set sharable across threads, see [`AtomicKeySet`].
#[derive(Clone, Default)]
struct KeySet(WeakHashSet<Weak<str>>);

impl KeySet {
    fn intern(&mut self, key: &str) -> Key {
        self.get(key).unwrap_or_else(|| {
            let arc = Arc::from(key);
            self.0.insert(Arc::clone(&arc));
            Key(arc)
        })
    }

    fn get(&self, key: &str) -> Option<Key> {
        self.0.get(key).map(Key)
    }
}

impl Debug for KeySet {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        fmt.debug_struct("KeySet").finish()
    }
}

/// A context, sharable across threads, for creating interned Keys.
///
/// If thread safety is not required, [`KeySet`] should perform better.
#[derive(Clone, Default)]
struct AtomicKeySet(Arc<RwLock<KeySet>>);

impl AtomicKeySet {
    fn intern(&self, key: &str) -> Key {
        self.get(key).unwrap_or_else(|| {
            let mut guard = self.0.write().unwrap_or_else(|x| x.into_inner());
            guard.intern(key)
        })
    }

    fn get(&self, key: &str) -> Option<Key> {
        let guard = self.0.read().unwrap_or_else(|x| x.into_inner());
        guard.get(key)
    }
}

impl Debug for AtomicKeySet {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        fmt.debug_struct("AtomicKeySet").finish()
    }
}

lazy_static::lazy_static! {
    static ref GLOBAL_KEY_SET: AtomicKeySet = AtomicKeySet::default();
}
