/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::io::Write;
use std::str::FromStr;

use anyhow::{bail, Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug)]
pub enum ASTFormat {
    CBOR,
    JSON,
}

impl ASTFormat {
    pub fn deserialize<'b, 'de, T>(self, source: &'de [u8]) -> Result<T>
    where
        T: Deserialize<'de>,
    {
        let value: T = match self {
            ASTFormat::JSON => {
                let mut de = serde_json::Deserializer::from_slice(source);
                T::deserialize(&mut de)?
            }
            ASTFormat::CBOR => {
                let mut de = serde_cbor::Deserializer::from_slice(source);
                T::deserialize(&mut de)?
            }
        };
        Ok(value)
    }

    pub fn serialize<W: Write, S: Serialize>(self, mut writer: W, value: &S) -> Result<()> {
        match self {
            ASTFormat::JSON => {
                serde_json::to_writer_pretty(&mut writer, value)?;
                writer.write_all(b"\n")?;
            }
            ASTFormat::CBOR => {
                serde_cbor::to_writer(&mut writer, value)?;
            }
        }
        Ok(())
    }
}

impl FromStr for ASTFormat {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "json" => Ok(ASTFormat::JSON),
            "cbor" => Ok(ASTFormat::CBOR),
            _ => bail!("unknown or unsupported AST format {}", s),
        }
    }
}
