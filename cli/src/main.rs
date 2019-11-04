/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::io::Write;
use std::path::PathBuf;

use anyhow::Result;
use clap::{value_t, App, AppSettings, Arg, ArgMatches, SubCommand};
use log::LevelFilter;

use self::args::{AppExt, FromArgs};
use self::serve::ServeOpts;

fn main() {
    let app = App::new(env!("CARGO_PKG_NAME"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::ColorAuto)
        .register::<Opts>();
    let matches = app.clone().get_matches();
    let opts = Opts::parse_args(&matches).unwrap_or_else(|error| {
        let stderr = std::io::stderr();
        let mut out = stderr.lock();
        writeln!(&mut out, "{}", error).unwrap();
        app.write_help(&mut out).unwrap();
        out.write_all(b"\n").unwrap();
        std::process::exit(1);
    });
    if let Err(e) = opts.run(app) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

#[derive(Debug)]
struct Opts {
    log_level: LevelFilter,
    mode: Mode,
}

impl Opts {
    const ARG_LOG_LEVEL: &'static str = "log-level";

    fn run(self, app: App) -> Result<()> {
        let _ = env_logger::builder()
            .filter_module("prosidy", self.log_level)
            .filter_module("prosidy_ast", self.log_level)
            .filter_module("prosidy_cli", self.log_level)
            .filter_module("prosidy_parse", self.log_level)
            .try_init();
        log::debug!("Initialized logger with level {:?}", self.log_level);
        log::debug!("Options: {:?}", self);
        self.mode.run(app)
    }
}

impl FromArgs for Opts {
    fn register_args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
        let arg = Arg::with_name(Opts::ARG_LOG_LEVEL)
            .help("Set the threshold for log messages printed to stderr")
            .long("log-level")
            .short("l")
            .global(true)
            .default_value("warn")
            .possible_values(&["trace", "debug", "info", "warn", "error", "off"]);
        app.arg(arg).register::<Mode>()
    }

    fn parse_args(matches: &ArgMatches) -> Result<Self> {
        let log_level = value_t!(matches, Opts::ARG_LOG_LEVEL, LevelFilter)?;
        let mode = Mode::parse_args(matches)?;
        Ok(Opts { log_level, mode })
    }
}

#[derive(Debug)]
enum Mode {
    Compile(Compile),
    Completions(Completions),
    Manifest(Manifest),
    Serve(ServeOpts),
}

impl Mode {
    const COMPILE: &'static str = "compile";
    const COMPLETIONS: &'static str = "generate-completions";
    const MANIFEST: &'static str = "manifest";
    const SERVE: &'static str = "serve";

    fn run(self, app: App) -> Result<()> {
        match self {
            Mode::Compile(compile) => compile.run(),
            Mode::Completions(complete) => complete.run(app),
            Mode::Manifest(manifest) => manifest.run(),
            Mode::Serve(serve) => serve.run(),
        }
    }
}

impl FromArgs for Mode {
    fn register_args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
        let compile = SubCommand::with_name(Mode::COMPILE)
            .about("Parse a Prosidy document into an AST")
            .register::<Compile>();
        let generate_completions = SubCommand::with_name(Mode::COMPLETIONS)
            .about("Generate completions for the Prosidy CLI tool")
            .setting(AppSettings::Hidden)
            .register::<Completions>();
        let manifest = SubCommand::with_name(Mode::MANIFEST)
            .about("Parse the metadata of a document or directory of documents")
            .register::<Manifest>();
        let serve = SubCommand::with_name(Mode::SERVE)
            .about("Serve Prosidy documents over HTTP")
            .register::<ServeOpts>();
        app
            .subcommand(compile)
            .subcommand(generate_completions)
            .subcommand(manifest)
            .subcommand(serve)
    }

    fn parse_args(matches: &ArgMatches) -> Result<Self> {
        let (sub, sub_matches) = matches.subcommand();
        match sub {
            Mode::COMPILE => {
                let compile = Compile::parse_args(sub_matches.unwrap())?;
                Ok(Mode::Compile(compile))
            }
            Mode::COMPLETIONS => {
                let completions = Completions::parse_args(sub_matches.unwrap())?;
                Ok(Mode::Completions(completions))
            }
            Mode::MANIFEST => {
                let manifest = Manifest::parse_args(sub_matches.unwrap())?;
                Ok(Mode::Manifest(manifest))
            }
            Mode::SERVE => {
                let serve = ServeOpts::parse_args(sub_matches.unwrap())?;
                Ok(Mode::Serve(serve))
            }
            _ => {
                anyhow::bail!("unknown subcommand {:?}", sub);
            }
        }
    }
}

#[derive(Debug)]
struct Compile {
    format: fmt::Format,
    io: io::IOOpts,
}

impl Compile {
    fn run(self) -> Result<()> {
        log::debug!("reading source");
        let source = self.io.input()?.contents()?;
        log::debug!("parsing source into Document");
        let doc = prosidy::parse::parse_document(&source)?;
        log::debug!("opening output");
        let output = self.io.output()?;
        log::debug!("rendering document to output");
        self.format.write(output, &doc)?;
        Ok(())
    }
}

impl FromArgs for Compile {
    fn register_args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
        app.register::<fmt::Format>().register::<io::IOOpts>()
    }

    fn parse_args(matches: &ArgMatches) -> Result<Self> {
        let format = fmt::Format::parse_args(matches)?;
        let io = io::IOOpts::parse_args(matches)?;
        Ok(Compile { format, io })
    }
}

#[derive(Debug)]
struct Completions {
    shell: clap::Shell,
}

impl Completions {
    const SHELL: &'static str = "shell";

    fn run(self, mut app: App) -> Result<()> {
        app.gen_completions_to("prosidy", self.shell, &mut std::io::stdout());
        Ok(())
    }
}

impl FromArgs for Completions {
    fn register_args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
        let shell = Arg::with_name(Completions::SHELL)
            .help("Sets the completion format")
            .possible_values(&["bash", "fish", "zsh", "powershell"])
            .default_value("bash");
        app.arg(shell)
    }

    fn parse_args(matches: &ArgMatches) -> Result<Self> {
        let shell = value_t!(matches, Completions::SHELL, clap::Shell)?;
        Ok(Completions { shell })
    }
}

#[derive(Debug)]
struct Manifest {
    path: PathBuf,
    format: fmt::Format,
}

impl Manifest {
    const PATH: &'static str = "manifest-path";

    pub fn run(self) -> Result<()> {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async move {
            let manifest = manifest::Manifest::read(&self.path, true).await?;
            let stdout = std::io::stdout();
            let lock = stdout.lock();
            self.format.write(lock, &manifest)
        })
    }
}

impl FromArgs for Manifest {
    fn register_args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
        let path = Arg::with_name(Manifest::PATH)
            .help("The directory to parse metadata from.")
            .value_name("DIR")
            .required(true);
        app.arg(path).register::<fmt::Format>()
    }

    fn parse_args(matches: &ArgMatches) -> Result<Self> {
        let format = fmt::Format::parse_args(matches)?;
        let path = value_t!(matches, Manifest::PATH, PathBuf)?;
        Ok(Manifest { format, path })
    }
}


mod args;
mod fmt;
mod io;
mod manifest;
mod mediatype;
mod serve;
