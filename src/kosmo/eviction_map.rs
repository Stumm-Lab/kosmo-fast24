/*
 * Copyright (c) Kia Shakiba
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

mod lfu_eviction_map;
mod fifo_eviction_map;
mod two_q_eviction_map;
mod lrfu_eviction_map;
mod lru_eviction_map;

use crate::{
	access::Access,
	algorithm::Object,
};

pub use crate::kosmo::{
	KosmoPolicy,
	eviction_map::{
		lfu_eviction_map::LfuEvictionMap,
		fifo_eviction_map::FifoEvictionMap,
		two_q_eviction_map::TwoQEvictionMap,
		lrfu_eviction_map::LrfuEvictionMap,
		lru_eviction_map::LruEvictionMap,
	},
	global_object::GlobalObject,
	local_object::LocalObjectPolicy,
};

pub trait EvictionMap {
	fn insert(&mut self, _: u64);

	fn exists_at(&self, _: u64) -> bool;
	fn reuse_distance(&self, _: &Object) -> u64;

	fn update(&mut self, _: &Access) {}

	fn as_local_object<'a>(&self, _: &'a GlobalObject, _: u64) -> LocalObjectPolicy<'a>;
}

pub enum EvictionMapPolicy {
	Lfu(LfuEvictionMap),
	Fifo(FifoEvictionMap),
	TwoQ(TwoQEvictionMap),
	Lrfu(LrfuEvictionMap),
	Lru(LruEvictionMap),
}

impl EvictionMapPolicy {
	pub fn new(policy: &KosmoPolicy, access: &Access) -> Self {
		match policy {
			KosmoPolicy::Lfu => EvictionMapPolicy::Lfu(
				LfuEvictionMap::new()
			),

			KosmoPolicy::Fifo => EvictionMapPolicy::Fifo(
				FifoEvictionMap::new(access)
			),

			KosmoPolicy::TwoQ(kin, kout) => EvictionMapPolicy::TwoQ(
				TwoQEvictionMap::new(access, *kin, *kout)
			),

			KosmoPolicy::Lrfu(p, lambda) => EvictionMapPolicy::Lrfu(
				LrfuEvictionMap::new(access, *p, *lambda)
			),

			KosmoPolicy::Lru => EvictionMapPolicy::Lru(
				LruEvictionMap::new(access)
			),
		}
	}
}

impl EvictionMap for EvictionMapPolicy {
	fn insert(&mut self, size: u64) {
		match self {
			EvictionMapPolicy::Lfu(eviction_map) => eviction_map.insert(size),
			EvictionMapPolicy::Fifo(eviction_map) => eviction_map.insert(size),
			EvictionMapPolicy::TwoQ(eviction_map) => eviction_map.insert(size),
			EvictionMapPolicy::Lrfu(eviction_map) => eviction_map.insert(size),
			EvictionMapPolicy::Lru(eviction_map) => eviction_map.insert(size),
		}
	}

	fn exists_at(&self, size: u64) -> bool {
		match self {
			EvictionMapPolicy::Lfu(eviction_map) => eviction_map.exists_at(size),
			EvictionMapPolicy::Fifo(eviction_map) => eviction_map.exists_at(size),
			EvictionMapPolicy::TwoQ(eviction_map) => eviction_map.exists_at(size),
			EvictionMapPolicy::Lrfu(eviction_map) => eviction_map.exists_at(size),
			EvictionMapPolicy::Lru(eviction_map) => eviction_map.exists_at(size),
		}
	}

	fn reuse_distance(&self, object: &Object) -> u64 {
		match self {
			EvictionMapPolicy::Lfu(eviction_map) => eviction_map.reuse_distance(object),
			EvictionMapPolicy::Fifo(eviction_map) => eviction_map.reuse_distance(object),
			EvictionMapPolicy::TwoQ(eviction_map) => eviction_map.reuse_distance(object),
			EvictionMapPolicy::Lrfu(eviction_map) => eviction_map.reuse_distance(object),
			EvictionMapPolicy::Lru(eviction_map) => eviction_map.reuse_distance(object),
		}
	}

	fn update(&mut self, access: &Access) {
		match self {
			EvictionMapPolicy::Lfu(eviction_map) => eviction_map.update(access),
			EvictionMapPolicy::Fifo(eviction_map) => eviction_map.update(access),
			EvictionMapPolicy::TwoQ(eviction_map) => eviction_map.update(access),
			EvictionMapPolicy::Lrfu(eviction_map) => eviction_map.update(access),
			EvictionMapPolicy::Lru(eviction_map) => eviction_map.update(access),
		}
	}

	fn as_local_object<'a>(
		&self,
		global_object: &'a GlobalObject,
		cache_size: u64,
	) -> LocalObjectPolicy<'a> {
		match self {
			EvictionMapPolicy::Lfu(eviction_map) =>
				eviction_map.as_local_object(global_object, cache_size),

			EvictionMapPolicy::Fifo(eviction_map) =>
				eviction_map.as_local_object(global_object, cache_size),

			EvictionMapPolicy::TwoQ(eviction_map) =>
				eviction_map.as_local_object(global_object, cache_size),

			EvictionMapPolicy::Lrfu(eviction_map) =>
				eviction_map.as_local_object(global_object, cache_size),

			EvictionMapPolicy::Lru(eviction_map) =>
				eviction_map.as_local_object(global_object, cache_size),
		}
	}
}
