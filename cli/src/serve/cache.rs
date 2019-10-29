

use hyper::http::header::{self, HeaderValue};
use hyper::http::response::Builder;
use hyper::{Body, Response, Request, StatusCode};
use sha2::{Digest, Sha256};

use super::opts::CacheOpts;

pub fn handle_caching(
    request: &Request<Body>,
    opts: &CacheOpts,
    builder: &mut Builder,
    data: &[u8],
) -> Result<(), anyhow::Result<Response<Body>>> {
    let mut hash = [0; 43];
    hash_bytes_b64(&mut hash, data);
    check_etag(request, &hash)?;
    let etag = HeaderValue::from_bytes(&hash)
        .map_err(|e| Err(e.into()))?;
    let cache_control = if opts.validate {
        HeaderValue::from_str(&format!("max-age={}, no-cache", opts.max_age))
    } else {
        HeaderValue::from_str(&format!("max-age={}", opts.max_age))
    }.map_err(|e| Err(e.into()))?;
    builder
        .header(header::ETAG, etag)
        .header(header::CACHE_CONTROL, cache_control);
    Ok(())
}

fn check_etag(request: &Request<Body>, hash: &[u8])  -> Result<(), anyhow::Result<Response<Body>>> {
    if let Some(prev_hash) = request.headers().get(header::IF_NONE_MATCH) {
        if prev_hash.as_bytes() == hash {
            return Err(Response::builder()
                .status(StatusCode::NOT_MODIFIED)
                .body(Body::default())
                .map_err(anyhow::Error::from));
        }
    }
    Ok(())
}

fn hash_bytes_b64(buf: &mut [u8; 43], data: &[u8]) {
    let mut digest = Sha256::new();
    digest.input(env!("CARGO_PKG_NAME"));
    digest.input(b"\0");
    digest.input(env!("CARGO_PKG_VERSION"));
    digest.input(b"\0");
    digest.input(&data);
    let hash = digest.result();
    base64::encode_config_slice(&hash, base64::STANDARD_NO_PAD, buf);
}
