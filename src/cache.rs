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

use crate::access::{Access, Command, Key, Size};
pub use crate::cache::policy::CachePolicy;

pub trait Cache: Send + Sync {
	fn size(&self) -> u64;
	fn miss_ratio(&self) -> f64;

	fn increment_count(&mut self);
	fn increment_hits(&mut self);

	fn clear_counters(&mut self);

	fn handle_self_populating(&mut self, access: &Access) -> bool {
		if self.get(access) {
			return true;
		}

		self.set(access);

		false
	}

	fn handle_lookaside(&mut self, access: &Access) {
		match access.command {
			Command::Get => {
				self.get(access);
			},

			Command::Set => {
				self.set(access);
			},
		}
	}

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

	fn set(&mut self, access: &Access) {
		if access.size as u64 > self.size() {
			return;
		}

		self.process_set(access);
	}

	fn del(&mut self, key: Key) {
		self.process_del(key);
	}

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
