/*
 * Copyright (c) Kia Shakiba
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use rayon::prelude::*;

use crate::{
	access::{Access, Key},
	algorithm::Algorithm,
	shards::Shards,
	curve::Curve,
	cache::{Cache, CachePolicy},
};

const NUM_CACHES: u32 = 100;

/// The MiniSim MRC generation algorithm.
pub struct Minisimulations {
	max_cache_size: u64,
	caches: Vec<Box<dyn Cache>>,

	shards: Option<Box<dyn Shards>>,
	shards_global_t: u64,
}

impl Algorithm for Minisimulations {
	fn process(&mut self, access: &Access) {
		let shards_global_t = self.shards
			.as_ref()
			.map(|shards| shards.get_global_t())
			.unwrap_or(0);

		if shards_global_t != self.shards_global_t {
			self.rescale(shards_global_t);
			self.shards_global_t = shards_global_t;
		}

		self.caches
			.par_iter_mut()
			.for_each(|cache| {
				cache.handle_self_populating(access);
			});
	}

	fn remove(&mut self, key: Key) {
		self.caches
			.par_iter_mut()
			.for_each(|cache| cache.del(key));
	}

	fn clean(&mut self) {
		self.caches
			.par_iter_mut()
			.for_each(|cache| cache.clear_counters());
	}

	fn resize(&mut self, size: u64) {
		self.caches
			.par_iter_mut()
			.for_each(|cache| cache.reduce(size));
	}

	fn curve(&mut self) -> Curve {
		let mut curve = Curve::new();

		for cache in &self.caches {
			let mut cache_size = cache.size();
			let mut miss_ratio = cache.miss_ratio();

			if let Some(shards) = &self.shards {
				cache_size = shards.unscale(cache_size);

				miss_ratio = (
					(miss_ratio * shards.get_sampled_count() as f64) /
						shards.get_expected_count() as f64
				).clamp(0.0, 1.0);
			}

			curve.add(cache_size, miss_ratio);
		}

		curve
	}

	fn verify_shards(&mut self, access: &Access) -> bool {
		if let Some(ref mut shards) = self.shards {
			if !shards.sample(access) {
				return false;
			}

			if let Some(key) = shards.get_removal() {
				self.remove(key);
			}
		}

		true
	}
}

impl Minisimulations {
	pub fn new(
		policy: &CachePolicy,
		max_cache_size: u64,
		shards: Option<Box<dyn Shards>>,
	) -> Self {
		let caches = get_caches(
			policy,
			max_cache_size,
			NUM_CACHES,
			shards.as_deref(),
		);

		let shards_global_t = shards
			.as_ref()
			.map(|shards| shards.get_global_t())
			.unwrap_or(0);

		Minisimulations {
			max_cache_size,
			caches,

			shards,
			shards_global_t,
		}
	}

	fn rescale(&mut self, shards_new_global_t: u64) {
		let ratio = shards_new_global_t as f64 / self.shards_global_t as f64;
		let num_caches = self.caches.len() as u64;

		let shards_rate = self.shards
			.as_ref()
			.map(|shards| shards.get_rate())
			.unwrap_or(1.0);

		self.caches
			.par_iter_mut()
			.enumerate()
			.for_each(|(index, cache)| {
				let cache_size = (index as u64 + 1) as f64 *
					(self.max_cache_size / num_caches) as f64 *
					shards_rate;

				cache.resize(cache_size as u64);
				cache.rescale(ratio);
			});
	}
}

fn get_caches(
	policy: &CachePolicy,
	max_cache_size: u64,
	num_caches: u32,
	shards: Option<&dyn Shards>
) -> Vec<Box<dyn Cache>> {
	(1..=num_caches)
		.map(|i| {
			let mut cache_size = (i as u64) * (max_cache_size / num_caches as u64);

			if let Some(shards) = shards {
				cache_size = shards.scale(cache_size);
			}

			policy.new_cache(cache_size)
		})
		.collect::<Vec<Box<dyn Cache>>>()
}
