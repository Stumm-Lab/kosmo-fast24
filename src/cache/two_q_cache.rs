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

pub struct TwoQCache {
	max_size: u64,

	kin: f64,
	kout: f64,

	count: f64,
	hits: f64,

	map: FxHashMap<Key, StackIndex>,

	ain: Stack,
	aout: Stack,
	am: Stack,
}

#[derive(Default)]
struct Stack {
	stack: VecList<Object>,
	size: u64,
}

enum StackIndex {
	Ain(Index<Object>),
	Aout(Index<Object>),
	Am(Index<Object>),
}

impl TwoQCache {
	pub fn new(size: u64, kin: f64, kout: f64) -> Self {
		assert!(kin > 0.0);
		assert!(kout > 0.0);
		assert!(kin + kout <= 1.0);

		TwoQCache {
			max_size: size,

			kin,
			kout,

			count: 0.0,
			hits: 0.0,

			map: FxHashMap::default(),

			ain: Stack::default(),
			aout: Stack::default(),
			am: Stack::default(),
		}
	}
}

impl Cache for TwoQCache {
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
		let Some(stack_index) = self.map.get(&access.key) else {
			return false;
		};

		match stack_index {
			StackIndex::Aout(index) => {
				let object = self.aout.remove(*index).unwrap();
				let index = self.am.push_front(object);
				let stack_index = StackIndex::Am(index);

				self.map.insert(access.key, stack_index);
			},

			StackIndex::Am(index) => {
				let object = self.am.remove(*index).unwrap();
				let index = self.am.push_front(object);
				let stack_index = StackIndex::Am(index);

				self.map.insert(access.key, stack_index);
			},

			_ => {},
		};

		true
	}

	fn process_set(&mut self, access: &Access) {
		if access.size as u64 > self.max_size || self.has(access.key) {
			return;
		}

		self.reduce(self.max_size - access.size as u64);

		let object = Object::new(access);
		let index = self.ain.push_front(object);

		self.map.insert(access.key, StackIndex::Ain(index));
	}

	fn process_del(&mut self, key: Key) {
		let Some(stack_index) = self.map.remove(&key) else {
			return;
		};

		match stack_index {
			StackIndex::Ain(index) => self.ain.remove(index),
			StackIndex::Aout(index) => self.aout.remove(index),
			StackIndex::Am(index) => self.am.remove(index),
		};
	}

	fn process_has(&self, key: Key) -> bool {
		self.map.contains_key(&key)
	}

	fn reduce(&mut self, target_size: u64) {
		let object_size = (self.max_size - target_size) as u32;

		while !self.ain.is_empty() && !self.can_ain_fit(object_size) {
			self.promote_ain_object();
		}

		while self.is_aout_full() ||
			(!self.aout.is_empty() && self.current_size() > target_size)
		{
			self.evict_aout();
		}

		while self.current_size() > target_size {
			self.evict_am();
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

impl TwoQCache {
	fn current_size(&self) -> u64 {
		self.ain.size + self.aout.size + self.am.size
	}

	fn ain_max_size(&self) -> u64 {
		(self.kin * self.max_size as f64) as u64
	}

	fn is_aout_full(&self) -> bool {
		self.aout.size > (self.kout * self.max_size as f64) as u64
	}

	fn can_ain_fit(&self, object_size: u32) -> bool {
		self.ain.size + object_size as u64 <= self.ain_max_size()
	}

	fn evict_aout(&mut self) {
		if let Some(object) = self.aout.pop_back() {
			self.map.remove(&object.key);
		}
	}

	fn evict_am(&mut self) {
		if let Some(object) = self.am.pop_back() {
			self.map.remove(&object.key);
		}
	}

	fn promote_ain_object(&mut self) {
		let object = self.ain.pop_back().unwrap();
		let key = object.key;

		let index = self.aout.push_front(object);
		let stack_index = StackIndex::Aout(index);

		self.map.insert(key, stack_index);
	}
}

impl Stack {
	fn is_empty(&self) -> bool {
		self.stack.is_empty()
	}

	fn remove(&mut self, index: Index<Object>) -> Option<Object> {
		let object = self.stack.remove(index);

		if let Some(object) = &object {
			self.size -= object.size as u64;
		}

		object
	}

	fn push_front(&mut self, object: Object) -> Index<Object> {
		self.size += object.size as u64;
		self.stack.push_front(object)
	}

	fn pop_back(&mut self) -> Option<Object> {
		let object = self.stack.pop_back();

		if let Some(object) = &object {
			self.size -= object.size as u64;
		}

		object
	}
}
