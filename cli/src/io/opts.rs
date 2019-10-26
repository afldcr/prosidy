/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::path::PathBuf;

use anyhow::Result;
use clap::{App, Arg, ArgGroup, ArgMatches};

use super::sync::{Input, Output};
use crate::args::FromArgs;

#[derive(Debug)]
pub struct IOOpts {
    input: Option<PathBuf>,
    output: Option<PathBuf>,
}

impl FromArgs for IOOpts {
    fn register_args<'a, 'b>(args: App<'a, 'b>) -> App<'a, 'b> {
        let input =
            Arg::with_name(ARG_INPUT).help("A filepath which will be read as a Prosidy document");
        let output = Arg::with_name(ARG_OUTPUT)
            .help("A filepath where output will be written to")
            .long("out")
            .short("o")
            .value_name("OUTPUT PATH");
        let stdin = Arg::with_name(ARG_STDIN)
            .help("Read a Prosidy document from standard input rather than a file")
            .long("stdin");
        let input_group = ArgGroup::with_name(GROUP_INPUT)
            .args(&[ARG_INPUT, ARG_STDIN])
            .required(true);
        args.arg(input).arg(output).arg(stdin).group(input_group)
    }

    fn parse_args(matches: &ArgMatches) -> Result<Self> {
        let input = if matches.is_present(ARG_STDIN) {
            None
        } else if let Some(path) = matches.value_of(ARG_INPUT) {
            Some(path.into())
        } else {
            anyhow::bail!("Missing input path");
        };
        let output = matches.value_of(ARG_OUTPUT).map(PathBuf::from);
        Ok(IOOpts { input, output })
    }
}

impl IOOpts {
    pub fn input(&self) -> Result<Input> {
        Input::new(self.input.as_ref())
    }

    pub fn output(&self) -> Result<Output> {
        Output::new(self.output.as_ref())
    }
}

const ARG_INPUT: &str = "input-path";
const ARG_OUTPUT: &str = "output-path";
const ARG_STDIN: &str = "stdin";
const GROUP_INPUT: &str = "input";
