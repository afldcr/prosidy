/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use anyhow::Result;
use clap::{App, ArgMatches};

pub trait FromArgs: Sized {
    fn register_args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b>;
    fn parse_args<'a>(matches: &ArgMatches<'a>) -> Result<Self>;
}

pub trait AppExt<'a, 'b>: Sized {
    fn register<T: FromArgs>(self) -> App<'a, 'b>;
}

impl<'a, 'b> AppExt<'a, 'b> for App<'a, 'b> {
    fn register<T: FromArgs>(self) -> App<'a, 'b> {
        T::register_args(self)
    }
}
