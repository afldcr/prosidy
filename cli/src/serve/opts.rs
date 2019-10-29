/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::net::{SocketAddr, IpAddr};
use std::path::PathBuf;

use anyhow::Result;
use clap::{App, Arg, ArgMatches, value_t};

use crate::args::{AppExt, FromArgs};
use crate::fmt::FormatOpts;

#[derive(Debug)]
pub struct ServeOpts {
    pub cache_opts: Option<CacheOpts>,
    pub listen_address: IpAddr,
    pub listen_port: u16,
    pub follow_symlinks: bool,
    pub format: FormatOpts,
    pub root_path: PathBuf,
}

impl ServeOpts {
    pub fn run(self) -> Result<()> {
        super::server::serve(self.into())
    }

    pub fn address(&self) -> SocketAddr {
        SocketAddr::new(self.listen_address, self.listen_port)
    }
}

impl FromArgs for ServeOpts {
    fn register_args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
        let address = Arg::with_name(ARG_ADDRESS)
            .help("Set the server to listen only to the specified interface")
            .long("address")
            .short("a")
            .value_name("IP ADDRESS")
            .default_value("127.0.0.1");
        let port = Arg::with_name(ARG_PORT)
            .help("Set the server to listen only to the specified port")
            .long("port")
            .short("P")
            .value_name("PORT")
            .default_value("7080");
        let root_path = Arg::with_name(ARG_ROOT_PATH)
            .help("Serve files from this path")
            .value_name("ROOT DIR")
            .required(true);
        let follow_symlinks = Arg::with_name(ARG_FOLLOW_SYMLINKS)
            .help("Follow symlinks when serving files")
            .long("follow")
            .short("F")
            .takes_value(false);
        app.args(&[
            address,
            port,
            follow_symlinks,
            root_path,
        ]).register::<FormatOpts>().register::<Option<CacheOpts>>()
    }

    fn parse_args(matches: &ArgMatches) -> Result<Self> {
        let listen_address = value_t!(matches, ARG_ADDRESS, IpAddr)?;
        let listen_port = value_t!(matches, ARG_PORT, u16)?;
        let root_path = value_t!(matches, ARG_ROOT_PATH, PathBuf)?.canonicalize()?;
        let format = FormatOpts::parse_args(matches)?;
        let follow_symlinks = matches.is_present(ARG_FOLLOW_SYMLINKS);
        let cache_opts = Option::parse_args(matches)?;
        Ok(ServeOpts {
            cache_opts,
            listen_address,
            listen_port,
            follow_symlinks,
            format,
            root_path,
        })
    }
}

#[derive(Debug)]
pub struct CacheOpts {
    pub max_age: u64,
    pub validate: bool,
}

impl FromArgs for Option<CacheOpts> {
    fn register_args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
        let enable = Arg::with_name(ARG_CACHE)
            .help("Enable HTTP caching")
            .long("cache")
            .short("c");
        let validate = Arg::with_name(ARG_CACHE_VALIDATE)
            .help("Force clients to re-validate cached entries")
            .long("validate")
            .requires(ARG_CACHE);
        let max_age = Arg::with_name(ARG_CACHE_MAX_AGE)
            .help("Sets the maximum cache period for served documents.")
            .long("max-age")
            .requires(ARG_CACHE)
            .default_value_if(ARG_CACHE, None, "3600")
            .value_name("SECONDS");
        app.args(&[
            enable,
            max_age,
            validate,
        ])
    }

    fn parse_args(matches: &ArgMatches) -> Result<Self> {
        if matches.is_present(ARG_CACHE) {
            let max_age = value_t!(matches, ARG_CACHE_MAX_AGE, u64)?;
            let validate = matches.is_present(ARG_CACHE_VALIDATE);
            Ok(Some(CacheOpts { max_age, validate }))
        } else {
            Ok(None)
        }
    }
}

const ARG_ADDRESS: &str = "ip";
const ARG_PORT: &str = "port";
const ARG_ROOT_PATH: &str = "root-path";
const ARG_FOLLOW_SYMLINKS: &str = "follow-symlinks";

const ARG_CACHE: &str = "cache";
const ARG_CACHE_MAX_AGE: &str = "cache-max-age";
const ARG_CACHE_VALIDATE: &str = "cache-validate";
