/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

macro_rules! http_error {
    ($($name:ident : $code:expr),* $(,)?) => {
        $(
            pub fn $name () -> Result<hyper::Response<hyper::Body>, hyper::http::Error> {
                let body: &'static [u8] = include_bytes!(concat!(
                    stringify!($code), ".xml",
                ));
                hyper::Response::builder()
                    .status($code)
                    .header(hyper::header::CONTENT_TYPE, mime::TEXT_XML.type_().as_str())
                    .body(hyper::Body::from(body))
        })*
    }
}

http_error! {
    bad_request: 400,
    forbidden: 403,
    not_found: 404,
    menthod_not_allowed: 405,
    internal_server_error: 500,
}
