# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

include .make/*.makefile

PROSIDY_FLAGS ?= --xmlns https://prosidy.org/schema/prosidy-manual.xsd --xslt style/manual.xsl
SERVE_FLAGS ?= --cache --validate --log-level info --xmlns https://prosidy.org/schema/prosidy-manual.xsd --xslt /style/manual.xsl
PLATFORM ?= linux-x64

manual_prosidy_srcs := $(wildcard manual/*.pro)
manual_xmls         := $(manual_prosidy_srcs:%.pro=target/%.xml)
manual_misc         := $(addprefix target/,$(wildcard manual/style/*) $(wildcard manual/schema/*))
rust_srcs           := $(shell find . -path '*/src/*.rs' -or -name 'Cargo.toml')

release_bins        := target/bin/prosidy-$(PLATFORM)

.PHONY: all clean license manual check check-xmls

all: $(manual_xmls) $(manual_misc) $(release_bins)

clean:
	cargo clean
	rm -rf target/manual target/bin

license:
	.mpl/headers add

manual: $(manual_xmls) $(manual_misc)

serve: target/bin/prosidy-$(PLATFORM)
	$< serve $(SERVE_FLAGS) ./manual

check: check-xmls
	.mpl/headers check
	cargo test

check-xmls: $(manual_xmls) $(manual_misc)
	xmllint --noout --schema target/manual/schema/prosidy-manual.xsd $<

#
# Rust targets
#

target/bin/prosidy-$(PLATFORM): $(rust_srcs)
	@mkdir -p target/bin
	cargo build --release -p prosidy-cli
	mv target/release/prosidy $@

#
# Building the Prosidy manual
#

$(manual_misc): target/manual/%: manual/%
	@mkdir -p $(dir $@)
	cp $< $@

# Compile all Prosidy documents
$(call template,xml,$(manual_prosidy_srcs))
