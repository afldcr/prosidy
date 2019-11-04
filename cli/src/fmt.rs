/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::io::Write;

use anyhow::Result;
use clap::{App, Arg, ArgMatches};
use prosidy::xml::{self, XML};
use prosidy::xml::quick_xml::{events::Event, events::BytesDecl, events::BytesText};
use serde::Serialize;

use crate::args::{AppExt, FromArgs};

#[derive(Clone, Debug)]
pub struct Format {
    kind: FormatKind,
    opts: FormatOpts,
}

impl Format {
    pub fn write<S: Serialize + XML, W: Write>(&self, writer: W, value: &S) -> Result<()> {
        self.kind.write(&self.opts, writer, value)
    }
}

impl FromArgs for Format {
    fn register_args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
        app.register::<FormatKind>().register::<FormatOpts>()
    }

    fn parse_args(matches: &ArgMatches) -> Result<Self> {
        let kind = FormatKind::parse_args(matches)?;
        let opts = FormatOpts::parse_args(matches)?;
        Ok(Format { kind, opts })
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FormatKind {
    CBOR,
    JSON,
    XML,
}

impl FormatKind {
    pub fn write<S: Serialize + XML, W: Write>(self, opts: &FormatOpts, writer: W, value: &S) -> Result<()> {
        match self {
            FormatKind::CBOR => opts.write_cbor(writer, value),
            FormatKind::JSON => opts.write_json(writer, value),
            FormatKind::XML  => opts.write_xml(writer, value),
        }
    }


    pub fn media_type(self) -> &'static mime::Mime {
        match self {
            FormatKind::CBOR => &crate::mediatype::APPLICATION_CBOR,
            FormatKind::JSON => &mime::APPLICATION_JSON,
            FormatKind::XML  => &mime::TEXT_XML,
        }
    }
}

impl FromArgs for FormatKind {
    fn register_args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
        let arg = Arg::with_name(ARG_FORMAT)
            .help("Selects the output format of the parsed AST")
            .long("format")
            .short("f")
            .default_value_if(ARG_JSON_PRETTY, None, ARG_FORMAT_JSON)
            .default_value_if(ARG_XSLT, None, ARG_FORMAT_XML)
            .default_value_if(ARG_XMLNS, None, ARG_FORMAT_XML)
            .default_value(ARG_FORMAT_JSON)
            .takes_value(true)
            .possible_values(&[ARG_FORMAT_CBOR, ARG_FORMAT_JSON, ARG_FORMAT_XML]);
        app.arg(arg)
    }

    fn parse_args(matches: &ArgMatches) -> Result<Self> {
        let fmt = match matches.value_of(ARG_FORMAT) {
            Some(ARG_FORMAT_CBOR) => FormatKind::CBOR,
            Some(ARG_FORMAT_JSON) => FormatKind::JSON,
            Some(ARG_FORMAT_XML) => FormatKind::XML,
            Some(format) => anyhow::bail!("Unknown format name {:?}", format),
            None => anyhow::bail!("No format name provided"),
        };
        Ok(fmt)
    }
}

#[derive(Clone, Debug)]
pub struct FormatOpts {
    json_pretty: bool,
    xml_namespace: Option<String>,
    xml_stylesheets: Vec<String>,
}

impl FormatOpts {
    pub fn write_cbor<S: Serialize, W: Write>(&self, writer: W, value: &S) -> Result<()> {
        serde_cbor::to_writer(writer, value)?;
        Ok(())
    }

    pub fn write_json<S: Serialize, W: Write>(&self, mut writer: W, value: &S) -> Result<()> {
        if self.json_pretty {
            serde_json::to_writer_pretty(&mut writer, value)?;
        } else {
            serde_json::to_writer(&mut writer, value)?;
        }
        writer.write_all(b"\n")?;
        Ok(())
    }

    pub fn write_xml<S: XML, W: Write>(&self, writer: W, value: &S) -> Result<()> {
        let mut writer = xml::quick_xml::Writer::new(writer);
        // first, write the XML declaration
        let decl = BytesDecl::new(b"1.0", Some(b"UTF-8"), None);
        writer.write_event(Event::Decl(decl))?;
        // next, write all of the stylesheet instructions as pre-processor events
        for stylesheet in self.xml_stylesheets.iter() {
            let contents = format!(
                r#"xml-stylesheet type="text/xsl" href="{}""#,
                stylesheet,
            );
            let event = BytesText::from_escaped_str(&contents);
            writer.write_event(Event::PI(event))?;
        }
        // now, create a callback hook for writing events into the writer.
        let mut first = true;
        let mut handle = |mut event: Event| {
            if first {
                first = false;
                let start = match event {
                    Event::Start(ref mut start) => start,
                    Event::Empty(ref mut empty) => empty,
                    _ => panic!("The first emitted XML event was not a tag"),
                };
                if let Some(ref ns) = self.xml_namespace {
                    start.push_attribute(("xmlns", ns.as_str()));
                }
                start.push_attribute(("xmlns:prosidy", PROSIDY_URI));
            }
            writer.write_event(event).map(|_| ())
        };
        value.to_events(&mut handle)?;
        writer.write_event(Event::Eof)?;
        writer.into_inner().write_all(b"\n")?;
        Ok(())
    }
}

impl FromArgs for FormatOpts {
    fn register_args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
        let json_pretty = Arg::with_name(ARG_JSON_PRETTY)
            .help("Pretty prints JSON output")
            .long("pretty")
            .short("p");
        let xmlns = Arg::with_name(ARG_XMLNS)
            .help("Assign a namespace to non-Prosidy tags in the document")
            .long("xmlns")
            .short("N")
            .value_name("NAMESPACE URI");
        let xslt = Arg::with_name(ARG_XSLT)
            .help("Attach one or more XSLT stylesheets to the XML output")
            .long("xslt")
            .short("s")
            .value_name("STYLESHEET")
            .number_of_values(1)
            .multiple(true);
        app.arg(json_pretty).arg(xslt).arg(xmlns)
    }

    fn parse_args(matches: &ArgMatches) -> Result<Self> {
        let json_pretty = matches.is_present(ARG_JSON_PRETTY);
        let xml_stylesheets = matches
            .values_of(ARG_XSLT)
            .into_iter()
            .flatten()
            .map(String::from)
            .collect();
        let xml_namespace = matches.value_of(ARG_XMLNS).map(String::from);
        Ok(FormatOpts {
            json_pretty,
            xml_stylesheets,
            xml_namespace,
        })
    }
}

const ARG_FORMAT: &str = "format";
const ARG_FORMAT_CBOR: &str = "cbor";
const ARG_FORMAT_JSON: &str = "json";
const ARG_FORMAT_XML: &str = "xml";

const ARG_JSON_PRETTY: &str = "json-pretty-print";
const ARG_XMLNS: &str = "xmlns";
const ARG_XSLT: &str = "xslt";

const PROSIDY_URI: &str = "https://prosidy.org/schema/prosidy.xsd";
