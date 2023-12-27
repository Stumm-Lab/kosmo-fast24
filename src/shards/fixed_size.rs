/*
 * Copyright (c) Kia Shakiba
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::{
	collections::BTreeSet,
	cmp::{Ord, Ordering},
};

use rustc_hash::FxHashSet;

use crate::{
	shards::Shards,
	access::{Access, Key},
};

pub struct ShardsFixedSize {
	global_t: u64,
	s_max: u32,

	sampled_count: u64,
	total_count: u64,
	expected_count: f64,

	entries: BTreeSet<Entry>,
	keys: FxHashSet<Key>,
}

struct Entry {
	key: Key,
	t: u64,
}

impl ShardsFixedSize {
	#[allow(dead_code)]
	pub fn new(global_t: u64, s_max: u32) -> Self {
		ShardsFixedSize {
			global_t,
			s_max,

			sampled_count: 0,
			total_count: 0,
			expected_count: 0.0,

			entries: BTreeSet::new(),
			keys: FxHashSet::default(),
		}
	}
}

impl Shards for ShardsFixedSize {
	fn get_global_t(&self) -> u64 {
		self.global_t
	}

	fn get_sampled_count(&self) -> u64 {
		self.sampled_count
	}

	fn get_total_count(&self) -> u64 {
		self.total_count
	}

	fn get_expected_count(&self) -> u64 {
		(self.expected_count + self.total_count as f64 * self.get_rate()) as u64
	}

	fn get_correction(&self) -> i64 {
		0
	}

	fn sample(&mut self, access: &Access) -> bool {
		self.total_count += 1;

		let Some(t) = self.sample_key(access.key) else {
			return false;
		};

		self.sampled_count += 1;

		let entry = Entry::new(access.key, t);

		if !self.entries.contains(&entry) {
			self.entries.insert(entry);
			self.keys.insert(access.key);
		}

		true
	}

	fn get_removal(&mut self) -> Option<Key> {
		if self.entries.len() <= self.s_max as usize {
			return None;
		}

		if let Some(entry) = self.entries.pop_first() {
			self.global_t = entry.t;
			self.keys.remove(&entry.key);

			self.expected_count += self.total_count as f64 * self.get_rate();
			self.total_count = 0;

			return Some(entry.key);
		}

		None
	}
}

impl Entry {
	fn new(key: Key, t: u64) -> Self {
		Entry {
			key,
			t,
		}
	}
}

impl Ord for Entry {
	fn cmp(&self, other: &Self) -> Ordering {
		other.t.cmp(&self.t)
	}
}

impl PartialOrd for Entry {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl PartialEq for Entry {
	fn eq(&self, other: &Self) -> bool {
		self.key == other.key
	}
}

impl Eq for Entry {}
