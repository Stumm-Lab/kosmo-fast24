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

pub struct LfuCache {
	max_size: u64,
	current_size: u64,

	count: f64,
	hits: f64,

	map: FxHashMap<Key, LfuObjectIndex>,
	count_lists: VecList<CountList>,
}

struct CountList {
	count: u64,
	list: VecList<LfuObject>,
}

#[derive(Clone, Default)]
struct LfuObjectIndex {
	count_list_index: Option<Index<CountList>>,
	list_index: Option<Index<LfuObject>>,
}

struct LfuObject {
	object: Object,
	index: LfuObjectIndex,
}

impl LfuCache {
	pub fn new(size: u64) -> Self {
		LfuCache {
			max_size: size,
			current_size: 0,

			count: 0.0,
			hits: 0.0,

			map: FxHashMap::default(),
			count_lists: VecList::new(),
		}
	}
}

impl Cache for LfuCache {
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
		let Some(lfu_object_index) = self.map.get(&access.key) else {
			return false;
		};

		let prev_count_list_index = lfu_object_index.count_list_index.unwrap();
		let prev_count_list = self.count_lists.get_mut(prev_count_list_index).unwrap();
		let prev_count = prev_count_list.count;

		let lfu_object = prev_count_list.remove(lfu_object_index.list_index.unwrap());
		let prev_is_empty = prev_count_list.is_empty();

		if let Some(next_count_list_index) = self.count_lists.get_next_index(prev_count_list_index) {
			let next_count_list = self.count_lists.get_mut(next_count_list_index).unwrap();

			if next_count_list.count == prev_count + 1 {
				let mut lfu_object_index = next_count_list.push(lfu_object);
				lfu_object_index.count_list_index = Some(next_count_list_index);

				self.map.insert(access.key, lfu_object_index);

				if prev_is_empty {
					self.count_lists.remove(prev_count_list_index);
				}

				return true;
			}
		}

		let mut count_list = CountList::new(prev_count + 1);

		let mut lfu_object_index = count_list.push(lfu_object);
		let next_count_list_index = self.count_lists.insert_after(prev_count_list_index, count_list);

		lfu_object_index.count_list_index = Some(next_count_list_index);

		self.map.insert(access.key, lfu_object_index);

		if prev_is_empty {
			self.count_lists.remove(prev_count_list_index);
		}

		true
	}

	fn process_set(&mut self, access: &Access) {
		if access.size as u64 > self.max_size || self.has(access.key) {
			return;
		}

		self.reduce(self.max_size - access.size as u64);

		if self.count_lists.is_empty() || self.count_lists.front().unwrap().count > 1 {
			self.count_lists.push_front(CountList::new(1));
		}

		let mut lfu_object = LfuObject::new(access);
		let count_list_index = self.count_lists.front_index().unwrap();
		let count_list = self.count_lists.get_mut(count_list_index).unwrap();

		lfu_object.index.count_list_index = Some(count_list_index);

		self.map.insert(access.key, count_list.push(lfu_object));

		self.current_size += access.size as u64;
	}

	fn process_del(&mut self, key: Key) {
		let Some(lfu_object_index) = self.map.remove(&key) else {
			return;
		};

		let count_list_index = lfu_object_index.count_list_index.unwrap();
		let count_list = self.count_lists.get_mut(count_list_index).unwrap();

		let lfu_object = count_list.remove(lfu_object_index.list_index.unwrap());
		self.current_size -= lfu_object.object.size as u64;

		if count_list.is_empty() {
			self.count_lists.remove(count_list_index);
		}
	}

	fn process_has(&self, key: Key) -> bool {
		self.map.contains_key(&key)
	}

	fn reduce(&mut self, target_size: u64) {
		while self.current_size > target_size {
			let count_list_index = self.count_lists.front_index().unwrap();
			let count_list = self.count_lists.get_mut(count_list_index).unwrap();

			let lfu_object = count_list.pop();

			self.map.remove(&lfu_object.object.key);
			self.current_size -= lfu_object.object.size as u64;

			if count_list.is_empty() {
				self.count_lists.remove(count_list_index);
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

impl CountList {
	fn new(count: u64) -> Self {
		CountList {
			count,
			list: VecList::new(),
		}
	}

	fn is_empty(&self) -> bool {
		self.list.is_empty()
	}

	fn push(&mut self, lfu_object: LfuObject) -> LfuObjectIndex {
		let index = self.list.push_front(lfu_object);
		let lfu_object = self.list.get_mut(index).unwrap();

		lfu_object.index.list_index = Some(index);
		lfu_object.index.clone()
	}

	fn pop(&mut self) -> LfuObject {
		self.list.pop_back().unwrap()
	}

	fn remove(&mut self, index: Index<LfuObject>) -> LfuObject {
		self.list.remove(index).unwrap()
	}
}

impl LfuObject {
	fn new(access: &Access) -> Self {
		LfuObject {
			object: Object::new(access),
			index: LfuObjectIndex::default(),
		}
	}
}
