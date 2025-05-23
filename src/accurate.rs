/*
 * Copyright (c) Kia Shakiba
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

#![feature(btree_cursors)]

mod access;
mod histogram;
mod shards;
mod curve;
mod cache;

use clap::Parser;

use kwik::{
	file::{
		FileReader,
		binary::{BinaryReader, SizedChunk},
	},
	progress::{Progress, Tag},
};

use crate::{
	access::Access,
	curve::Curve,
	cache::CachePolicy,
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
	#[arg(short, long)]
	path: String,

	#[arg(short, long)]
	wss: u64,

	#[arg(short = 'e', long)]
	policy: CachePolicy,

	#[arg(short, long)]
	output: String,
}

fn main() {
	let args = Args::parse();

	let mut curve = Curve::default();
	let step_size = if args.wss > 100 { args.wss / 100 } else { 1 };

	let cache_sizes = (step_size..=args.wss)
		.step_by(step_size as usize)
		.collect::<Vec<u64>>();

	let Ok(reader) = BinaryReader::<Access>::from_path(&args.path) else {
		panic!("Invalid path.");
	};

	println!("{}", args.path);

	let mut progress = Progress::new(reader.size() * cache_sizes.len() as u64)
		.with_tag(Tag::Tps)
		.with_tag(Tag::Eta)
		.with_tag(Tag::Time);

	// Loop through all cache sizes individually and simulate them one-by-one.
	// We could do this in parallel, but the memory overhead is too large.
	for cache_size in &cache_sizes {
		let mut cache = args.policy.new_cache(*cache_size);

		let Ok(reader) = BinaryReader::<Access>::from_path(&args.path) else {
			panic!("Invalid path.");
		};

		let mut count: u64 = 0;

		for mut access in reader {
			if access.is_valid_self_populating() {
				access.timestamp = count + 1;
				count += 1;

				cache.handle_self_populating(&access);
			}

			progress.tick(Access::chunk_size());
		}

		curve.add(cache.size(), cache.miss_ratio());

		if curve.to_file(&args.output).is_err() {
			println!("Could not save curve to storage.");
		}
	}
}
