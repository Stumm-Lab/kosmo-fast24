/*
 * Copyright (c) Kia Shakiba
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

#![feature(btree_cursors)]

mod access;

use std::cmp;
use rustc_hash::FxHashMap;
use clap::Parser;
use crate::access::{Access, Key};

use kwik::{
	FileReader,
	binary_reader::{BinaryReader, SizedChunk},
	progress::{Progress, Tag},
  fmt,
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

	let mut map = FxHashMap::<Key, u64>::default();
	let mut wss: u64 = 0;
	// This is the WSS based on the first seen size of an object.
	let mut naive_wss: u64 = 0;

	while let Some(access) = reader.read_chunk() {
		if !access.is_valid_self_populating() {
			progress.tick(Access::SIZE);
			continue;
		}

		if let Some(value) = map.get_mut(&access.key) {
			let max = cmp::max(*value, access.size as u64);
			wss = wss - *value + max;
			*value = max;
		} else {
			map.insert(access.key, access.size as u64);
			wss += access.size as u64;
			naive_wss += access.size as u64;
		}
		progress.tick(Access::SIZE);
	}

	println!("WSS: {wss} [bytes] ({})", fmt::memory(wss, Some(2)));
	println!("Naive WSS: {naive_wss} [bytes] ({})", fmt::memory(naive_wss, Some(2)));
}
