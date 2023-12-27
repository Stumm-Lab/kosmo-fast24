/*
 * Copyright (c) Kia Shakiba
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::BTreeSet;
use std::cmp::{Ord, Ordering};
use rustc_hash::FxHashMap;

use crate::{
	cache::{Cache, Object},
	access::{Access, Timestamp, Key},
};

pub struct LrfuCache {
	max_size: u64,
	current_size: u64,

	count: f64,
	hits: f64,

	map: FxHashMap<Key, LrfuObject>,
	stack: BTreeSet<LrfuObject>,

	p: f64,
	lambda: f64,

	intrinsic_timestamp: Timestamp,
}

#[derive(Clone)]
struct LrfuObject {
	object: Object,

	last_access: Timestamp,
	crf: f64,
}

impl LrfuCache {
	pub fn new(size: u64, p: f64, lambda: f64) -> Self {
		assert!(p >= 2.0);
		assert!(lambda >= 0.0);
		assert!(lambda <= 1.0);

		LrfuCache {
			max_size: size,
			current_size: 0,

			count: 0.0,
			hits: 0.0,

			map: FxHashMap::default(),
			stack: BTreeSet::new(),

			p,
			lambda,

			intrinsic_timestamp: 0,
		}
	}

	fn get_updated_crf(
		&self,
		now: Timestamp,
		lrfu_object: &LrfuObject
	) -> f64 {
		let last_access = lrfu_object.last_access;
		let last_crf = lrfu_object.crf;

		self.f(0) + self.f(now - last_access) * last_crf
	}

	fn f(&self, x: u64) -> f64 {
		(1.0 / self.p).powf(self.lambda * x as f64)
	}
}

impl Cache for LrfuCache {
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
		self.intrinsic_timestamp += 1;

		if let Some(mut lrfu_object) = self.map.remove(&access.key) {
			self.stack.remove(&lrfu_object);

			let crf = self.get_updated_crf(self.intrinsic_timestamp, &lrfu_object);
			lrfu_object.update(self.intrinsic_timestamp, crf);

			self.map.insert(access.key, lrfu_object.clone());
			self.stack.insert(lrfu_object);

			return true;
		}

		false
	}

	fn process_set(&mut self, access: &Access) {
		self.intrinsic_timestamp += 1;

		if access.size as u64 > self.max_size || self.has(access.key) {
			return;
		}

		self.reduce(self.max_size - access.size as u64);

		let lrfu_object = LrfuObject::new(
			access,
			self.intrinsic_timestamp,
			self.f(0)
		);

		self.map.insert(access.key, lrfu_object.clone());
		self.stack.insert(lrfu_object);

		self.current_size += access.size as u64;
	}

	fn process_del(&mut self, key: Key) {
		self.intrinsic_timestamp += 1;

		if let Some(lrfu_object) = self.map.remove(&key) {
			self.current_size -= lrfu_object.object.size as u64;
			self.stack.remove(&lrfu_object);
		}
	}

	fn process_has(&self, key: Key) -> bool {
		self.map.contains_key(&key)
	}

	fn reduce(&mut self, target_size: u64) {
		while self.current_size > target_size {
			if let Some(lrfu_object) = self.stack.pop_last() {
				self.map.remove(&lrfu_object.object.key);
				self.current_size -= lrfu_object.object.size as u64;
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

impl LrfuObject {
	fn new(access: &Access, now: Timestamp, f0: f64) -> Self {
		LrfuObject {
			object: Object::new(access),

			last_access: now,
			crf: f0,
		}
	}

	fn update(&mut self, now: Timestamp, crf: f64) {
		self.last_access = now;
		self.crf = crf;
	}
}

impl Ord for LrfuObject {
	fn cmp(&self, other: &Self) -> Ordering {
		if self.object.eq(&other.object) {
			return Ordering::Equal;
		}

		match other.crf.total_cmp(&self.crf) {
			Ordering::Equal => other.last_access.cmp(&self.last_access),
			ordering => ordering,
		}
	}
}

impl PartialOrd for LrfuObject {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl PartialEq for LrfuObject {
	fn eq(&self, other: &Self) -> bool {
		self.object.eq(&other.object)
	}
}

impl Eq for LrfuObject {}
