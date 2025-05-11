/*
 * Copyright (c) Kia Shakiba
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::{
	io,
	collections::BTreeMap,
	collections::btree_map::Values,
	ops::Bound,
	iter::IntoIterator,
};

use kwik::{
	math,
	file::{
		FileReader,
		FileWriter,
		csv::{CsvWriter, CsvReader, RowData, ReadRow, WriteRow},
	},
};

use crate::{
	histogram::Histogram,
	shards::Shards,
};

/// An MRC curve, storing the points of the MRC.
#[derive(Clone, Default)]
pub struct Curve {
	points: BTreeMap<u64, Point>,
}

/// An individual point on an MRC.
#[derive(Clone)]
pub struct Point {
	size: u64,
	miss_ratio: f64,
}

impl Curve {
	/// Returns an empty MRC.
	pub fn new() -> Self {
		Curve {
			points: BTreeMap::new(),
		}
	}

	/// Returns the number of points in the MRC.
	pub fn size(&self) -> usize {
		self.points.len()
	}

	/// Returns `true` if the MRC is empty.
	pub fn is_empty(&self) -> bool {
		self.points.is_empty()
	}

	/// Constructs an MRC from the supplied histogram.
	pub fn from_histogram(histogram: &Histogram) -> Self {
		let mut points = BTreeMap::<u64, Point>::new();

		let total = histogram.get_total();
		let mut current: f64 = 0.0;

		for (size, count) in histogram.into_iter() {
			current += count;

			points.insert(
				size,
				Point::new(size, 1.0 - current / total)
			);
		}

		Curve {
			points
		}
	}

	/// Constructs an MRC from the supplied histogram, applying
	/// the adjusted SHARDS correction if necessary.
	pub fn from_corrected_histogram(
		histogram: &Histogram,
		shards: &dyn Shards
	) -> Self {
		let mut points = BTreeMap::<u64, Point>::new();

		let mut correction = shards.get_correction() as f64;
		let total = histogram.get_corrected_total(shards);

		let mut current: f64 = 0.0;

		for (size, count) in histogram.into_iter() {
			current += count;

			if correction > 0.0 || correction.abs() < current {
				current += correction;
				correction = 0.0;
			} else if correction < 0.0 {
				correction += current;
				current = 0.0;
			}

			points.insert(
				size,
				Point::new(size, 1.0 - current / total)
			);
		}

		Curve {
			points
		}
	}

	/// Returns the maximum size of the MRC.
	pub fn get_max_size(&self) -> u64 {
		match self.points.last_key_value() {
			Some((size, _)) => *size,
			None => 0,
		}
	}

	/// Returns the miss ratio at the supplied size. If an exact point
	/// corresponding to the size does not exist, the miss ratio
	/// of the previous point is returned. If no such point exists,
	/// `1.0` is returned.
	pub fn get_miss_ratio(&self, size: u64) -> f64 {
		match (
			self.points.upper_bound(Bound::Included(&size)).prev(),
			self.points.lower_bound(Bound::Included(&size)).next(),
		) {
			(Some((_, prev_point)), Some((next_cache_size, next_point))) => {
				if size == *next_cache_size {
					return next_point.get_miss_ratio();
				}

				prev_point.get_miss_ratio()
			},

			(None, Some((next_cache_size, next_point))) => {
				if size == *next_cache_size {
					return next_point.get_miss_ratio();
				}

				1.0
			},

			(Some((_, prev_point)), None) => {
				prev_point.get_miss_ratio()
			},

			(None, None) => match self.points.last_key_value() {
				Some((_, point)) => point.get_miss_ratio(),
				None => 1.0,
			},
		}
	}

	/// Adds a point to the MRC.
	pub fn add(&mut self, size: u64, miss_ratio: f64) {
		self.points.insert(
			size,
			Point::new(size, miss_ratio)
		);
	}

	/// Returns the mean absolute error (MAE) of the current MRC
	/// compared to the supplied MRC.
	pub fn mae(&self, curve: &Curve) -> f64 {
		let max_size = *math::max(&[
			self.get_max_size(),
			curve.get_max_size(),
		]).unwrap();

		let num_points: u64 = 100;
		let step_size = max_size / num_points;

		let total = (step_size..(max_size + step_size))
			.step_by(step_size as usize)
			.fold(0.0_f64, |total_error, size| {
				total_error + (
					self.get_miss_ratio(size) - curve.get_miss_ratio(size)
				).abs()
			});

		total / num_points as f64
	}

	/// Saves the MRC to a CSV file.
	pub fn to_file(&self, path: &str) -> io::Result<()> {
		let mut writer = CsvWriter::<Point>::from_path(path)?;

		for point in self.into_iter() {
			writer.write_row(point)?;
		}

		Ok(())
	}

	/// Constructs an MRC from a CSV file.
	pub fn from_file(path: &str) -> io::Result<Curve> {
		let mut curve = Curve::default();
		let reader = CsvReader::<Point>::from_path(path)?;

		for point in reader {
			curve.add(point.get_size(), point.get_miss_ratio());
		}

		Ok(curve)
	}
}

impl Point {
	pub fn new(size: u64, miss_ratio: f64) -> Self {
		Point {
			size,
			miss_ratio,
		}
	}

	pub fn get_size(&self) -> u64 {
		self.size
	}

	pub fn get_miss_ratio(&self) -> f64 {
		self.miss_ratio
	}
}

impl ReadRow for Point {
	fn from_row(row: &RowData) -> io::Result<Self> {
		let size = row.get(0)?
			.parse::<u64>()
			.expect("Invalid point size.");

		let miss_ratio = row.get(1)?
			.parse::<f64>()
			.expect("Invalid point miss ratio.");

		let point = Point::new(
			size,
			miss_ratio
		);

		Ok(point)
	}
}

impl WriteRow for Point {
	fn as_row(&self, row: &mut RowData) -> io::Result<()> {
		row.push(self.size.to_string());
		row.push(self.miss_ratio.to_string());

		Ok(())
	}
}

impl<'a> IntoIterator for &'a Curve {
	type Item = &'a Point;
	type IntoIter = Values<'a, u64, Point>;

	fn into_iter(self) -> Self::IntoIter {
		self.points.values()
	}
}
