/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use super::key::Key;
use super::text::Text;

/// A set of Prosidy properties.
///
/// `PropSet`s consist of both valued _properties_ (e.g. `foo = 'bar'`) and boolean _settings_
/// (e.g.  `'baz'`).
#[derive(Clone, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct PropSet<'a> {
    properties: HashSet<Key>,
    #[serde(borrow)]
    settings: HashMap<Key, Text<'a>>,
}

impl<'a> PropSet<'a> {
    #[inline]
    pub fn new() -> Self {
        PropSet::default()
    }

    #[inline]
    pub fn with_capacity(cap: usize) -> Self {
        PropSet {
            properties: HashSet::with_capacity(cap),
            settings: HashMap::with_capacity(cap),
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.properties.is_empty() && self.settings.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.properties.len() + self.settings.len()
    }

    /// Sets a keyed value, called a "setting", in the PropSet. Returns the previous value if the
    /// key already existed.
    ///
    /// ```rust
    /// # use prosidy_ast::{Key, Text, PropSet};
    /// let mut props = PropSet::new();
    /// let key = Key::from("foo");
    /// let hello = Text::from("hello");
    /// let world = Text::from("world");
    /// assert_eq!(None, props.put(key.clone(), hello.clone()));
    /// assert_eq!(Some(hello), props.put(key, world));
    /// ```
    #[inline]
    pub fn put<K, V>(&mut self, key: K, value: V) -> Option<Text<'a>>
    where
        K: Into<Key>,
        V: Into<Text<'a>>,
    {
        self.settings.insert(key.into(), value.into())
    }

    /// Retrieves a setting from the PropSet and deletes it.
    ///
    /// ```
    /// # use prosidy_ast::{Key, Text, PropSet};
    /// let mut props = PropSet::new();
    /// let key = Key::from("foo");
    /// let val = Text::from("hello");
    /// assert_eq!(None, props.delete(&key));
    /// props.put(key.clone(), val.clone());
    /// assert_eq!(Some(val), props.delete(&key));
    /// assert_eq!(None, props.delete(&key));
    /// ```
    #[inline]
    pub fn delete<K: Borrow<Key>>(&mut self, key: K) -> Option<Text<'a>> {
        self.settings.remove(key.borrow())
    }

    /// Retrieves the setting associated with a key from the PropSet.
    /// ```rust
    /// # use prosidy_ast::{Key, Text, PropSet};
    /// let mut props = PropSet::new();
    /// let key = Key::from("foo");
    /// let val = Text::from("hello");
    /// assert_eq!(None, props.lookup(&key));
    /// props.put(key.clone(), val.clone());
    /// assert_eq!(Some(val.clone()), props.lookup(&key));
    /// // Repeatable, unlike `PropSet::delete`.
    /// assert_eq!(Some(val), props.lookup(&key));
    /// ```
    #[inline]
    pub fn lookup<K: Borrow<Key>>(&self, key: K) -> Option<Text<'a>> {
        self.settings.get(key.borrow()).cloned()
    }

    /// Sets a propperty on the PropSet which does not take a value. The namespaces for settings
    /// and properties does not overlap.
    /// ```rust
    /// # use prosidy_ast::{Key, PropSet};
    /// let mut props = PropSet::new();
    /// let key = Key::from("foo");
    /// props.set(key.clone());
    /// assert_eq!(None, props.lookup(&key));
    /// assert!(props.is_set(&key));
    /// ```
    #[inline]
    pub fn set<K: Into<Key>>(&mut self, key: K) {
        self.properties.insert(key.into());
    }

    /// Unsets a property on the PropSet. Returns whether or not the property had been previously
    /// set.
    /// ```rust
    /// # use prosidy_ast::{Key, PropSet};
    /// let mut props = PropSet::new();
    /// let key = Key::from("foo");
    /// assert_eq!(false, props.unset(&key));
    /// props.set(key.clone());
    /// assert_eq!(true, props.unset(&key));
    /// ```
    #[inline]
    pub fn unset<K: Borrow<Key>>(&mut self, key: K) -> bool {
        self.properties.remove(key.borrow())
    }

    /// Checks whether or not a the PropSet contains a property.
    /// ```rust
    /// # use prosidy_ast::{Key, PropSet};
    /// let mut props = PropSet::new();
    /// let key = Key::from("foo");
    /// assert_eq!(false, props.is_set(&key));
    /// props.set(key.clone());
    /// assert_eq!(true, props.is_set(&key));
    /// ```
    #[inline]
    pub fn is_set<K: Borrow<Key>>(&self, key: K) -> bool {
        self.properties.contains(key.borrow())
    }

    /// Iterates over every setting in the PropSet. The order of yielded settings is not
    /// guarenteed.
    /// ```rust
    /// # use prosidy_ast::{Key, PropSet, Text};
    /// let mut props = PropSet::new();
    /// let foo = Key::new("foo");
    /// let bar = Key::new("bar");
    /// let baz = Key::new("baz");
    /// props.put(foo.clone(), Text::from("foo-value"));
    /// props.put(bar.clone(), Text::from("bar-value"));
    /// props.put(baz.clone(), Text::from("baz-value"));
    /// let mut count = 0;
    /// for (k, v) in props.settings() {
    ///     count += 1;
    ///     assert!(v.starts_with(k.as_str()));
    ///     assert!(v.ends_with("-value"));
    ///     assert_eq!(v.len(), k.as_str().len() + "-value".len());
    /// }
    /// assert_eq!(count, 3);
    /// ```
    #[inline]
    pub fn settings<'r>(&'r self) -> impl 'r + Iterator<Item = (&'r Key, Text<'a>)> {
        self.settings.iter().map(|(k, v)| (k, v.clone()))
    }

    /// Iterates over every property in the PropSet. The order of yielded properties is not
    /// guarenteed.
    /// ```rust
    /// # use prosidy_ast::{Key, PropSet};
    /// let mut props = PropSet::new();
    /// let foo = Key::new("foo");
    /// let bar = Key::new("bar");
    /// let baz = Key::new("baz");
    /// props.set(foo.clone());
    /// props.set(bar.clone());
    /// props.set(baz.clone());
    /// for prop in props.properties() {
    ///     assert!(*prop == foo || *prop == bar || *prop == baz)
    /// }
    /// ```
    #[inline]
    pub fn properties<'r>(&'r self) -> impl 'r + Iterator<Item = &'r Key> {
        self.properties.iter()
    }

    /// Iterates over every property and every setting in the PropSet. No order is guarenteed.
    /// ```rust
    /// # use prosidy_ast::{Key, PropSet, Text};
    /// let mut props = PropSet::new();
    /// let foo = Key::new("foo");
    /// let bar = Key::new("bar");
    /// props.set(foo.clone());
    /// props.put(bar.clone(), Text::from("baz"));
    /// for (key, opt_val) in props.iter() {
    ///     if *key == foo {
    ///         assert_eq!(opt_val, None);
    ///     } else if *key == bar {
    ///         assert_eq!(opt_val, Some(Text::from("baz")));
    ///     } else {
    ///         panic!("there should only be two keys!");
    ///     }
    /// }
    /// ```
    #[inline]
    pub fn iter<'r>(&'r self) -> impl 'r + Iterator<Item = (&'r Key, Option<Text<'a>>)> {
        self.settings()
            .map(|(k, v)| (k, Some(v)))
            .chain(self.properties().map(|k| (k, None)))
    }
}

impl<'a> Debug for PropSet<'a> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        struct KV<'a, 'b>(&'b Key, &'b Text<'a>);
        impl<'a, 'b> Debug for KV<'a, 'b> {
            fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(fmt, "{:?} = {:?}", self.0.as_str(), self.1.as_str())
            }
        }
        let mut fmt = fmt.debug_list();
        for prop in self.properties.iter() {
            fmt.entry(&prop.as_str());
        }
        for (k, v) in self.settings.iter() {
            let kv = KV(k, v);
            fmt.entry(&kv);
        }
        fmt.finish()
    }
}

impl<'a> From<(HashSet<Key>, HashMap<Key, Text<'a>>)> for PropSet<'a> {
    fn from(pair: (HashSet<Key>, HashMap<Key, Text<'a>>)) -> Self {
        PropSet {
            properties: pair.0,
            settings: pair.1,
        }
    }
}
