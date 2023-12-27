/*
 * Copyright (c) Kia Shakiba
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use rustc_hash::FxHashMap;
use dlv_list::{VecList, Index};

use crate::{
	cache::{Cache, Object},
	access::{Access, Key},
};

pub struct FifoCache {
	max_size: u64,
	current_size: u64,

	count: f64,
	hits: f64,

	map: FxHashMap<Key, Index<Object>>,
	stack: VecList<Object>,
}

impl FifoCache {
	pub fn new(size: u64) -> Self {
		FifoCache {
			max_size: size,
			current_size: 0,

			count: 0.0,
			hits: 0.0,

			map: FxHashMap::default(),
			stack: VecList::new(),
		}
	}
}

impl Cache for FifoCache {
	fn size(&self) -> u64 {
		self.max_size
	}

	fn miss_ratio(&self) -> f64 {
		if self.count > 0.0 {
			return 1.0 - self.hits / self.count;
		}

		0.0
	}

	fn increment_count(&mut self) {
		self.count += 1.0
	}

	fn increment_hits(&mut self) {
		self.hits += 1.0
	}

	fn clear_counters(&mut self) {
		self.count = 0.0;
		self.hits = 0.0;
	}

	fn process_get(&mut self, access: &Access) -> bool {
		self.process_has(access.key)
	}

	fn process_set(&mut self, access: &Access) {
		if access.size as u64 > self.max_size || self.has(access.key) {
			return;
		}

		self.reduce(self.max_size - access.size as u64);

		let object = Object::new(access);
		let index = self.stack.push_front(object);

		self.map.insert(access.key, index);
		self.current_size += access.size as u64;
	}

	fn process_del(&mut self, key: Key) {
		let Some(index) = self.map.remove(&key) else {
			return;
		};

		let object = self.stack.remove(index).unwrap();
		self.current_size -= object.size as u64;
	}

	fn process_has(&self, key: Key) -> bool {
		self.map.contains_key(&key)
	}

	fn reduce(&mut self, target_size: u64) {
		while self.current_size > target_size {
			if let Some(object) = self.stack.pop_back() {
				self.map.remove(&object.key);
				self.current_size -= object.size as u64;
			}
		}
	}

	fn resize(&mut self, size: u64) {
		self.reduce(size);
		self.max_size = size;
	}

	fn rescale(&mut self, ratio: f64) {
		self.count *= ratio;
		self.hits *= ratio;
	}
}
