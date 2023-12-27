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
		local_object::{LocalObjectPolicy, LrfuLocalObject},
	},
};

pub struct LrfuEvictionMap {
	p: f64,
	lambda: f64,

	timestamp: Timestamp,
	map: Vec<EvictionRecord>,
}

struct EvictionRecord {
	size: u64,
	crf: f64,
}

impl EvictionMap for LrfuEvictionMap {
	fn insert(&mut self, size: u64) {
		if self.map.last().is_some_and(|record| record.size > size) {
			return;
		}

		let mut updated_crf = f(self.p, self.lambda, 0);

		while self.map.last().is_some_and(|record| record.size <= size) {
			if let Some(record) = self.map.pop() {
				updated_crf = record.crf;
			}
		}

		if self.map.is_empty() || self.map.last().is_some_and(|record| record.size != size + 1) {
			self.map.push(EvictionRecord::new(size + 1, updated_crf));
		}
	}

	fn exists_at(&self, size: u64) -> bool {
		self.crf_at(size).is_some()
	}

	fn reuse_distance(&self, object: &Object) -> u64 {
		match self.map.last() {
			Some(record) => cmp::max(record.size, object.size as u64),
			None => object.size as u64,
		}
	}

	fn update(&mut self, access: &Access) {
		self.map.iter_mut().for_each(|record| record.update(self.timestamp, access, self.p, self.lambda));
		self.timestamp = access.timestamp;

		if !self.map.last().is_some_and(|record| record.size == 0) {
			self.map.push(EvictionRecord::new(0, f(self.p, self.lambda, 0)));
		}
	}

	fn as_local_object<'a>(
		&self,
		global_object: &'a GlobalObject,
		cache_size: u64,
	) -> LocalObjectPolicy<'a> {
		let local_object = LrfuLocalObject::new(
			global_object,
			self.crf_at(cache_size)
		);

		LocalObjectPolicy::Lrfu(local_object)
	}
}

impl LrfuEvictionMap {
	pub fn new(access: &Access, p: f64, lambda: f64) -> Self {
		assert!(p >= 2.0);
		assert!(lambda >= 0.0);
		assert!(lambda <= 1.0);

		LrfuEvictionMap {
			p,
			lambda,

			timestamp: access.timestamp,
			map: vec![EvictionRecord::new(0, f(p, lambda, 0))],
		}
	}

	pub fn crf_at(&self, size: u64) -> Option<f64> {
		let mut crf: Option<f64> = None;

		for record in self.map.iter().rev() {
			if record.size == size {
				return Some(record.crf);
			}

			if record.size > size {
				return crf;
			}

			crf = Some(record.crf);
		}

		crf
	}
}

impl EvictionRecord {
	fn new(size: u64, crf: f64) -> Self {
		EvictionRecord {
			size,
			crf,
		}
	}

	fn update(
		&mut self,
		last_timestamp: Timestamp,
		access: &Access,
		p: f64,
		lambda: f64,
	) {
		self.crf = f(p, lambda, 0) + f(p, lambda, access.timestamp - last_timestamp) * self.crf;
	}
}

fn f(p: f64, lambda: f64, x: u64) -> f64 {
	(1.0 / p).powf(lambda * x as f64)
}

#[cfg(test)]
mod tests {
	#[test]
	fn crf_is_correct() {
		use crate::access::{Access, Command};
		use crate::kosmo::eviction_map::{EvictionMap, LrfuEvictionMap};

		let mut access = Access {
			timestamp: 1,
			command: Command::Get,
			key: 0,
			size: 1,
			ttl: 0,
		};

		let mut eviction_map = LrfuEvictionMap::new(&access, 2.0, 0.5);
		assert_eq!(eviction_map.crf_at(1), Some(1.0));

		access.timestamp += 1;
		eviction_map.update(&access);
		assert_eq!(eviction_map.crf_at(1), Some(1.7071067811865475));

		eviction_map.insert(5);
		assert_eq!(eviction_map.crf_at(4), None);
		assert_eq!(eviction_map.crf_at(5), None);
		assert_eq!(eviction_map.crf_at(6), Some(1.7071067811865475));

		access.timestamp += 1;
		eviction_map.update(&access);
		assert_eq!(eviction_map.crf_at(4), Some(1.0));
		assert_eq!(eviction_map.crf_at(5), Some(1.0));
		assert_eq!(eviction_map.crf_at(6), Some(2.2071067811865475));
	}
}
