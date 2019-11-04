/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::collections::HashMap;
use std::fs::{self, FileType};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;

use anyhow::Result;
use prosidy::parse::parse_meta;
use prosidy::xml::quick_xml::events::{BytesEnd, BytesStart, Event};
use prosidy::xml::{quick_xml::Result as XMLResult, XML};
use prosidy::{PropSet, Text};
use serde::ser::Serializer;
use serde::Serialize;

#[cfg(feature = "server")]
use futures::prelude::*;
#[cfg(feature = "server")]
use tokio_fs as tfs;

#[derive(Debug, Serialize)]
/// A collection of [`PropSet`] nodes extracted from the headers of each Prosidy file in a directory.
pub struct Manifest(HashMap<PathBuf, Entry>);

impl Manifest {
    const TAG_MANIFEST: &'static str = "prosidy:manifest";
    const TAG_ITEM: &'static str = "prosidy:item";
    const ATTR_PATH: &'static str = "prosidy:path";

    pub fn read<P: AsRef<Path>>(path: P, follow_symlinks: bool) -> Result<Manifest> {
        let root_path: Arc<Path> = Arc::from(path.as_ref().canonicalize()?);
        anyhow::ensure!(
            root_path.is_dir(),
            "Manifests can only be read from a directory"
        );
        log::info!("reading manifest from {:?}", root_path);
        let dir_entries = fs::read_dir(Arc::clone(&root_path))?;
        let map: Result<HashMap<PathBuf, Entry>> = dir_entries
            .map(|dir_entry| {
                let dir_entry = dir_entry.map_err(anyhow::Error::from)?;
                let file_type = dir_entry.file_type()?;
                let path = dir_entry.path();
                if let Some(resolved) = entry_path(file_type, path, follow_symlinks)? {
                    if let Some(entry) = Entry::try_read(&resolved)? {
                        let rel_path = resolved
                            .strip_prefix(&root_path)
                            .expect("all paths to be a child of the root path")
                            .to_path_buf();
                        return Ok(Some((rel_path, entry)));
                    }
                }
                Ok(None) as Result<Option<_>>
            })
            .flat_map(Result::transpose)
            .collect();
        Ok(Manifest(map?))
    }

    #[cfg(feature = "server")]
    pub async fn read_async<P: AsRef<Path>>(path: P, follow_symlinks: bool) -> Result<Manifest> {
        let root_path: Arc<Path> = Arc::from(path.as_ref().canonicalize()?);
        anyhow::ensure!(
            root_path.is_dir(),
            "Manifests can only be read from a directory"
        );
        log::info!("reading manifest from {:?}", root_path);
        let dir_entries = tfs::read_dir(Arc::clone(&root_path)).await?;
        let map = dir_entries
            .try_filter_map(|dir_entry| {
                async move {
                    let file_type = dir_entry.file_type().await?;
                    let path = dir_entry.path();
                    entry_path(file_type, path, follow_symlinks)
                }
            })
            .map_err(anyhow::Error::from)
            .try_filter_map(|path| {
                let root_path = Arc::clone(&root_path);
                async move {
                    let opt_entry = Entry::try_read_async(&path).await?;
                    Ok(opt_entry.map(|entry| {
                        let rel_path = path
                            .strip_prefix(root_path)
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
        F: for<'a> FnMut(Event<'a>) -> XMLResult<()>,
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
    pub fn try_read<P: AsRef<Path>>(path: P) -> Result<Option<Entry>> {
        let path = path.as_ref();
        log::info!("reading the header of {:?}", path);
        // Reads the filepath into a string, then wraps it in an atomic reference counter & pins
        // it. This lets us treat `parsed` as owned by an Entry via coercing it's lifetime.
        let string = fs::read_to_string(path)?;
        let source = Pin::new(Arc::from(string));
        // Read metadata from the source string. It will be returned with the anonymous lifetime
        // which can't be kept past this functions end, so we'll transmute it into the correct
        // lifetime.
        let parsed: PropSet<'_> = match parse_meta(&source) {
            Ok(parsed) => parsed,
            Err(err) => {
                log::warn!("Failed to parse {:?} as a Prosidy file: {}", path, err);
                return Ok(None);
            }
        };
        let props: PropSet<'static> = unsafe { std::mem::transmute(parsed) };
        Ok(Some(Entry { source, props }))
    }

    #[cfg(feature = "server")]
    pub async fn try_read_async<P: AsRef<Path>>(path: P) -> Result<Option<Entry>> {
        let path = path.as_ref();
        log::info!("reading the header of {:?}", path);
        // Reads the filepath into a string, then wraps it in an atomic reference counter & pins
        // it. This lets us treat `parsed` as owned by an Entry via coercing it's lifetime.
        let bytes = tfs::read(path).await?;
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
            }
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

fn entry_path(
    file_type: FileType,
    path: PathBuf,
    follow_symlinks: bool,
) -> std::io::Result<Option<PathBuf>> {
    let path = if file_type.is_file() {
        Some(path)
    } else if follow_symlinks && file_type.is_symlink() {
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
