/*
 * Copyright (c) Kia Shakiba
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

#![feature(btree_cursors)]

mod access;

use rustc_hash::FxHashSet;
use clap::Parser;
use crate::access::{Access, Key};

use kwik::{
	FileReader,
	binary_reader::{BinaryReader, SizedChunk},
	progress::{Progress, Tag},
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
	#[arg(short, long)]
	path: String,
}

fn main() {
	let args = Args::parse();

	let mut reader = BinaryReader::<Access>::new(&args.path)
		.expect("Invalid trace path.");

	println!("{}", args.path);

	let mut progress = Progress::new(reader.size(), &[
		Tag::Tps,
		Tag::Eta,
		Tag::Time,
	]);

	let mut set = FxHashSet::<Key>::default();
	let mut wss: u64 = 0;

	while let Some(access) = reader.read_chunk() {
		if access.is_valid_self_populating() && set.insert(access.key) {
			wss += access.size as u64;
		}

		progress.tick(Access::SIZE);
	}

	println!("WSS: {wss}");
}
