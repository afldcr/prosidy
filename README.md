<!-- This Source Code Form is subject to the terms of the Mozilla Public
     License, v. 2.0. If a copy of the MPL was not distributed with this
     file, You can obtain one at https://mozilla.org/MPL/2.0/. -->

# Prosidy

An extensible language for writing.

## Project structure

- `prosidy-ast` (`./ast`): Contains core document types.
- `prosidy-parse` (`./parse`): A library for parsing into an AST.
- `prosidy` (`./prosidy`): Just re-exports the other libraries.
- `prosidy-cli` (`./cli`): The command line interface for Prosidy.
