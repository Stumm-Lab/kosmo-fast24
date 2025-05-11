/*
 * Copyright (c) Kia Shakiba
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::{
	access::Access,
	algorithm::Object,
	kosmo::{
		eviction_map::EvictionMap,
		global_object::GlobalObject,
		local_object::{LocalObjectPolicy, LfuLocalObject},
	},
};

pub struct LfuEvictionMap {
	global_count: u64,
	map: Vec<EvictionRecord>,
}

struct EvictionRecord {
	size: u64,
	count: u64,
}

impl EvictionMap for LfuEvictionMap {
	fn insert(&mut self, size: u64) {
		while self.map.last().is_some_and(|record| record.size <= size) {
			self.map.pop();
		}

		self.map.push(EvictionRecord::new(size, self.global_count));
	}

	fn exists_at(&self, size: u64) -> bool {
		self.count_at(size).is_some()
	}

	fn reuse_distance(&self, object: &Object) -> u64 {
		for record in self.map.iter().rev() {
			if record.count == self.global_count {
				return record.size + 1;
			}
		}

		object.size as u64
	}

	fn update(&mut self, _: &Access) {
		self.global_count += 1;
	}

	fn as_local_object<'a>(
		&self,
		global_object: &'a GlobalObject,
		cache_size: u64,
	) -> LocalObjectPolicy<'a> {
		let local_object = LfuLocalObject::new(
			global_object,
			self.count_at(cache_size)
		);

		LocalObjectPolicy::Lfu(local_object)
	}
}

impl LfuEvictionMap {
	pub fn new() -> Self {
		LfuEvictionMap {
			global_count: 1,
			map: Vec::new(),
		}
	}

	pub fn count_at(&self, size: u64) -> Option<u64> {
		for record in self.map.iter().rev() {
			if record.size >= size {
				return match self.global_count - record.count {
					0 => None,
					count => Some(count),
				};
			}
		}

		Some(self.global_count)
	}
}

impl EvictionRecord {
	fn new(size: u64, count: u64) -> Self {
		EvictionRecord {
			size,
			count,
		}
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn count_is_correct() {
		use crate::access::{Access, Command};
		use crate::kosmo::eviction_map::{EvictionMap, LfuEvictionMap};

		let access = Access {
			timestamp: 0,
			command: Command::Get,
			key: 0,
			size: 1,
			ttl: None,
		};

		let mut eviction_map = LfuEvictionMap::new();
		assert_eq!(eviction_map.count_at(1), Some(1));

		eviction_map.update(&access);
		assert_eq!(eviction_map.count_at(1), Some(2));

		eviction_map.insert(5);
		assert_eq!(eviction_map.count_at(4), None);
		assert_eq!(eviction_map.count_at(5), None);
		assert_eq!(eviction_map.count_at(6), Some(2));

		eviction_map.update(&access);
		assert_eq!(eviction_map.count_at(4), Some(1));
		assert_eq!(eviction_map.count_at(5), Some(1));
		assert_eq!(eviction_map.count_at(6), Some(3));
	}
}
