/*
 * Copyright (c) Kia Shakiba
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::iter::IntoIterator;
use crate::shards::Shards;

pub const BUCKET_SIZE: u64 = 64 * 1024;

pub struct Histogram {
	infinity: Bucket,
	buckets: Vec<Bucket>,
}

pub struct Bucket {
	size: u64,
	count: f64,

	shards_global_t: u64,
}

impl Histogram {
	pub fn new(shards: Option<&dyn Shards>) -> Self {
		let shards_global_t = shards.map(|shards| shards.get_global_t());

		Histogram {
			infinity: Bucket::new(0, shards_global_t),
			buckets: Vec::new(),
		}
	}

	pub fn is_empty(&self) -> bool {
		self.buckets.is_empty()
	}

	pub fn clear(&mut self) {
		self.infinity.clear();
		self.buckets.clear();
	}

	pub fn increment(
		&mut self,
		shards: Option<&dyn Shards>,
		reuse_distance: Option<u64>,
	) {
		let Some(mut reuse_distance) = reuse_distance else {
			if let Some(shards) = shards {
				self.infinity.rescale(shards.get_global_t());
			}

			self.infinity.increment();
			return;
		};

		if let Some(shards) = shards {
			reuse_distance = shards.unscale(reuse_distance);
		}

		reuse_distance = get_rounded_reuse_distance(reuse_distance);

		let search = self.buckets.binary_search_by_key(
			&reuse_distance,
			|bucket| bucket.get_size()
		);

		match search {
			Ok(pos) => {
				if let Some(shards) = shards {
					self.buckets[pos].rescale(shards.get_global_t());
				}

				self.buckets[pos].increment();
			},

			Err(pos) => {
				let shards_global_t = shards.map(|shards| shards.get_global_t());

				self.buckets.insert(pos, Bucket::new(
					reuse_distance,
					shards_global_t,
				));
			},
		}
	}

	pub fn rescale_buckets(&mut self, shards: &dyn Shards) {
		for bucket in &mut self.buckets {
			bucket.rescale(shards.get_global_t());
		}
	}

	pub fn get_total(&self) -> f64 {
		let mut total: f64 = self.infinity.get_count();

		for bucket in &self.buckets {
			total += bucket.get_count();
		}

		total
	}

	pub fn get_corrected_total(&self, shards: &dyn Shards) -> f64 {
		let mut total: f64 = self.infinity.get_count();

		for bucket in &self.buckets {
			total += bucket.get_count();
		}

		total + shards.get_correction() as f64
	}

	pub fn resize(&mut self, size: u64) {
		self.buckets.retain(|bucket| bucket.get_size() <= size);
	}

	pub fn scaled_resize(&mut self, shards: &dyn Shards, size: u64) {
		self.resize(shards.unscale(size));
	}
}

impl Bucket {
	pub fn new(size: u64, shards_global_t: Option<u64>) -> Self {
		let shards_global_t = shards_global_t.unwrap_or(0);

		Bucket {
			size,
			count: 1.0,

			shards_global_t,
		}
	}

	pub fn get_size(&self) -> u64 {
		self.size
	}

	pub fn get_count(&self) -> f64 {
		self.count
	}

	pub fn clear(&mut self) {
		self.count = 1.0;
	}

	pub fn increment(&mut self) {
		self.count += 1.0;
	}

	pub fn rescale(&mut self, global_t: u64) {
		if self.shards_global_t == 0 || self.shards_global_t == global_t {
			return;
		}

		self.count *= global_t as f64 / self.shards_global_t as f64;
		self.shards_global_t = global_t
	}
}

impl<'a> IntoIterator for &'a Histogram {
	type Item = (u64, f64);
	type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

	fn into_iter(self) -> Self::IntoIter {
		Box::new(self.buckets.iter().map(|bucket| {
			(bucket.get_size(), bucket.get_count())
		}))
	}
}

fn get_rounded_reuse_distance(reuse_distance: u64) -> u64 {
	(reuse_distance as f64 / BUCKET_SIZE as f64).ceil() as u64 * BUCKET_SIZE
}
