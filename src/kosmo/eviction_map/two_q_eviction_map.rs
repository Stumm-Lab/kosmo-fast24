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
		local_object::{
			LocalObjectPolicy,
			TwoQLocalObject,
			two_q_local_object::StackLocation,
		},
	},
};

pub struct TwoQEvictionMap {
	kin: f64,
	kout: f64,

	fifo_map: Vec<FifoEvictionRecord>,

	lfu_global_count: u64,
	lfu_map: Vec<LfuEvictionRecord>,
}

struct FifoEvictionRecord {
	size: u64,
	timestamp: Timestamp,
}

struct LfuEvictionRecord {
	size: u64,
	count: u64,
}

impl EvictionMap for TwoQEvictionMap {
	fn insert(&mut self, size: u64) {
		self.insert_fifo(size);
		self.insert_lfu(size);
	}

	fn exists_at(&self, size: u64) -> bool {
		self.stack_location_at(size).is_some()
	}

	fn reuse_distance(&self, object: &Object) -> u64 {
		let smallest_a1 = match self.fifo_map.last() {
			Some(record) => (cmp::max(record.size, object.size as u64) as f64 / (self.kin + self.kout)) as u64,
			None => (object.size as f64 / (self.kin + self.kout)) as u64,
		};

		let smallest_am = self.lfu_map
			.iter()
			.rev()
			.find(|record| self.lfu_global_count - record.count >= 2)
			.map(|record| record.size);

		match smallest_am {
			Some(smallest_am) => cmp::min(smallest_a1, smallest_am),
			None => smallest_a1,
		}
	}

	fn update(&mut self, access: &Access) {
		self.lfu_global_count += 1;

		let fifo_should_insert = match self.fifo_map.last() {
			Some(record) => record.size != 0,
			None => true,
		};

		if fifo_should_insert {
			self.fifo_map.push(
				FifoEvictionRecord::new(0, access.timestamp)
			);
		}
	}

	fn as_local_object<'a>(
		&self,
		global_object: &'a GlobalObject,
		cache_size: u64,
	) -> LocalObjectPolicy<'a> {
		let local_object = TwoQLocalObject::new(
			global_object,
			self.stack_location_at(cache_size),
		);

		LocalObjectPolicy::TwoQ(local_object)
	}
}

impl TwoQEvictionMap {
	pub fn new(access: &Access, kin: f64, kout: f64) -> Self {
		assert!(kin > 0.0);
		assert!(kout > 0.0);
		assert!(kin + kout <= 1.0);

		TwoQEvictionMap {
			kin,
			kout,

			fifo_map: vec![FifoEvictionRecord::new(0, access.timestamp)],

			lfu_global_count: 1,
			lfu_map: Vec::new(),
		}
	}

	fn ain_size(&self, size: u64) -> u64 {
		(size as f64 * self.kin) as u64
	}

	fn aout_size(&self, size: u64) -> u64 {
		(size as f64 * self.kout) as u64
	}

	fn a1_size(&self, size: u64) -> u64 {
		self.ain_size(size) + self.aout_size(size)
	}

	fn insert_fifo(&mut self, size: u64) {
		let size = self.a1_size(size);

		if self.fifo_map.last().is_some_and(|record| record.size > size) {
			return;
		}

		let mut updated_timestamp: Timestamp = 0;

		if self.fifo_map.last().is_some_and(|record| record.size <= size) {
			if let Some(record) = self.fifo_map.pop() {
				updated_timestamp = record.timestamp;
			}
		}

		while self.fifo_map.last().is_some_and(|record| record.size <= size) {
			self.fifo_map.pop();
		}

		let fifo_should_insert = match self.fifo_map.last() {
			Some(record) => record.size != size + 1,
			None => true,
		};

		if fifo_should_insert {
			self.fifo_map.push(
				FifoEvictionRecord::new(size + 1, updated_timestamp)
			);
		}
	}

	fn insert_lfu(&mut self, size: u64) {
		while self.lfu_map.last().is_some_and(|record| record.size <= size) {
			self.lfu_map.pop();
		}

		self.lfu_map.push(LfuEvictionRecord::new(size, self.lfu_global_count));
	}

	pub fn stack_location_at(&self, size: u64) -> Option<StackLocation> {
		let ain_size = self.ain_size(size);
		let a1_size = self.a1_size(size);

		let mut ain_timestamp: Option<Timestamp> = None;
		let mut aout_timestamp: Option<Timestamp> = None;

		for record in self.fifo_map.iter().rev() {
			if record.size > a1_size {
				break;
			}

			if record.size == a1_size {
				aout_timestamp = Some(record.timestamp);
				break;
			}

			if record.size > ain_size {
				aout_timestamp = Some(record.timestamp);
			} else {
				ain_timestamp = Some(record.timestamp);
				aout_timestamp = Some(record.timestamp);
			}
		}

		let Some(aout_timestamp) = aout_timestamp else {
			return ain_timestamp.map(StackLocation::A1);
		};

		let am_exists = self.lfu_map
			.iter()
			.any(|record| {
				record.size > ain_size
					&& record.size <= a1_size
					&& self.lfu_global_count - record.count >= 2
			});

		match am_exists {
			true => Some(StackLocation::Am),
			false => Some(StackLocation::A1(aout_timestamp)),
		}
	}
}

impl FifoEvictionRecord {
	fn new(size: u64, timestamp: Timestamp) -> Self {
		FifoEvictionRecord {
			size,
			timestamp,
		}
	}
}

impl LfuEvictionRecord {
	fn new(size: u64, count: u64) -> Self {
		LfuEvictionRecord {
			size,
			count,
		}
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn stack_location_is_correct() {
		use crate::{
			access::{Access, Command},
			kosmo::{
				eviction_map::{EvictionMap, TwoQEvictionMap},
				eviction_map::two_q_eviction_map::StackLocation,
			},
		};

		let mut access = Access {
			timestamp: 1,
			command: Command::Get,
			key: 0,
			size: 1,
			ttl: 0,
		};

		let mut eviction_map = TwoQEvictionMap::new(&access, 0.25, 0.50);
		assert_eq!(eviction_map.stack_location_at(100), Some(StackLocation::A1(1)));

		access.timestamp += 1;
		eviction_map.update(&access);
		assert_eq!(eviction_map.stack_location_at(100), Some(StackLocation::A1(1)));

		eviction_map.insert(100);
		assert_eq!(eviction_map.stack_location_at(96), None);
		assert_eq!(eviction_map.stack_location_at(100), None);
		assert_eq!(eviction_map.stack_location_at(104), Some(StackLocation::A1(1)));

		access.timestamp += 1;
		eviction_map.update(&access);
		assert_eq!(eviction_map.stack_location_at(96), Some(StackLocation::A1(3)));
		assert_eq!(eviction_map.stack_location_at(100), Some(StackLocation::A1(3)));
		assert_eq!(eviction_map.stack_location_at(104), Some(StackLocation::A1(1)));
	}
}
