/*
 * Copyright (c) Kia Shakiba
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::io::{self, Cursor};
use byteorder::{LittleEndian, ReadBytesExt};
use kwik::file::binary::{SizedChunk, ReadChunk, WriteChunk};

pub type Timestamp = u64;
pub type Key = u64;
pub type Size = u32;
pub type Ttl = u32;

#[derive(Debug, PartialEq, Clone)]
pub enum Command {
	Get,
	Set,
}

/// An individual access in a cache access trace.
#[derive(Debug, Clone)]
pub struct Access {
	pub timestamp: Timestamp,
	pub command: Command,
	pub key: Key,
	pub size: Size,
	pub ttl: Option<Ttl>,
}

impl Access {
	/// Returns `true` if the access is a GET and has a non-zero size.
	pub fn is_valid_self_populating(&self) -> bool {
		self.command == Command::Get && self.size > 0
	}
}

impl SizedChunk for Access {
	fn chunk_size() -> usize {
		25
	}
}

impl ReadChunk for Access {
	fn from_chunk(buf: &[u8]) -> io::Result<Self> {
		let mut rdr = Cursor::new(buf);

		let timestamp = rdr.read_u64::<LittleEndian>()?;

		let command_byte = rdr.read_u8()?;
		let command = Command::from_byte(command_byte)?;

		let key = rdr.read_u64::<LittleEndian>()?;
		let size = rdr.read_u32::<LittleEndian>()?;

		let ttl = match rdr.read_u32::<LittleEndian>()? {
			0 => None,
			value => Some(value),
		};

		let access = Access {
			timestamp,
			command,
			key,
			size,
			ttl,
		};

		Ok(access)
	}
}

impl WriteChunk for Access {
	fn as_chunk(&self, buf: &mut Vec<u8>) -> io::Result<()> {
		buf.extend_from_slice(&self.timestamp.to_le_bytes());
		buf.extend_from_slice(&self.command.as_byte().to_le_bytes());
		buf.extend_from_slice(&self.key.to_le_bytes());
		buf.extend_from_slice(&self.size.to_le_bytes());
		buf.extend_from_slice(&self.ttl.unwrap_or(0).to_le_bytes());

		Ok(())
	}
}

impl Command {
	fn from_byte(byte: u8) -> io::Result<Self> {
		match byte {
			0 => Ok(Command::Get),
			1 => Ok(Command::Set),

			_ => Err(io::Error::new(
				io::ErrorKind::InvalidData,
				"Invalid command byte.",
			)),
		}
	}

	fn as_byte(&self) -> u8 {
		match self {
			Command::Get => 0,
			Command::Set => 1,
		}
	}
}
