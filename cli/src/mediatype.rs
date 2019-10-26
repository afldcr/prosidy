use std::path::Path;
use std::ops::Deref;

use lazy_static::lazy_static;
use mime::{Name, Mime};
use phf::{Map, phf_map};

use StaticMime::*;

pub fn infer_media_type<P: AsRef<Path>>(path: P) -> &'static Mime {
    let path = path.as_ref();
    path.extension()
        .and_then(|ext| ext.to_str())
        .and_then(|ext| MEDIA_TYPES.get(ext))
        .map(|mime| mime.deref())
        .unwrap_or(DEFAULT_MEDIA_TYPE)
}

const DEFAULT_MEDIA_TYPE: &'static Mime = &mime::APPLICATION_OCTET_STREAM;

static MEDIA_TYPES: Map<&'static str, StaticMime> = phf_map! {
    "css"   => Static(mime::TEXT_CSS),
    "gif"   => Static(mime::IMAGE_GIF),
    "html"  => Static(mime::TEXT_HTML),
    "jpeg"  => Static(mime::IMAGE_JPEG),
    "jpg"   => Static(mime::IMAGE_JPEG),
    "js"    => Static(mime::APPLICATION_JAVASCRIPT),
    "json"  => Static(mime::APPLICATION_JSON),
    "pdf"   => Static(mime::APPLICATION_PDF),
    "png"   => Static(mime::IMAGE_PNG),
    "svg"   => Static(mime::IMAGE_SVG),
    "txt"   => Static(mime::TEXT_PLAIN),
    "woff"  => Static(mime::FONT_WOFF),
    "woff2" => Static(mime::FONT_WOFF2),
    "xml"   => Static(mime::TEXT_XML),
    "xsd"   => Static(mime::TEXT_XML),

    "cbor"  => Lazy(&APPLICATION_CBOR),
    "xsl"   => Lazy(&APPLICATION_XSLT),
    "xslt"  => Lazy(&APPLICATION_XSLT),
};

lazy_static! {
    pub static ref APPLICATION_CBOR: Mime = {
        "application/cbor".parse::<Mime>().expect("Failed to instantiate media type")
    };

    pub static ref APPLICATION_XSLT: Mime = {
        "application/xslt+xml".parse::<Mime>().expect("Failed to instantiate media type")
    };

    pub static ref CBOR: Name<'static> = {
        APPLICATION_CBOR.subtype()
    };
}

enum StaticMime {
    Static(Mime),
    Lazy(&'static (dyn Deref<Target=Mime> + Send + Sync)),
}

impl Deref for StaticMime {
    type Target = Mime;
    fn deref(&self) -> &Mime {
        match self {
            Static(mime) => mime,
            Lazy(lazy) => &lazy,
        }
    }
}
