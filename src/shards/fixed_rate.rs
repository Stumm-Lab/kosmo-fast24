/*
 * Copyright (c) Kia Shakiba
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::{
	shards::Shards,
	access::Access,
};

pub struct ShardsFixedRate {
	global_t: u64,

	sampled_count: u64,
	total_count: u64,
}

impl ShardsFixedRate {
	#[allow(dead_code)]
	pub fn new(global_t: u64) -> Self {
		ShardsFixedRate {
			global_t,

			sampled_count: 0,
			total_count: 0,
		}
	}
}

impl Shards for ShardsFixedRate {
	fn get_global_t(&self) -> u64 {
		self.global_t
	}

	fn get_sampled_count(&self) -> u64 {
		self.sampled_count
	}

	fn get_total_count(&self) -> u64 {
		self.total_count
	}

	fn get_expected_count(&self) -> u64 {
		(self.get_rate() * self.total_count as f64) as u64
	}

	fn sample(&mut self, access: &Access) -> bool {
		self.total_count += 1;

		if self.sample_key(access.key).is_none() {
			return false;
		}

		self.sampled_count += 1;

		true
	}
}
