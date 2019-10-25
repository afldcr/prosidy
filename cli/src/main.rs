/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::path::PathBuf;

use ::xml::namespace::Namespace;
use ::xml::EmitterConfig;
use anyhow::Result;
use prosidy::Document;
use structopt::StructOpt;

use format::ASTFormat;

fn main() {
    let opts = Opts::from_args();
    env_logger::init();
    if let Err(e) = opts.run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

#[derive(Debug, StructOpt)]
enum Opts {
    AST(AST),
    Compile(Compile),
    Render(Render),
}

impl Opts {
    fn run(self) -> Result<()> {
        match self {
            Opts::AST(ast) => ast.run(),
            Opts::Compile(compile) => compile.run(),
            Opts::Render(render) => render.run(),
        }
    }
}

#[derive(Debug, StructOpt)]
/// Read a Prosidy file and serialize its AST
struct AST {
    #[structopt(flatten)]
    io: IOOpts,
    #[structopt(short = "f", long = "format", default_value = "json")]
    ast_format: ASTFormat,
}

impl AST {
    fn run(self) -> Result<()> {
        log::debug!("opening prosidy input");
        let mut input = io::Input::new(self.io.input.as_ref())?;
        log::debug!("reading input to string");
        let source = input.contents_string()?;
        log::debug!("parsing document from input");
        let doc = prosidy::parse::parse_document(source.as_str())?;
        log::debug!("opening output to write serialized AST into");
        let output = io::Output::new(self.io.output.as_ref())?;
        self.ast_format.serialize(output, &doc)?;
        Ok(())
    }
}

#[derive(Debug, StructOpt)]
struct Compile {
    #[structopt(flatten)]
    io: IOOpts,
    #[structopt(flatten)]
    xml: XMLOpts,
}

impl Compile {
    fn run(self) -> Result<()> {
        log::debug!("opening prosidy input");
        let mut input = io::Input::new(self.io.input.as_ref())?;
        log::debug!("reading input to string");
        let source = input.contents_string()?;
        log::debug!("parsing document from input");
        let doc = prosidy::parse::parse_document(source.as_str())?;
        log::debug!("opening output to render document into");
        let output = io::Output::new(self.io.output.as_ref())?;
        self.xml.render(output, doc)
    }
}

#[derive(Debug, StructOpt)]
struct Render {
    #[structopt(flatten)]
    io: IOOpts,
    #[structopt(flatten)]
    xml: XMLOpts,
    #[structopt(short = "f", long = "format", default_value = "json")]
    ast_format: ASTFormat,
}

impl Render {
    fn run(self) -> Result<()> {
        log::debug!("opening prosidy input");
        let mut input = io::Input::new(self.io.input.as_ref())?;
        log::debug!("reading input to string");
        let source = input.contents_bytes()?;
        log::debug!("parsing AST from input");
        let doc = self.ast_format.deserialize::<Document>(&source)?;
        log::debug!("opening output to write converted document into");
        let output = io::Output::new(self.io.output.as_ref())?;
        log::debug!("writing document");
        self.xml.render(output, doc)
    }
}

#[derive(Debug, StructOpt)]
struct IOOpts {
    #[structopt(short = "i", long = "in")]
    /// The file path to the Prosidy source to parse. Defaults to stdin.
    input: Option<PathBuf>,
    #[structopt(short = "o", long = "out")]
    /// The destination to serialize the AST to. Defaults to stdout.
    output: Option<PathBuf>,
}

#[derive(Debug, StructOpt)]
struct XMLOpts {
    #[structopt(short = "p", long = "prefix", requires = "namespace")]
    prefix: Option<String>,
    #[structopt(short = "n", long = "namespace", requires = "prefix")]
    namespace: Option<String>,
}

impl XMLOpts {
    fn namespace(&self) -> Option<(&str, &str)> {
        let XMLOpts { prefix, namespace } = self;
        match (prefix, namespace) {
            (Some(ref prefix), Some(ref namespace)) => Some((prefix, namespace)),
            (None, None) => None,
            _ => unreachable!("--prefix and --namespace must be provided together"),
        }
    }

    fn render(self, output: io::Output, doc: Document) -> Result<()> {
        let mut namespace = Namespace::empty();
        namespace.put("prosidy", "https://prosidy.org/schema");
        let prefix: Option<&str> = self.namespace().map(|(pfx, uri)| {
            namespace.put(pfx, uri);
            pfx
        });
        let mut writer = EmitterConfig::new()
            .perform_indent(false)
            .write_document_declaration(true)
            .normalize_empty_elements(true)
            .keep_element_names_stack(true)
            .create_writer(output);
        for event in xml::XML::new(&doc, &namespace, prefix) {
            writer.write(event)?;
        }
        Ok(())
    }
}

mod format;
mod io;
mod xml;
