# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

include .make/*.makefile

PROSIDY_FLAGS ?= --namespace https://prosidy.org/manual/schema --prefix pm

manual_prosidy_srcs := $(wildcard manual/*.pro)
manual_xmls         := $(manual_prosidy_srcs:%.pro=target/%.xml)
manual_htmls        := $(manual_xmls:%.xml=%.html)
manual_styles       := $(foreach style,$(wildcard manual/style/*),$(style:manual/style/%=target/manual/%))
rust_srcs           := $(shell find . -path '*/src/*.rs' -or -name 'Cargo.toml')

.PHONY: all clean license manual check check-xmls

all: $(manual_htmls) target/release/prosidy

clean:
	cargo clean
	rm -rf target/manual

license:
	.mpl/headers add

manual: $(manual_htmls)

check: check-xmls
	.mpl/headers check
	cargo test

check-xmls: $(manual_xmls) $(manual_styles) target/manual/schema.xsd target/manual/prosidy.xsd
	xmllint --noout --schema target/manual/schema.xsd $<

#
# Rust targets
#

target/release/prosidy: $(rust_srcs)
	cargo build --release -p prosidy-cli

#
# Building the Prosidy manual
#

$(manual_htmls): %.html: %.xml $(manual_styles)
	xsltproc --output $@ manual/style/manual.xsl $<

target/manual/prosidy.xsd:
	@mkdir -p target/manual
	curl -L -o $@ https://prosidy.org/schema/prosidy.xsd

target/manual/schema.xsd: manual/style/schema.xsd
	@mkdir -p target/manual
	cp $< $@

target/manual/hyperlinks.xml: manual/style/hyperlinks.xml
	@mkdir -p target/manual
	cp $< $@

target/manual/manual.xsl: manual/style/manual.xsl
	@mkdir -p target/manual
	cp $< $@

# Compile all Prosidy documents
$(call template,xml,$(manual_prosidy_srcs))
