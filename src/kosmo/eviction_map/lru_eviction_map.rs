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
		local_object::{LocalObjectPolicy, LruLocalObject},
	},
};

pub struct LruEvictionMap {
	evicted_size: u64,
}

impl EvictionMap for LruEvictionMap {
	fn insert(&mut self, size: u64) {
		self.evicted_size = size;
	}

	fn exists_at(&self, size: u64) -> bool {
		self.evicted_size < size
	}

	fn reuse_distance(&self, object: &Object) -> u64 {
		if self.evicted_size > 0 {
			self.evicted_size
		} else {
			object.size as u64
		}
	}

	fn update(&mut self, access: &Access) {
		self.evicted_size = access.size as u64 - 1;
	}

	fn as_local_object<'a>(
		&self,
		global_object: &'a GlobalObject,
		cache_size: u64,
	) -> LocalObjectPolicy<'a> {
		let local_object = LruLocalObject::new(
			global_object,
			self.exists_at(cache_size)
		);

		LocalObjectPolicy::Lru(local_object)
	}
}

impl LruEvictionMap {
	pub fn new(access: &Access) -> Self {
		LruEvictionMap {
			evicted_size: access.size as u64 - 1,
		}
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn exists_is_correct() {
		use crate::access::{Access, Command};
		use crate::kosmo::eviction_map::{EvictionMap, LruEvictionMap};

		let mut access = Access {
			timestamp: 0,
			command: Command::Get,
			key: 0,
			size: 1,
			ttl: 0,
		};

		let mut eviction_map = LruEvictionMap::new(&access);
		assert!(!eviction_map.exists_at(0));
		assert!(eviction_map.exists_at(1));

		eviction_map.insert(10);
		assert!(!eviction_map.exists_at(5));
		assert!(!eviction_map.exists_at(10));
		assert!(eviction_map.exists_at(11));

		access.size = 4;
		eviction_map.update(&access);
		assert!(!eviction_map.exists_at(1));
		assert!(eviction_map.exists_at(4));
		assert!(eviction_map.exists_at(5));
	}
}
