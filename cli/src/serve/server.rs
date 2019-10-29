/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::borrow::Cow;
use std::sync::Arc;
use std::result::{Result as StdResult};
use std::path::{Path, PathBuf};
use std::cmp::Ordering;
use std::time::Instant;

use anyhow::Result;
use hyper::{header, Request, Response, Body, Method, Server};
use hyper::http::{Error as HttpError, response::Builder};
use hyper::service::{make_service_fn, service_fn};
use hyper::server::conn::AddrStream;
use mime::Mime;
use tokio::runtime::Runtime;
use tokio::prelude::*;

use crate::fmt::FormatKind;
use crate::mediatype::{CBOR, infer_media_type};
use super::opts::ServeOpts;
use super::http_error::*;
use super::cache::handle_caching;

pub fn serve(opts: Arc<ServeOpts>) -> Result<()> {
    let addr = opts.address();
    let make_service = make_service_fn(|stream: &AddrStream| {
        let addr = stream.remote_addr();
        let opts = opts.clone();
        async move {
            let service = service_fn(move |req: Request<Body>| {
                let start = Instant::now();
                let method = req.method().clone();
                let uri = req.uri().clone();
                let opts = opts.clone();
                handle(opts, req).map(move |result| {
                    let dur = Instant::now() - start;
                    match result {
                        Ok(response) => {
                            log::info!(
                                "addr={addr:?}\nmethod={method:?}\nuri={uri:?}\nduration={dur:?}\nstatus={status:}",
                                addr=addr,
                                method=method,
                                uri=uri,
                                dur=dur,
                                status=response.status(),
                            );
                            Ok(response)
                        }
                        Err(error) => {
                            log::error!(
                                "addr={addr:?}\nmethod={method:?}\nuri={uri:?}\nduration={dur:?}\nerror={err:}",
                                addr=addr,
                                method=method,
                                uri=uri,
                                dur=dur,
                                err=error,
                            );
                            internal_server_error()
                        }
                    }
                })
            });
            Ok::<_, HttpError>(service)
        }
    });
    let server = Server::bind(&addr).serve(make_service);
    let rt = Runtime::new()?;
    rt.block_on(server)?;
    Ok(())
}

macro_rules! handle {
    ($e:expr) => {
        match $e {
            Ok(ok) => ok,
            Err(err) => return err,
        }
    }
}

async fn handle(opts: Arc<ServeOpts>, request: Request<Body>) -> Result<Response<Body>> {
    handle! { check_method(&request) };
    let path = handle! {
        normalize_path(
            opts.follow_symlinks,
            &opts.root_path,
            request.uri().path(),
        )
    };
    let bytes = tokio::fs::read(&path).await?;
    let mut builder = Response::builder();
    // add caching metadata, if caching is enabled
    if let Some(ref opts) = opts.cache_opts {
        handle!(handle_caching(&request, opts, &mut builder, &bytes));
    }
    // check the extension for how to respond
    if path.extension() == Some("pro".as_ref()) {
        handle_prosidy(&request, builder, opts, bytes)
    } else {
        let mime = infer_media_type(&path);
        builder
            .header(header::CONTENT_TYPE, mime.as_ref())
            .body(bytes.into())
            .map_err(anyhow::Error::from)
    }
}

fn handle_prosidy(request: &Request<Body>, mut builder: Builder, opts: Arc<ServeOpts>, bytes: Vec<u8>) -> Result<Response<Body>> {
    let source = String::from_utf8(bytes)?;
    let doc = prosidy::parse::parse_document(&source)?;

    let mut output = Vec::with_capacity(8192);
    let format = determine_format(request);
    format.write(&opts.format, &mut output, &doc)?;
    builder
        .header(header::CONTENT_TYPE, format.media_type().as_ref())
        .body(output.into())
        .err_into()
}

fn determine_format(request: &Request<Body>) -> FormatKind {
    determine_format_from_params(request)
        .or_else(|| determine_format_from_headers(request))
        .unwrap_or(FormatKind::XML)
}

fn determine_format_from_headers(request: &Request<Body>) -> Option<FormatKind> {
    let accept = request.headers().get(header::ACCEPT)?.to_str().ok()?;
    log::debug!("Reading format types from ACCEPT header: {:?}", accept);
    accept
        // Split the header on commas, then try to parse each segment as a mime type
        .split(',').flat_map(|raw| raw.trim_start().parse::<Mime>().ok())
        // Then, try to match the mime type and subtype against supported formats,
        // attaching the quality if one is set.
        .flat_map(|mime| {
            let kind = match mime.type_() {
                mime::APPLICATION => match mime.subtype() {
                    mime::JSON => Some(FormatKind::JSON),
                    mime::XML => Some(FormatKind::XML),
                    other if other == *CBOR => Some(FormatKind::CBOR),
                    _ => None,
                }
                mime::TEXT => match mime.subtype() {
                    mime::XML => Some(FormatKind::XML),
                    _ => None,
                },
                _ => None,
            }?;
            // https://developer.mozilla.org/en-US/docs/Glossary/Quality_values
            let quality = mime.get_param("q")
                .and_then(|q| q.as_ref().parse::<f64>().ok())
                .unwrap_or(1.0);
            Some((kind, quality))
        })
        .max_by(|(_, q1), (_, q2)| {
            q1.partial_cmp(q2).unwrap_or(Ordering::Equal)
        })
        .map(|(kind, _)| kind)
}

fn determine_format_from_params(request: &Request<Body>) -> Option<FormatKind> {
    let query = request.uri().query()?;
    log::debug!("Reading format types from query params: {:?}", query);
    query
        .split('&')
        .map(|s| if s.chars().any(char::is_uppercase) {
            Cow::Owned(s.to_lowercase())
        } else {
            Cow::Borrowed(s)
        })
        .flat_map(|s| match s.as_ref() {
            "cbor" => Some(FormatKind::CBOR),
            "json" => Some(FormatKind::JSON),
            "xml"  => Some(FormatKind::XML),
            _      => None
        })
        .next()
}

fn check_method(request: &Request<Body>) -> Handle<()> {
    if request.method() == Method::GET {
        Ok(())
    } else {
        Err(menthod_not_allowed().err_into())
    }
}


fn normalize_path(follow: bool, root: &Path, path_str: &str) -> Handle<PathBuf> {
    use std::io::ErrorKind::*;
    use std::path::Component::*;
    let mut buf = PathBuf::new();
    for part in Path::new(path_str.trim_start_matches('/')).components() {
        match part {
            CurDir | ParentDir if buf.pop() => {},
            Normal(path) => buf.push(path),
            _ => return Err(bad_request().err_into()),
        }
    }
    let full = root.join(buf);
    let canon = full.canonicalize().map_err(|e| match e.kind() {
        NotFound => not_found().err_into(),
        PermissionDenied => forbidden().err_into(),
        _ => Err(e).err_into(),
    })?;
    if (follow || canon == full) && canon.is_file() {
        Ok(canon)
    } else {
        Err(not_found().err_into())
    }
}

type Handle<T> = StdResult<T, Result<Response<Body>>>;

trait ResultExt<T> {
    fn err_into(self) -> Result<T>;
}

impl<T, E> ResultExt<T> for StdResult<T, E>
where
    E: 'static + std::error::Error + Send + Sync,
{
    fn err_into(self) -> Result<T> {
        self.map_err(anyhow::Error::from)
    }
}

#[test]
fn auto_format_default() {
    let req = Request::new(Body::default());
    assert_eq!(
        FormatKind::XML,
        determine_format(&req),
        "if no format is specified, default to XML",
    );
}

#[test]
fn auto_format_accept() {
    let req = Request::builder()
        .header(header::ACCEPT, "text/html;q=1, text/xml;q=0.5, application/json;q=0.9")
        .body(Body::default()).unwrap();
    assert_eq!(
        FormatKind::JSON,
        determine_format(&req),
        "if an ACCEPT header is provided, provide the supported format with the highest quality",
    );
}

#[test]
fn auto_format_params() {
    let req = Request::builder()
        .uri("/?nonsense&cbor&json")
        .body(Body::default()).unwrap();
    assert_eq!(
        FormatKind::CBOR,
        determine_format(&req),
        "if query parameters are provided with a format, return the first supported format",
    );
}

#[test]
fn auto_format_accept_and_params() {
    let req = Request::builder()
        .header(header::ACCEPT, "text/html;q=1, text/xml;q=0.5, application/json;q=0.9")
        .uri("/?nonsense&cbor&json")
        .body(Body::default()).unwrap();
    assert_eq!(
        FormatKind::CBOR,
        determine_format(&req),
        "query parameters take precedence over the ACCEPT header (browsers send ACCEPT by default)",
    );
}
