/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::fs::File;
use std::io::{self, Read, Stdin, Stdout, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Arg, ArgGroup, App, ArgMatches};

use crate::args::FromArgs;

const ARG_INPUT: &str = "input-path";
const ARG_OUTPUT: &str = "output-path";
const ARG_STDIN: &str = "stdin";
const GROUP_INPUT: &str = "input";

#[derive(Debug)]
pub struct IOOpts {
    input: Option<PathBuf>,
    output: Option<PathBuf>,
}

impl FromArgs for IOOpts {
    fn register_args<'a, 'b>(args: App<'a, 'b>) -> App<'a, 'b> {
        let input = Arg::with_name(ARG_INPUT)
            .help("A filepath which will be read as a Prosidy document");
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

#[derive(Debug)]
pub enum Input<'a> {
    StdIO(Stdin),
    File(&'a Path, File),
}

impl<'a> Input<'a> {
    pub fn new<P: AsRef<Path>>(opt_path: Option<&'a P>) -> Result<Self> {
        opt_path
            .map(Input::open)
            .unwrap_or_else(|| Ok(Input::stdio()))
    }

    pub fn stdio() -> Self {
        Input::StdIO(io::stdin())
    }

    pub fn open<P: AsRef<Path>>(path: &'a P) -> Result<Self> {
        let path: &'a Path = path.as_ref();
        let file = File::open(path).with_context(|| {
            format! {
                "failed to open {:?} in read mode",
                path,
            }
        })?;
        Ok(Input::File(path, file))
    }

    pub fn contents(&mut self) -> Result<String> {
        let string_length = self.filesize()?.unwrap_or(1024);
        let mut buf = String::with_capacity(string_length);
        self.read_to_string(&mut buf)?;
        buf.shrink_to_fit();
        Ok(buf)
    }

    pub fn filesize(&self) -> Result<Option<usize>> {
        if let Input::File(path, file) = self {
            let metadata = file.metadata().with_context(|| {
                format! {
                    "failed to read filesystem metadata from {:?}",
                    path,
                }
            })?;
            Ok(Some(metadata.len() as usize))
        } else {
            Ok(None)
        }
    }
}

impl<'a> Read for Input<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Input::StdIO(stdin) => stdin.read(buf),
            Input::File(_, file) => file.read(buf),
        }
    }
}

#[derive(Debug)]
pub enum Output<'a> {
    StdIO(Stdout),
    File(&'a Path, File),
}

impl<'a> Output<'a> {
    pub fn new<P: AsRef<Path>>(opt_path: Option<&'a P>) -> Result<Self> {
        opt_path
            .map(Output::open)
            .unwrap_or_else(|| Ok(Output::stdio()))
    }

    pub fn stdio() -> Self {
        Output::StdIO(io::stdout())
    }

    pub fn open<P: AsRef<Path>>(path: &'a P) -> Result<Self> {
        let path: &'a Path = path.as_ref();
        let file = File::create(path).with_context(|| {
            format! {
                "failed to open {:?} in write mode",
                path,
            }
        })?;
        Ok(Output::File(path, file))
    }
}

impl<'a> Write for Output<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Output::StdIO(stdout) => stdout.write(buf),
            Output::File(_, file) => file.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Output::StdIO(stdout) => stdout.flush(),
            Output::File(_, file) => file.flush(),
        }
    }
}
