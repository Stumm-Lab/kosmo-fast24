/*
 * Copyright (c) Kia Shakiba
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::io::{Cursor, Error, ErrorKind};
use byteorder::{LittleEndian, ReadBytesExt};

use kwik::{
	binary_reader::{SizedChunk, Chunk as ReadChunk},
	binary_writer::Chunk as WriteChunk,
};

pub type Timestamp = u64;
pub type Key = u64;
pub type Size = u32;
pub type Ttl = u32;

#[derive(Debug, PartialEq, Clone)]
pub enum Command {
	Get,
	Set,
}

#[derive(Debug, Clone)]
pub struct Access {
	pub timestamp: Timestamp,
	pub command: Command,
	pub key: Key,
	pub size: Size,
	pub ttl: Ttl,
}

impl Access {
	pub fn is_valid_self_populating(&self) -> bool {
		self.command == Command::Get && self.size > 0
	}
}

impl SizedChunk for Access {
	const SIZE: usize = 25;
}

impl ReadChunk for Access {
	fn new(buf: &[u8; Self::SIZE]) -> Result<Self, Error> where Self: Sized {
		let mut rdr = Cursor::new(buf);

		let Ok(timestamp) = rdr.read_u64::<LittleEndian>() else {
			return Err(Error::new(ErrorKind::InvalidData, "Invalid access timestamp."));
		};

		let command = match rdr.read_u8() {
			Ok(byte) => Command::from_byte(byte)?,

			Err(_) => return Err(Error::new(
				ErrorKind::InvalidData,
				"Invalid access command."
			)),
		};

		let Ok(key) = rdr.read_u64::<LittleEndian>() else {
			return Err(Error::new(
				ErrorKind::InvalidData,
				"Invalid access key."
			));
		};

		let Ok(size) = rdr.read_u32::<LittleEndian>() else {
			return Err(Error::new(
				ErrorKind::InvalidData,
				"Invalid access size."
			));
		};

		let Ok(ttl) = rdr.read_u32::<LittleEndian>() else {
			return Err(Error::new(
				ErrorKind::InvalidData,
				"Invalid access ttl."
			));
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
	fn as_chunk(&self, buf: &mut Vec<u8>) -> Result<(), Error> {
		buf.extend_from_slice(&self.timestamp.to_le_bytes());
		buf.extend_from_slice(&self.command.as_byte().to_le_bytes());
		buf.extend_from_slice(&self.key.to_le_bytes());
		buf.extend_from_slice(&self.size.to_le_bytes());
		buf.extend_from_slice(&self.ttl.to_le_bytes());

		Ok(())
	}
}

impl Command {
	fn from_byte(byte: u8) -> Result<Self, Error> {
		match byte {
			0 => Ok(Command::Get),
			1 => Ok(Command::Set),

			_ => Err(Error::new(ErrorKind::InvalidData, "Invalid command byte.")),
		}
	}

	fn as_byte(&self) -> u8 {
		match self {
			Command::Get => 0,
			Command::Set => 1,
		}
	}
}
