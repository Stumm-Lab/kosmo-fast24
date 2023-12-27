/*
 * Copyright (c) Kia Shakiba
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::cmp;

use crate::{
	access::{Access, Timestamp},
	algorithm::Object,
	kosmo::{
		eviction_map::EvictionMap,
		global_object::GlobalObject,
		local_object::{LocalObjectPolicy, FifoLocalObject},
	},
};

pub struct FifoEvictionMap {
	map: Vec<EvictionRecord>,
}

struct EvictionRecord {
	size: u64,
	timestamp: Timestamp,
}

impl EvictionMap for FifoEvictionMap {
	fn insert(&mut self, size: u64) {
		if self.map.last().is_some_and(|record| record.size > size) {
			return;
		}

		let mut updated_timestamp: Timestamp = 0;

		if self.map.last().is_some_and(|record| record.size <= size) {
			if let Some(record) = self.map.pop() {
				updated_timestamp = record.timestamp;
			}
		}

		while self.map.last().is_some_and(|record| record.size <= size) {
			self.map.pop();
		}

		let should_insert = match self.map.last() {
			Some(record) => record.size != size + 1,
			None => true,
		};

		if should_insert {
			self.map.push(
				EvictionRecord::new(size + 1, updated_timestamp)
			);
		}
	}

	fn exists_at(&self, size: u64) -> bool {
		self.timestamp_at(size).is_some()
	}

	fn reuse_distance(&self, object: &Object) -> u64 {
		match self.map.last() {
			Some(record) => cmp::max(record.size, object.size as u64),
			None => object.size as u64,
		}
	}

	fn update(&mut self, access: &Access) {
		let should_insert = match self.map.last() {
			Some(record) => record.size != 0,
			None => true,
		};

		if should_insert {
			self.map.push(
				EvictionRecord::new(0, access.timestamp)
			);
		}
	}

	fn as_local_object<'a>(
		&self,
		global_object: &'a GlobalObject,
		cache_size: u64,
	) -> LocalObjectPolicy<'a> {
		let local_object = FifoLocalObject::new(
			global_object,
			self.timestamp_at(cache_size)
		);

		LocalObjectPolicy::Fifo(local_object)
	}
}

impl FifoEvictionMap {
	pub fn new(access: &Access) -> Self {
		FifoEvictionMap {
			map: vec![EvictionRecord::new(0, access.timestamp)],
		}
	}

	pub fn timestamp_at(&self, size: u64) -> Option<Timestamp> {
		let mut timestamp: Option<Timestamp> = None;

		for record in self.map.iter().rev() {
			if record.size == size {
				return Some(record.timestamp);
			}

			if record.size > size {
				return timestamp;
			}

			timestamp = Some(record.timestamp);
		}

		timestamp
	}
}

impl EvictionRecord {
	fn new(size: u64, timestamp: Timestamp) -> Self {
		EvictionRecord {
			size,
			timestamp,
		}
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn timestamp_is_correct() {
		use crate::access::{Access, Command};
		use crate::kosmo::eviction_map::{EvictionMap, FifoEvictionMap};

		let mut access = Access {
			timestamp: 1,
			command: Command::Get,
			key: 0,
			size: 1,
			ttl: 0,
		};

		let mut eviction_map = FifoEvictionMap::new(&access);
		assert_eq!(eviction_map.timestamp_at(1), Some(1));

		access.timestamp += 1;
		eviction_map.update(&access);
		assert_eq!(eviction_map.timestamp_at(1), Some(1));

		eviction_map.insert(5);
		assert_eq!(eviction_map.timestamp_at(4), None);
		assert_eq!(eviction_map.timestamp_at(5), None);
		assert_eq!(eviction_map.timestamp_at(6), Some(1));

		access.timestamp += 1;
		eviction_map.update(&access);
		assert_eq!(eviction_map.timestamp_at(4), Some(3));
		assert_eq!(eviction_map.timestamp_at(5), Some(3));
		assert_eq!(eviction_map.timestamp_at(6), Some(1));
	}
}
