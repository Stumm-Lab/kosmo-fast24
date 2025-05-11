/*
 * Copyright (c) Kia Shakiba
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

pub mod policy;
mod lru_cache;
mod lfu_cache;
mod fifo_cache;
mod two_q_cache;
mod lrfu_cache;

use crate::access::{Access, Key, Size};
pub use crate::cache::policy::CachePolicy;

/// A cache (used by MiniSim and accurate).
pub trait Cache: Send + Sync {
	/// Returns the cache's size.
	fn size(&self) -> u64;

	/// Returns the cache's miss ratio.
	fn miss_ratio(&self) -> f64;

	fn increment_count(&mut self);
	fn increment_hits(&mut self);

	/// Resets the cache's counters to zero.
	fn clear_counters(&mut self);

	/// Handles one cache access, self-populating if it does not exist
	/// (i.e., read-through).
	fn handle_self_populating(&mut self, access: &Access) -> bool {
		if self.get(access) {
			return true;
		}

		self.set(access);

		false
	}

	/// Performs a get request on the cache.
	fn get(&mut self, access: &Access) -> bool {
		if access.size as u64 > self.size() {
			return false;
		}

		self.increment_count();

		if self.process_get(access) {
			self.increment_hits();

			return true;
		}

		false
	}

	/// Performs a set request on the cache.
	fn set(&mut self, access: &Access) {
		if access.size as u64 > self.size() {
			return;
		}

		self.process_set(access);
	}

	/// Performs a del request on the cache.
	fn del(&mut self, key: Key) {
		self.process_del(key);
	}

	/// Returns `true` if the cache has an object with
	/// the supplied key.
	fn has(&self, key: Key) -> bool {
		self.process_has(key)
	}

	fn process_get(&mut self, _: &Access) -> bool;
	fn process_set(&mut self, _: &Access);
	fn process_del(&mut self, _: Key);
	fn process_has(&self, _: Key) -> bool;

	fn reduce(&mut self, _: u64);
	fn resize(&mut self, _: u64);
	fn rescale(&mut self, _: f64);
}

#[derive(Clone)]
pub struct Object {
	pub key: Key,
	pub size: Size,
}

impl Object {
	fn new(access: &Access) -> Self {
		Object {
			key: access.key,
			size: access.size,
		}
	}
}

impl PartialEq for Object {
	fn eq(&self, other: &Self) -> bool {
		self.key == other.key
	}
}

impl Eq for Object {}

pub use crate::{
	cache::lfu_cache::*,
	cache::fifo_cache::*,
	cache::two_q_cache::*,
	cache::lru_cache::*,
	cache::lrfu_cache::*,
};
