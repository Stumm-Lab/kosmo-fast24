/*
 * Copyright (c) Kia Shakiba
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

mod fixed_rate;
mod fixed_size;

use fasthash::murmur3;
use crate::access::{Access, Key};

const MODULUS: u64 = 16777216;

pub trait Shards {
	fn get_global_t(&self) -> u64;
	fn get_sampled_count(&self) -> u64;
	fn get_total_count(&self) -> u64;
	fn get_expected_count(&self) -> u64;

	fn get_correction(&self) -> i64 {
		self.get_expected_count() as i64 - self.get_sampled_count() as i64
	}

	fn get_rate(&self) -> f64 {
		self.get_global_t() as f64 / MODULUS as f64
	}

	fn sample(&mut self, access: &Access) -> bool;

	fn sample_key(&self, key: Key) -> Option<u64> {
		let t = (hash(key) % MODULUS as u128) as u64;

		match t < self.get_global_t() {
			true => Some(t),
			false => None,
		}
	}

	fn scale(&self, size: u64) -> u64 {
		(size as f64 * self.get_rate()) as u64
	}

	fn unscale(&self, size: u64) -> u64 {
		(size as f64 / self.get_rate()) as u64
	}

	fn get_removal(&mut self) -> Option<Key> {
		None
	}
}

fn hash(key: Key) -> u128 {
	murmur3::hash128(key.to_le_bytes())
}

#[allow(unused_imports)]
pub use crate::{
	shards::fixed_rate::*,
	shards::fixed_size::*,
};
