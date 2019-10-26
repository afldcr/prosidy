/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::io::Write;

use anyhow::Result;
use clap::{App, Arg, ArgMatches};
use prosidy::Document;
use xml::namespace::Namespace;
use xml::EmitterConfig;

use crate::args::{AppExt, FromArgs};
use crate::xmlgen::XMLGen;

#[derive(Clone, Debug)]
pub struct Format {
    kind: FormatKind,
    opts: FormatOpts,
}

impl Format {
    pub fn write<W: Write>(&self, writer: W, document: &Document) -> Result<()> {
        self.kind.write(&self.opts, writer, document)
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

#[derive(Copy, Clone, Debug)]
pub enum FormatKind {
    CBOR,
    JSON,
    XML,
}

impl FormatKind {
    pub fn write<W: Write>(self, opts: &FormatOpts, writer: W, document: &Document) -> Result<()> {
        match self {
            FormatKind::CBOR    => opts.write_cbor(writer, document),
            FormatKind::JSON    => opts.write_json(writer, document),
            FormatKind::XML     => opts.write_xml(writer, document),
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
    xml_namespace: Option<XmlNS>,
    xml_stylesheets: Vec<String>,
}

impl FormatOpts {
    pub fn write_cbor<W: Write>(&self, writer: W, document: &Document) -> Result<()> {
        serde_cbor::to_writer(writer, document)?;
        Ok(())
    }


    pub fn write_json<W: Write>(&self, mut writer: W, document: &Document) -> Result<()> {
        if self.json_pretty {
            serde_json::to_writer_pretty(&mut writer, document)?;
        } else {
            serde_json::to_writer(&mut writer, document)?;
        }
        writer.write_all(b"\n")?;
        Ok(())
    }

    pub fn write_xml<W: Write>(&self, mut writer: W, document: &Document) -> Result<()> {
        let mut namespace = Namespace::empty();
        namespace.put(PROSIDY_PREFIX, PROSIDY_URI);
        let prefix = self.xml_namespace.as_ref().map(|xmlns| {
            namespace.put(&xmlns.prefix, &xmlns.uri);
            xmlns.prefix.as_str()
        });
        let mut event_writer = EmitterConfig {
            autopad_comments: true,
            cdata_to_characters: false,
            indent_string: Default::default(),
            keep_element_names_stack: true,
            line_separator: Default::default(),
            normalize_empty_elements: true,
            perform_escaping: true,
            perform_indent: false,
            write_document_declaration: true,
        }.create_writer(&mut writer);
        let stylesheets = self.xml_stylesheets.iter().map(String::as_str);
        for event in XMLGen::new(document, &namespace, stylesheets, prefix) {
            event_writer.write(event)?;
        }
        writer.write_all(b"\n")?;
        Ok(())
    }
}

impl FromArgs for FormatOpts {
    fn register_args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
        let json_pretty = Arg::with_name(ARG_JSON_PRETTY)
            .help("Pretty prints JSON output")
            .long("pretty")
            .short("p");
        let xslt = Arg::with_name(ARG_XSLT)
            .help("Attach one or more XSLT stylesheets to the XML output")
            .long("xslt")
            .short("s")
            .value_name("STYLESHEET")
            .number_of_values(1)
            .multiple(true);
        app.arg(json_pretty).arg(xslt).register::<Option<XmlNS>>()
    }

    fn parse_args(matches: &ArgMatches) -> Result<Self> {
        let json_pretty = matches.is_present(ARG_JSON_PRETTY);
        let xml_stylesheets = matches
            .values_of(ARG_XSLT)
            .into_iter()
            .flatten()
            .map(|s| {
                format! {
                    r#"type="text/xsl" href="{}""#,
                    xml::escape::escape_str_attribute(s),
                }
            })
            .collect();
        let xml_namespace = Option::parse_args(matches)?;
        Ok(FormatOpts {
            json_pretty,
            xml_stylesheets,
            xml_namespace,
        })
    }
}

#[derive(Clone, Debug)]
pub struct XmlNS {
    prefix: String,
    uri: String,
}

impl FromArgs for Option<XmlNS> {
    fn register_args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
        let arg = Arg::with_name(ARG_XMLNS)
            .help("Set the XML namespace prefix and URI to apply to all nodes")
            .long("xmlns")
            .short("N")
            .value_names(&["prefix", "uri"]);
        app.arg(arg)
    }

    fn parse_args(matches: &ArgMatches) -> Result<Self> {
        if let Some(mut values) = matches.values_of(ARG_XMLNS) {
            let prefix = values.next().unwrap();
            if prefix == PROSIDY_PREFIX {
                anyhow::bail!(
                    "The namespace prefix 'prosidy' is reserved; \
                     please choose a different namespace."
                );
            }
            let uri = values.next().unwrap();
            debug_assert!(values.next().is_none());
            Ok(Some(XmlNS {
                prefix: prefix.into(),
                uri: uri.into(),
            }))
        } else {
            Ok(None)
        }
    }
}

const ARG_FORMAT: &str = "format";
const ARG_FORMAT_CBOR: &str = "cbor";
const ARG_FORMAT_JSON: &str = "json";
const ARG_FORMAT_XML: &str = "xml";

const ARG_JSON_PRETTY: &str = "json-pretty-print";
const ARG_XMLNS: &str = "xmlns";
const ARG_XSLT: &str = "xslt";

const PROSIDY_PREFIX: &str = "prosidy";
const PROSIDY_URI: &str = "https://prosidy.org/schema/prosidy.xsd";
