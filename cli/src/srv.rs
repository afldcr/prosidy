/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::path::Path;
use std::net::SocketAddr;

use tokio::prelude::*;
use hyper::rt;
use hyper::{Body, Method, Request, Response, Server, StatusCode};

pub struct Config<'a> {
    addr: SocketAddr,
    root: &'a Path,
}

impl<'a> Config<'a> {
    pub fn new<P: AsRef<Path>>(addr: SocketAddr, root: &'a P) -> Self {
        let root = root.as_ref();
        Config { addr, root }
    }

    pub fn serve(self) {
        let server = Server::bind(&self.addr)
            .serve(|| hyper::service::service_fn_ok(handler))
            .map_err(|e| log::error!("server error: {}", e));
        rt::run(server);
    }
}

fn handler(req: Request<Body>) -> Response<Body> {
    Response::new(Body::from("jafc"))
}
