/*
 * Copyright (c) Kia Shakiba
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::{
	io::{Error, ErrorKind},
	collections::BTreeMap,
	collections::btree_map::Values,
	ops::Bound,
	iter::IntoIterator,
};

use kwik::{
	math,
	csv_writer::{
		FileWriter,
		CsvWriter,
		CsvRow as WriterCsvRow,
		Row as WriterRow,
	},
	csv_reader::{
		FileReader,
		CsvReader,
		CsvRow as ReaderCsvRow,
		Row as ReaderRow,
	},
};

use crate::{
	histogram::Histogram,
	shards::Shards,
};

#[derive(Clone, Default)]
pub struct Curve {
	points: BTreeMap<u64, Point>,
}

#[derive(Clone)]
pub struct Point {
	size: u64,
	miss_ratio: f64,
}

impl Curve {
	pub fn new() -> Self {
		Curve {
			points: BTreeMap::new(),
		}
	}

	pub fn size(&self) -> usize {
		self.points.len()
	}

	pub fn is_empty(&self) -> bool {
		self.points.is_empty()
	}

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

	pub fn get_max_size(&self) -> u64 {
		match self.points.last_key_value() {
			Some((size, _)) => *size,
			None => 0,
		}
	}

	pub fn get_miss_ratio(&self, size: u64) -> f64 {
		if self.points.is_empty() {
			return 1.0;
		}

		let prev_cursor = self.points.upper_bound(Bound::Included(&size));
		let next_cursor = self.points.lower_bound(Bound::Included(&size));

		match (prev_cursor.key_value(), next_cursor.key_value()) {
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
				None => 0.0,
			},
		}
	}

	pub fn add(&mut self, size: u64, miss_ratio: f64) {
		self.points.insert(
			size,
			Point::new(size, miss_ratio)
		);
	}

	pub fn mae(&self, curve: &Curve) -> f64 {
		let max_size = math::max(&[
			self.get_max_size(),
			curve.get_max_size(),
		]);

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

	pub fn to_file(&self, path: &str) -> Result<(), Error> {
		let mut writer = CsvWriter::<Point>::new(path)?;

		for point in self.into_iter() {
			writer.write_row(point);
		}

		Ok(())
	}

	pub fn from_file(path: &str) -> Result<Curve, Error> {
		let mut curve = Curve::default();
		let mut reader = CsvReader::<Point>::new(path)?;

		while let Some(point) = reader.read_row() {
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

impl WriterRow for Point {
	fn as_row(&self, row: &mut WriterCsvRow) -> Result<(), Error> {
		row.push(&self.size.to_string());
		row.push(&self.miss_ratio.to_string());

		Ok(())
	}
}

impl ReaderRow for Point {
	fn new(row: &ReaderCsvRow) -> Result<Self, Error> where Self: Sized {
		let Ok(size) = row.get(0)?.parse::<u64>() else {
			return Err(Error::new(
				ErrorKind::InvalidData,
				"Invalid curve point size."
			));
		};

		let Ok(miss_ratio) = row.get(1)?.parse::<f64>() else {
			return Err(Error::new(
				ErrorKind::InvalidData,
				"Invalid curve point miss ratio."
			));
		};

		let point = Point::new(
			size,
			miss_ratio
		);

		Ok(point)
	}
}

impl<'a> IntoIterator for &'a Curve {
	type Item = &'a Point;
	type IntoIter = Values<'a, u64, Point>;

	fn into_iter(self) -> Self::IntoIter {
		self.points.values()
	}
}
