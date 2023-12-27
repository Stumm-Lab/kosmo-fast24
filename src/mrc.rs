/*
 * Copyright (c) Kia Shakiba
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

#![feature(btree_cursors)]

mod access;
mod algorithm;
mod histogram;
mod figure;
mod curve;
mod curve_plot;
mod shards;
mod cache;
mod kosmo;
mod minisimulations;

use std::time::Instant;
use clap::{Parser, ValueEnum};

use kwik::{
	mem,
	fmt,
	FileReader,
	binary_reader::{BinaryReader, SizedChunk},
	progress::{Progress, Tag},
};

use crate::{
	access::Access,
	shards::{Shards, ShardsFixedRate, ShardsFixedSize},
	algorithm::Algorithm,
	cache::CachePolicy,
	kosmo::{Kosmo, KosmoPolicy},
	minisimulations::Minisimulations,
	figure::Figure,
	curve::Curve,
	curve_plot::CurvePlot,
};

const BATCH_SIZE: usize = 10_000_000;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
	#[arg(short, long)]
	path: String,

	#[arg(short, long)]
	wss: u64,

	#[arg(short = 't', long)]
	shards_t: Option<u64>,

	#[arg(short, long)]
	shards_s: Option<u32>,

	#[arg(short, long)]
	kosmo_policy: Option<KosmoPolicy>,

	#[arg(short, long)]
	minisim_policy: Option<CachePolicy>,

	#[arg(short, long)]
	output: String,

	#[arg(short, long)]
	accurate_path: Option<String>,

	#[arg(short, long)]
	run_type: RunType,
}

#[derive(Clone, PartialEq, ValueEnum)]
enum RunType {
	Memory,
	Throughput,
}

fn main() {
	let args = Args::parse();

	let mut algorithm = match (&args.kosmo_policy, &args.minisim_policy) {
		(Some(_), None) => init_kosmo(&args),
		(None, Some(_)) => init_minisimulations(&args),
		(Some(_), Some(_)) => panic!("You may not configure both Kosmo and MiniSim simultaneously."),
		(None, None) => panic!("You must configure at one of Kosmo or MiniSim."),
	};

	let mut reader = BinaryReader::<Access>::new(&args.path)
		.expect("Invalid trace path.");

	println!("{}", args.path);

	let mut progress = Progress::new(reader.size(), &[
		Tag::Tps,
		Tag::Eta,
		Tag::Time,
	]);

	if args.run_type == RunType::Memory {
		mem::clear(None).expect("Could not clear memory refs.");
	}

	let mut accesses: Option<Vec<Access>> = match args.run_type {
		RunType::Throughput => Some(Vec::<Access>::new()),
		_ => None,
	};

	let mut total_time: u64 = 0;
	let mut total_accesses: u64 = 0;

	while let Some(access) = reader.read_chunk() {
		match accesses.as_mut() {
			Some(accesses) if accesses.len() == BATCH_SIZE => {
				total_time += run_batch(&mut algorithm, accesses);
				accesses.clear();
			},

			Some(accesses) => accesses.push(access),
			None => algorithm.handle(&access),
		}

		progress.tick(Access::SIZE);
		total_accesses += 1;
	}

	if let Some(accesses) = accesses.as_mut() {
		if !accesses.is_empty() {
			total_time += run_batch(&mut algorithm, accesses);
		}
	}

	let accurate_curve = args.accurate_path.map(|path| {
		Curve::from_file(&path)
			.expect("Could not find accurate curve.")
	});

	let mut figure = Figure::new(1);
	let mut plot = CurvePlot::default();

	let curve = algorithm.curve();

	if let Some(accurate_curve) = &accurate_curve {
		println!("MAE: {}", accurate_curve.mae(&curve));
	}

	let algorithm_id = match args.kosmo_policy.is_some() {
		true => "Kosmo",
		false => "MiniSim",
	};

	plot.add(algorithm_id, &curve);

	match args.run_type {
		RunType::Memory => {
			let hwm = mem::hwm(None).expect("Could not get memory HWM.");

			println!(
				"Memory usage: {} ({} B)",
				fmt::memory(hwm, Some(2)),
				hwm,
			);
		},

		RunType::Throughput => {
			let throughput = total_accesses / total_time;

			println!(
				"Throughput: {} accesses/ms",
				fmt::number(throughput),
			);
		},
	}

	figure.add(&mut plot);

	figure
		.save(&args.output)
		.expect("Could not save figure.");
}

fn run_batch(algorithm: &mut Box<dyn Algorithm>, accesses: &[Access]) -> u64 {
	let start_time = Instant::now();

	for access in accesses {
		algorithm.handle(access);
	}

	start_time.elapsed().as_millis() as u64
}

fn init_kosmo(args: &Args) -> Box<dyn Algorithm> {
	let policy = args.kosmo_policy.as_ref().unwrap().clone();
	let shards = init_shards(args);

	Box::new(Kosmo::new(&[policy], shards))
}

fn init_minisimulations(args: &Args) -> Box<dyn Algorithm> {
	let policy = args.minisim_policy.as_ref().unwrap();
	let shards = init_shards(args);

	Box::new(Minisimulations::new(policy, args.wss, shards))
}

fn init_shards(args: &Args) -> Option<Box<dyn Shards>> {
	match (args.shards_t, args.shards_s) {
		(Some(t), Some(s_max)) => Some(Box::new(ShardsFixedSize::new(t, s_max))),
		(Some(t), None) => Some(Box::new(ShardsFixedRate::new(t))),
		(None, Some(_)) => panic!("You must specify an initial sampling threshold when using SHARDS fixed-size."),
		(None, None) => None,
	}
}
