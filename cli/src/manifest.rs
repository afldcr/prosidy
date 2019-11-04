/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;

use anyhow::Result;
use futures::prelude::*;
use prosidy::{PropSet, Text};
use prosidy::parse::parse_meta;
use prosidy::xml::{XML, quick_xml::Result as XMLResult};
use prosidy::xml::quick_xml::events::{Event, BytesStart, BytesEnd};
use serde::Serialize;
use serde::ser::Serializer;
use tokio_fs as fs;

#[derive(Debug, Serialize)]
/// A collection of [`PropSet`] nodes extracted from the headers of each Prosidy file in a directory.
pub struct Manifest(HashMap<PathBuf, Entry>);

impl Manifest {
    const TAG_MANIFEST: &'static str = "prosidy:manifest";
    const TAG_ITEM: &'static str = "prosidy:item";
    const ATTR_PATH: &'static str = "prosidy:path";

    pub async fn read<P: AsRef<Path>>(path: P, follow_symlinks: bool) -> Result<Manifest> {
        let root_path: Arc<Path> = Arc::from(path.as_ref().canonicalize()?);
        anyhow::ensure!(root_path.is_dir(), "Manifests can only be read from a directory");
        log::info!("reading manifest from {:?}", root_path);
        let dir_entries = fs::read_dir(Arc::clone(&root_path)).await?;
        let map = dir_entries
            .try_filter_map(|dir_entry| entry_path(dir_entry, follow_symlinks))
            .map_err(anyhow::Error::from)
            .try_filter_map(|path| {
                let root_path = Arc::clone(&root_path);
                async move {
                    let opt_entry = Entry::try_read(&path).await?;
                    Ok(opt_entry.map(|entry| {
                        let rel_path = path.strip_prefix(root_path)
                            .expect("all paths to be a child of the root path")
                            .to_path_buf();
                        (rel_path, entry)
                    }))
                }
            })
            .try_collect()
            .await?;
        Ok(Manifest(map))
    }
}

impl XML for Manifest {
    fn to_events<F>(&self, emit: &mut F) -> XMLResult<()>
    where
        F: for<'a> FnMut(Event<'a>) -> XMLResult<()>
    {
        let start = BytesStart::borrowed_name(Manifest::TAG_MANIFEST.as_bytes());
        emit(Event::Start(start))?;
        for (path, entry) in self.0.iter() {
            let mut start = BytesStart::borrowed_name(Manifest::TAG_ITEM.as_bytes());
            let path_str = path.to_string_lossy();
            start.push_attribute((Manifest::ATTR_PATH, path_str.as_ref()));
            for (name, opt_val) in entry.props.iter() {
                let val = opt_val.unwrap_or(Text::EMPTY);
                start.push_attribute((name.as_str(), val.as_str()));
            }
            emit(Event::Empty(start))?;
        }
        let end = BytesEnd::borrowed(Manifest::TAG_MANIFEST.as_bytes());
        emit(Event::End(end))
    }
}

#[derive(Debug)]
pub struct Entry {
    source: Pin<Arc<str>>,
    props: PropSet<'static>,
}

impl Entry {
    pub async fn try_read<P: AsRef<Path>>(path: P) -> Result<Option<Entry>> {
        let path = path.as_ref();
        log::info!("reading the header of {:?}", path);
        // Reads the filepath into a string, then wraps it in an atomic reference counter & pins
        // it. This lets us treat `parsed` as owned by an Entry via coercing it's lifetime.
        let bytes = fs::read(path).await?;
        let string = String::from_utf8(bytes)?;
        let source = Pin::new(Arc::from(string));
        // Read metadata from the source string. It will be returned with the anonymous lifetime
        // which can't be kept past this functions end, so we'll transmute it into the correct
        // lifetime.
        let parsed: PropSet<'_> = match parse_meta(&source) {
            Ok(parsed) => parsed,
            Err(err) => {
                log::warn!("Failed to parse {:?} as a Prosidy file: {}", path, err);
                return Ok(None);
            },
        };
        let props: PropSet<'static> = unsafe { std::mem::transmute(parsed) };
        Ok(Some(Entry { source, props }))
    }
}

impl Serialize for Entry {
    fn serialize<S: Serializer>(&self, ser: S) -> std::result::Result<S::Ok, S::Error> {
        self.props.serialize(ser)
    }
}

async fn entry_path(dir_entry: fs::DirEntry, follow_symlinks: bool) -> std::io::Result<Option<PathBuf>> {
    let file_type = dir_entry.file_type().await?;
    let path = if file_type.is_file() {
        Some(dir_entry.path())
    } else if follow_symlinks && file_type.is_symlink() {
        let path = dir_entry.path().to_path_buf();
        if path.extension() == Some("pro".as_ref()) && path.canonicalize()?.is_file() {
            Some(path)
        } else {
            None
        }
    } else {
        None
    };
    Ok(path)
}
