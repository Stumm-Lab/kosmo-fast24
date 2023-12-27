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
		KosmoPolicy,
		eviction_map::{EvictionMap, EvictionMapPolicy},
	},
};

pub struct GlobalObject {
	object: Object,
	eviction_maps: Vec<EvictionMapPolicy>,
}

impl GlobalObject {
	pub fn new(access: &Access, policies: &[KosmoPolicy]) -> Self {
		let eviction_maps = policies
			.iter()
			.map(|policy| EvictionMapPolicy::new(policy, access))
			.collect::<Vec<EvictionMapPolicy>>();

		GlobalObject {
			object: Object::new(access),
			eviction_maps,
		}
	}

	pub fn object(&self) -> &Object {
		&self.object
	}

	pub fn eviction_maps(&self) -> &[EvictionMapPolicy] {
		&self.eviction_maps
	}

	pub fn reuse_distances(&self) -> Vec<Option<u64>> {
		self.eviction_maps
			.iter()
			.map(|eviction_map| Some(eviction_map.reuse_distance(&self.object)))
			.collect::<Vec<Option<u64>>>()
	}

	pub fn update(&mut self, access: &Access) {
		self.object.update(access);

		self.eviction_maps
			.iter_mut()
			.for_each(|eviction_map| eviction_map.update(access));
	}

	pub fn evict_by_policy_index(&mut self, index: usize, cache_size: u64) {
		self.eviction_maps[index].insert(cache_size)
	}

	pub fn exists_at(&self, size: u64) -> bool {
		self.eviction_maps
			.iter()
			.any(|eviction_map| eviction_map.exists_at(size))
	}
}

impl PartialEq for GlobalObject {
	fn eq(&self, other: &Self) -> bool {
		self.object.eq(&other.object)
	}
}

impl Eq for GlobalObject {}
